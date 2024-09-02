use super::Identifier;
use crate::{
    ast::{function::FunctionDeclaration, variable::VariableDeclaration},
    keyword::{Keyword, ModifierFlag},
    *,
};
use ast::{expression::Expression, typeparam::TypeParameter};
use core::ops::ControlFlow;
use token::{Token, TokenPosition, TokenStream};

#[derive(Debug)]
pub struct ClassDeclaration {
    modifiers: ModifierFlag,
    identifier: Identifier,
    type_params: Vec<TypeParameter>,
    super_class: Option<Identifier>,
    variables: Box<[VariableDeclaration]>,
    functions: Box<[FunctionDeclaration]>,
    position: TokenPosition,
}

impl ClassDeclaration {
    pub fn parse(
        modifier_tokens: &[Token<Keyword>],
        decisive_token: Token<Keyword>,
        tokens: &mut TokenStream<Keyword>,
    ) -> Result<Self, CompileError> {
        let start_token = modifier_tokens
            .iter()
            .min_by(|a, b| a.position().start().cmp(&b.position().start()))
            .unwrap_or(&decisive_token);

        let modifiers =
            ModifierFlag::from_keywords(modifier_tokens.iter().filter_map(
                |v| match v.token_type() {
                    TokenType::Keyword(keyword) => Some(*keyword),
                    _ => None,
                },
            ))
            .map_err(|err| {
                let token = modifier_tokens
                    .iter()
                    .find_map(|v| match v.token_type() {
                        TokenType::Keyword(keyword) => (*keyword == err).then(|| v),
                        _ => None,
                    })
                    .unwrap();
                CompileError::unexpected_token(token)
            })?;

        let identifier = Identifier::from_tokens(tokens)?;

        let type_params = TypeParameter::parse(tokens)?;

        let super_class = if tokens.expect_keyword(Keyword::Extends).is_ok() {
            let super_id = Identifier::from_tokens(tokens)?;
            Some(super_id)
        } else {
            None
        };

        expect_symbol(tokens, '{')?;
        let mut variables = Vec::new();
        let mut functions = Vec::new();

        {
            let mut modifiers = Vec::new();
            loop {
                if tokens.expect_symbol('}').is_ok() {
                    break;
                }

                let token = tokens.next_non_blank();
                match token.token_type() {
                    TokenType::Keyword(keyword) => {
                        if keyword.is_modifier() {
                            modifiers.push(token);
                            continue;
                        }
                        match keyword {
                            //
                            _ => return Err(CompileError::unexpected_token(&token)),
                        }
                    }
                    TokenType::Identifier => {
                        let kind = tokens
                            .transaction::<_, _, ()>(|tokens| {
                                if tokens.expect_symbol(':').is_ok() {
                                    return ControlFlow::Break(MemberKind::Variable);
                                } else if tokens.expect_symbol('=').is_ok() {
                                    return ControlFlow::Break(MemberKind::Variable);
                                } else if tokens.expect_symbol('(').is_ok() {
                                    return ControlFlow::Break(MemberKind::Function);
                                } else {
                                    return ControlFlow::Break(MemberKind::Err(
                                        CompileError::unexpected_token(&tokens.shift().unwrap()),
                                    ));
                                }
                            })
                            .unwrap_err();

                        match kind {
                            MemberKind::Variable => {
                                tokens.unshift();
                                let member = VariableDeclaration::parse(
                                    modifiers.as_slice(),
                                    token,
                                    tokens,
                                )?;
                                variables.push(member);
                            }
                            MemberKind::Function => {
                                tokens.unshift();
                                let member = FunctionDeclaration::parse(
                                    modifiers.as_slice(),
                                    token,
                                    tokens,
                                )?;
                                functions.push(member);
                            }
                            MemberKind::Err(err) => return Err(err),
                        }
                        modifiers.clear();
                    }
                    _ => return Err(CompileError::unexpected_token(&token)),
                }
            }
        }

        let position = start_token
            .position()
            .merged(&tokens.peek_last().unwrap().position());

        Ok(Self {
            modifiers,
            identifier,
            type_params,
            super_class,
            variables: variables.into_boxed_slice(),
            functions: functions.into_boxed_slice(),
            position,
        })
    }

    #[inline]
    pub fn modifiers(&self) -> ModifierFlag {
        self.modifiers
    }

    #[inline]
    pub fn identifier(&self) -> &Identifier {
        &self.identifier
    }

    #[inline]
    pub fn type_params(&self) -> &[TypeParameter] {
        &self.type_params
    }

    #[inline]
    pub fn super_class(&self) -> Option<&Identifier> {
        self.super_class.as_ref()
    }

