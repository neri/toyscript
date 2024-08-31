use super::Identifier;
use crate::{
    ast::{function::FunctionDeclaration, variable::VariableDeclaration},
    keyword::{Keyword, ModifierFlag},
    *,
};
use core::ops::ControlFlow;
use token::{Token, TokenPosition, TokenStream};

#[derive(Debug)]
pub struct ClassDeclaration {
    modifiers: ModifierFlag,
    identifier: Identifier,
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

                let token = tokens.shift().unwrap();
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
                            .transaction::<_, _, ()>(|snapshot| {
                                if snapshot.expect_symbol(':').is_ok() {
                                    return ControlFlow::Break(MemberKind::Variable);
                                } else if snapshot.expect_symbol('=').is_ok() {
                                    return ControlFlow::Break(MemberKind::Variable);
                                } else if snapshot.expect_symbol('(').is_ok() {
                                    return ControlFlow::Break(MemberKind::Function);
                                } else {
                                    return ControlFlow::Break(MemberKind::Err(
                                        CompileError::unexpected_token(&snapshot.shift().unwrap()),
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
