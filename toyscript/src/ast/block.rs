//! Block of Statements

use super::statement::Statement;
use crate::*;
use token::{Token, TokenPosition, TokenStream};

pub struct Block {
    statements: Arc<[Statement]>,
    position: TokenPosition,
}

impl Block {
    #[inline]
    pub fn empty() -> Self {
        Self {
            statements: Arc::new([]),
            position: TokenPosition::empty(),
        }
    }

    pub fn parse(
        decisive_token: Token<Keyword>,
        tokens: &mut TokenStream<Keyword>,
    ) -> Result<Self, CompileError> {
        let start_token = decisive_token;

        if let Ok(token) = tokens.expect_symbol('}') {
            return Ok(Self {
                statements: Arc::from(Vec::new().into_boxed_slice()),
                position: start_token.position().merged(&token.position()),
            });
        }

        let mut statements = Vec::new();

        let _end_block = loop {
            match tokens.expect(&[
                TokenType::Symbol(';'),
                TokenType::Symbol('{'),
                TokenType::Symbol('}'),
                TokenType::Eof,
            ]) {
                Ok(token) => match token.token_type() {
                    TokenType::Symbol(';') => continue,
                    TokenType::Symbol('{') => {
                        let block = Block::parse(token, tokens)?;
                        statements.push(Statement::Block(block));
                    }
                    TokenType::Symbol('}') => {
                        break token;
                    }
                    TokenType::Eof => {
                        return Err(CompileError::missing_token(
                            &token,
                            &[TokenType::Symbol('}')],
                        ))
                    }
                    _ => unreachable!(),
                },
                Err(_) => {
                    let statement = Statement::parse(tokens)?;
                    statements.push(statement);
                }
            }
        };

        let position = start_token
            .position()
            .merged(&tokens.peek_last().unwrap().position());

        Ok(Self {
            statements: Arc::from(statements.into_boxed_slice()),
            position,
        })
    }

    #[inline]
    pub fn from_statements(statements: Vec<Statement>) -> Self {
        Self {
            statements: Arc::from(statements.into_boxed_slice()),
            // TODO:
            position: TokenPosition::empty(),
        }
    }

    #[inline]
    pub fn statements(&self) -> &Arc<[Statement]> {
        &self.statements
    }

    #[inline]
    pub fn position(&self) -> TokenPosition {
        self.position
    }
}

impl core::fmt::Debug for Block {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_list().entries(self.statements.as_ref()).finish()
    }
}

impl Clone for Block {
    fn clone(&self) -> Self {
        Self {
            statements: self.statements.clone(),
            position: self.position.clone(),
        }
    }
}