    #[inline]
    pub fn variables(&self) -> &[VariableDeclaration] {
        &self.variables
    }

    #[inline]
    pub fn functions(&self) -> &[FunctionDeclaration] {
        &self.functions
    }

    #[inline]
    pub fn position(&self) -> TokenPosition {
        self.position
    }
}

enum MemberKind {
    Variable,
    Function,
    Err(CompileError),
}

#[derive(Debug)]
pub struct EnumDeclaration {
    modifiers: ModifierFlag,
    identifier: Identifier,
    variants: Vec<(Identifier, Option<Expression>)>,
}

impl EnumDeclaration {
    pub fn parse(
        modifier_tokens: &[Token<Keyword>],
        decisive_token: Token<Keyword>,
        tokens: &mut TokenStream<Keyword>,
    ) -> Result<Self, CompileError> {
        let _start_token = modifier_tokens
            .iter()
            .min_by(|a, b| a.position().start().cmp(&b.position().start()))
            .unwrap_or(&decisive_token);

        let modifiers =
            ModifierFlag::from_keywords(modifier_tokens.iter().filter_map(
                |v| match v.token_type() {
                    TokenType::Keyword(keyword) => Some(*keyword),
                    _ => None,
                },
            ))
            .map_err(|err| {
                let token = modifier_tokens
                    .iter()
                    .find_map(|v| match v.token_type() {
                        TokenType::Keyword(keyword) => (*keyword == err).then(|| v),
                        _ => None,
                    })
                    .unwrap();
                CompileError::unexpected_token(token)
            })?;

        let identifier = Identifier::from_tokens(tokens)?;

        let mut variants = Vec::new();

        if modifiers.contains(ModifierFlag::IMPORT) {
            expect_symbol(tokens, ';')?;
        } else {
            expect_symbol(tokens, '{')?;
            loop {
                let token = tokens.next_non_blank();
                match token.token_type() {
                    TokenType::Symbol('}') => break,
                    TokenType::Identifier => {
                        let identifier = Identifier::new(token.source(), token.position());

                        let assignment = if tokens.expect_symbol('=').is_ok() {
                            Some(Expression::parse(tokens, &[TokenType::Symbol(',')])?)
                        } else {
                            None
                        };
                        variants.push((identifier, assignment));

                        if tokens.expect_symbol('}').is_ok() {
                            break;
                        }
                        expect_symbol(tokens, ',')?;
                    }

                    _ => {
                        return Err(CompileError::missing_token(
                            &token,
                            &[TokenType::Symbol('}'), TokenType::Identifier],
                        ))
                    }
                }
                //
            }
            expect_eol(tokens)?;
        }

        Ok(Self {
            modifiers,
            identifier,
            variants,
        })
    }

    #[inline]
    pub fn modifiers(&self) -> ModifierFlag {
        self.modifiers
    }

    #[inline]
    pub fn identifier(&self) -> &Identifier {
        &self.identifier
    }

    #[inline]
    pub fn variants(&self) -> &[(Identifier, Option<Expression>)] {
        &self.variants
    }
}

#[derive(Debug, PartialEq)]
pub enum TypeDescriptor {
    Simple(Identifier),
}

impl TypeDescriptor {
    pub fn expect(tokens: &mut TokenStream<Keyword>) -> Result<Self, CompileError> {
        let token = tokens.next_non_blank();
        match token.token_type() {
            TokenType::Identifier => Identifier::parse(token, tokens).map(|v| Self::Simple(v)),
            TokenType::Keyword(keyword) => {
                if keyword.is_type_identifier() {
                    Ok(Self::Simple(Identifier::from_keyword(
                        *keyword,
                        token.position(),
                    )))
                } else {
                    Err(CompileError::with_token(
                        CompileErrorKind::SyntaxError,
                        &token,
                        Some("Expected Type".to_string()),
                    ))
                }
            }
            _ => Err(CompileError::with_token(
                CompileErrorKind::SyntaxError,
                &token,
                Some("Expected Type".to_string()),
            )),
        }
    }

    pub fn position(&self) -> TokenPosition {
        match self {
            Self::Simple(v) => v.id_position(),
        }
    }

    #[inline]
    pub fn as_string(&self) -> &String {
        self.identifier().as_string()
    }

    pub fn identifier(&self) -> &Identifier {
        match self {
            TypeDescriptor::Simple(v) => v,
        }
    }
}

impl Clone for TypeDescriptor {
    fn clone(&self) -> Self {
        match self {
            Self::Simple(arg0) => Self::Simple(arg0.clone()),
        }
    }
}
