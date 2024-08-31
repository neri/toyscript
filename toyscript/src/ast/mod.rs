//! ToyScript Abstract Syntax Tree

pub mod block;
pub mod class;
pub mod expression;
pub mod function;
// pub mod integer;
pub mod statement;
pub mod typeparam;
pub mod variable;

use self::statement::Statement;
use crate::keyword::Keyword;
use crate::*;
use token::{Token, TokenPosition, TokenStream};

/// ToyScript Abstract Syntax Tree
pub struct Ast {
    items: Vec<Statement>,
}

impl Ast {
    #[inline]
    pub fn program(&self) -> &[Statement] {
        &self.items
    }

    pub fn from_tokens(tokens: &mut TokenStream<Keyword>) -> Result<Ast, CompileError> {
        let mut statements = Vec::new();

        loop {
            match Statement::parse(tokens)? {
                Statement::Eof(_) => break,
                statement => statements.push(statement),
            }
        }

        Ok(Self { items: statements })
    }
}

impl core::fmt::Debug for Ast {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Program: {:#?}", self.items)
    }
}

pub struct Identifier {
    identifier: String,
    id_position: TokenPosition,
}

impl Identifier {
    #[inline]
    pub fn new(identifier: &str, id_position: TokenPosition) -> Self {
        Self {
            identifier: identifier.to_string(),
            id_position,
        }
    }

    #[inline]
    pub fn from_keyword(keyword: Keyword, id_position: TokenPosition) -> Self {
        Self {
            identifier: keyword.as_str().to_string(),
            id_position,
        }
    }

    #[inline]
    pub fn from_tokens(tokens: &mut TokenStream<Keyword>) -> Result<Self, CompileError> {
        let id_token = expect(tokens, &[TokenType::Identifier])?;
        Self::parse(id_token, tokens)
    }

    pub fn parse(
        leading: Token<Keyword>,
        tokens: &mut TokenStream<Keyword>,
    ) -> Result<Self, CompileError> {
        let _ = tokens;
        match leading.token_type() {
            TokenType::Identifier => {
                let identifier = Identifier::new(leading.source(), leading.position());
                return Ok(identifier);
            }
            _ => (),
        }
        return Err(CompileError::unexpected_token(&leading));
    }

    #[inline]
    pub fn as_str(&self) -> &str {
        &self.identifier
    }

    #[inline]
    pub fn to_string(&self) -> String {
        self.as_str().to_string()
    }

    #[inline]
    pub fn id_position(&self) -> TokenPosition {
        self.id_position
    }
}

impl PartialEq for Identifier {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.identifier == other.identifier
    }
}

impl Clone for Identifier {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            identifier: self.identifier.clone(),
            id_position: self.id_position,
        }
    }
}

impl core::fmt::Display for Identifier {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl core::fmt::Debug for Identifier {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:#?}", self.as_str())
    }
}
