use super::Identifier;
use crate::{
    ast::{function::FunctionDeclaration, variable::VariableDeclaration},
    keyword::{Keyword, ModifierFlag},
    *,
};
use ast::{
    decorator::Decorator, expression::Expression, function::FunctionSyntaxFlavor,
    typeparam::TypeParameter,
};
use core::ops::ControlFlow;
use token::{Token, TokenPosition, TokenStream};

#[derive(Debug)]
pub struct ClassDeclaration {
    decorators: Vec<Decorator>,
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
        decorators: Vec<Decorator>,
        modifier_tokens: &[Token<Keyword>],
        decisive_token: Token<Keyword>,
        tokens: &mut TokenStream<Keyword>,
    ) -> Result<Self, CompileError> {
        let start_token = modifier_tokens
            .iter()
            .min_by(|a, b| a.position().start().cmp(&b.position().start()))
            .unwrap_or(&decisive_token);

        let modifiers = ModifierFlag::from_tokens(modifier_tokens, &[])
            .map_err(|token| CompileError::unexpected_token(&token))?;

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
                            Keyword::Constructor => {
                                if !modifiers.is_empty() {
                                    return Err(CompileError::unexpected_token(&token));
                                }
                                let member = FunctionDeclaration::parse(
                                    FunctionSyntaxFlavor::Class,
                                    Vec::new(),
                                    modifiers.as_slice(),
                                    &token,
                                    Some(&token),
                                    tokens,
                                )?;
                                functions.push(member);
                            }
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
                                    None,
                                )?;
                                variables.push(member);
                            }
                            MemberKind::Function => {
                                let member = FunctionDeclaration::parse(
                                    FunctionSyntaxFlavor::Class,
                                    Vec::new(),
                                    modifiers.as_slice(),
                                    &token,
                                    Some(&token),
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
            decorators,
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
    pub fn decorations(&self) -> &[Decorator] {
        &self.decorators
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
    decorations: Vec<Decorator>,
    modifiers: ModifierFlag,
    identifier: Identifier,
    variants: Vec<(Identifier, Option<Expression>)>,
}

impl EnumDeclaration {
    pub fn parse(
        decorations: Vec<Decorator>,
        modifier_tokens: &[Token<Keyword>],
        decisive_token: Token<Keyword>,
        tokens: &mut TokenStream<Keyword>,
    ) -> Result<Self, CompileError> {
        let _start_token = modifier_tokens
            .iter()
            .min_by(|a, b| a.position().start().cmp(&b.position().start()))
            .unwrap_or(&decisive_token);

        let modifiers = ModifierFlag::from_tokens(modifier_tokens, &[])
            .map_err(|token| CompileError::unexpected_token(&token))?;

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
                        let identifier = Identifier::from_token(&token);

                        let assignment = if tokens.expect_symbol('=').is_ok() {
                            Some(Expression::parse(tokens, ending_mode!(','))?)
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
            decorations,
            modifiers,
            identifier,
            variants,
        })
    }

    #[inline]
    pub fn decorations(&self) -> &[Decorator] {
        &self.decorations
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
pub enum TypeDeclaration {
    Simple(Identifier),
}

impl TypeDeclaration {
    pub fn expect(tokens: &mut TokenStream<Keyword>) -> Result<Self, CompileError> {
        let token = tokens.next_non_blank();
        match token.token_type() {
            TokenType::Identifier => Identifier::parse(token, tokens).map(|v| Self::Simple(v)),
            TokenType::Keyword(keyword) => {
                if keyword.is_type_identifier() {
                    Ok(Self::Simple(Identifier::from_token(&token)))
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
    pub fn as_string(&self) -> String {
        self.identifier().as_string()
    }

    pub fn identifier(&self) -> &Identifier {
        match self {
            TypeDeclaration::Simple(v) => v,
        }
    }
}

impl Clone for TypeDeclaration {
    fn clone(&self) -> Self {
        match self {
            Self::Simple(arg0) => Self::Simple(arg0.clone()),
        }
    }
}
