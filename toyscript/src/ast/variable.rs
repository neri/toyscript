//! Variable Declarations

use super::expression::Expression;
use super::Identifier;
use crate::keyword::Keyword;
use crate::*;
use ast::class::TypeDescriptor;
use token::{Token, TokenStream};

#[allow(dead_code)]
#[derive(Debug)]
pub struct VariableDeclaration {
    modifiers: Box<[Keyword]>,
    variables: Box<[Variable]>,
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct Variable {
    identifier: Identifier,
    type_desc: Option<TypeDescriptor>,
    assignment: Option<Expression>,
    is_mutable: bool,
}

impl VariableDeclaration {
    pub fn parse(
        modifier_tokens: &[Token<Keyword>],
        decisive_token: Token<Keyword>,
        tokens: &mut TokenStream<Keyword>,
    ) -> Result<Self, CompileError> {
        let _position_start = modifier_tokens
            .iter()
            .min_by(|a, b| a.position().start().cmp(&b.position().start()))
            .unwrap_or(&decisive_token)
            .position()
            .start();

        let mut modifiers = modifier_tokens
            .iter()
            .flat_map(|v| match v.token_type() {
                TokenType::Keyword(keyword) => Some(*keyword),
                _ => None,
            })
            .collect::<Vec<_>>();
        match decisive_token.token_type() {
            TokenType::Keyword(keyword) => {
                modifiers.push(*keyword);
            }
            _ => (),
        }

        let is_mutable = !modifiers.contains(&Keyword::Const);

        let mut variables = Vec::new();

        loop {
            let identifier = Identifier::from_tokens(tokens)?;

            let type_desc = if let Ok(_) = tokens.expect_symbol(':') {
                let var_type = TypeDescriptor::expect(tokens)?;
                Some(var_type)
            } else {
                None
            };

            let assignment = if let Ok(_) = tokens.expect_symbol('=') {
                let assignment = Expression::parse(
                    tokens,
                    &[
                        TokenType::NewLine,
                        TokenType::Symbol(','),
                        TokenType::Symbol(';'),
                        TokenType::Symbol('}'),
                    ],
                )?;
                Some(assignment)
            } else {
                None
            };

            variables.push(Variable {
                identifier,
                type_desc,
                assignment,
                is_mutable,
            });

            if try_expect_eol(tokens).is_ok() || tokens.expect_symbol(',').is_err() {
                break;
            }
        }

        expect_eol(tokens)?;

        Ok(Self {
            modifiers: modifiers.into_boxed_slice(),
            variables: variables.into_boxed_slice(),
        })
    }

    pub fn parse_declare(
        modifier_tokens: &[Token<Keyword>],
        decisive_token: Token<Keyword>,
        tokens: &mut TokenStream<Keyword>,
    ) -> Result<Self, CompileError> {
        let mut modifiers = modifier_tokens
            .iter()
            .flat_map(|v| match v.token_type() {
                TokenType::Keyword(keyword) => Some(*keyword),
                _ => None,
            })
            .collect::<Vec<_>>();
        match decisive_token.token_type() {
            TokenType::Keyword(keyword) => {
                modifiers.push(*keyword);
            }
            _ => (),
        }

        let is_mutable = !modifiers.contains(&Keyword::Const);

        let mut variables = Vec::new();

        let identifier = Identifier::from_tokens(tokens)?;

        let type_desc = if let Ok(_) = tokens.expect_symbol(':') {
            Some(TypeDescriptor::expect(tokens)?)
        } else {
            None
        };

        variables.push(Variable {
            identifier,
            type_desc,
            assignment: None,
            is_mutable,
        });

        expect_eol(tokens)?;

        Ok(Self {
            modifiers: modifiers.into_boxed_slice(),
            variables: variables.into_boxed_slice(),
        })
    }

    #[inline]
    pub fn varibales(&self) -> &[Variable] {
        &self.variables
    }
}

impl Variable {
    #[inline]
    pub fn identifier(&self) -> &Identifier {
        &self.identifier
    }

    #[inline]
    pub fn type_desc(&self) -> Option<&TypeDescriptor> {
        self.type_desc.as_ref()
    }

    #[inline]
    pub fn assignment(&self) -> Option<&Expression> {
        self.assignment.as_ref()
    }

    #[inline]
    pub fn is_mutable(&self) -> bool {
        self.is_mutable
    }
}
