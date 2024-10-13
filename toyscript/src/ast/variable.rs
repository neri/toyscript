//! Variable Declarations

use crate::*;
use ast::{class::TypeDeclaration, expression::Expression, typeparam::TypeParameter, Identifier};
use keyword::Keyword;
use token::{Token, TokenPosition, TokenStream};

#[derive(Debug, Clone)]
pub struct VariableDeclaration {
    modifiers: Box<[Keyword]>,
    variables: Box<[Variable]>,
}

#[derive(Debug, Clone)]
pub struct Variable {
    identifier: Identifier,
    type_decl: Option<TypeDeclaration>,
    assignment: Option<Expression>,
    position: TokenPosition,
    is_mutable: bool,
}

impl VariableDeclaration {
    pub fn parse(
        modifier_tokens: &[Token<Keyword>],
        decisive_token: Token<Keyword>,
        tokens: &mut TokenStream<Keyword>,
        ending_tokens: Option<&[TokenType<Keyword>]>,
        type_params: &[TypeParameter],
    ) -> Result<Self, CompileError> {
        let allowed_ending = ending_tokens.unwrap_or(&[
            TokenType::Eof,
            TokenType::NewLine,
            TokenType::Symbol(','),
            TokenType::Symbol(';'),
            TokenType::Symbol('}'),
        ]);
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

            let position_start = identifier.id_position().start();

            let type_desc = if let Ok(_) = tokens.expect_symbol(':') {
                let var_type = TypeDeclaration::expect(tokens, type_params)?;
                Some(var_type)
            } else {
                None
            };

            let assignment = if let Ok(_) = tokens.expect_symbol('=') {
                let assignment = Expression::parse(tokens, Some(allowed_ending), type_params)?;
                Some(assignment)
            } else {
                None
            };

            let position_end = tokens.peek_last().unwrap().position().end();

            variables.push(Variable {
                identifier,
                type_decl: type_desc,
                assignment,
                position: TokenPosition::new(position_start as u32, position_end as u32),
                is_mutable,
            });

            if try_expect_eol(tokens).is_ok() || tokens.expect_symbol(',').is_err() {
                break;
            }
        }

        if ending_tokens.is_none() {
            expect_eol(tokens)?;
        }

        Ok(Self {
            modifiers: modifiers.into_boxed_slice(),
            variables: variables.into_boxed_slice(),
        })
    }

    pub fn parse_declare(
        modifier_tokens: &[Token<Keyword>],
        decisive_token: Token<Keyword>,
        tokens: &mut TokenStream<Keyword>,
        type_params: &[TypeParameter],
    ) -> Result<Self, CompileError> {
        let position_start = modifier_tokens
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

        let identifier = Identifier::from_tokens(tokens)?;

        let type_decl = if let Ok(_) = tokens.expect_symbol(':') {
            Some(TypeDeclaration::expect(tokens, type_params)?)
        } else {
            None
        };

        let position_end = tokens.peek_last().unwrap().position().end();

        variables.push(Variable {
            identifier,
            type_decl,
            assignment: None,
            position: TokenPosition::new(position_start as u32, position_end as u32),
            is_mutable,
        });

        expect_eol(tokens)?;

        Ok(Self {
            modifiers: modifiers.into_boxed_slice(),
            variables: variables.into_boxed_slice(),
        })
    }

    #[inline]
    pub fn modifiers(&self) -> &[Keyword] {
        &self.modifiers
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
    pub fn type_decl(&self) -> Option<&TypeDeclaration> {
        self.type_decl.as_ref()
    }

    #[inline]
    pub fn assignment(&self) -> Option<&Expression> {
        self.assignment.as_ref()
    }

    #[inline]
    pub fn position(&self) -> TokenPosition {
        self.position
    }

    #[inline]
    pub fn is_mutable(&self) -> bool {
        self.is_mutable
    }
}
