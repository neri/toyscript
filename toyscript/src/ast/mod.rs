//! ToyScript Abstract Syntax Tree

pub mod block;
pub mod class;
pub mod decorator;
pub mod expression;
pub mod float;
pub mod function;
pub mod identifier;
pub mod integer;
pub mod statement;
pub mod typeparam;
pub mod variable;

use crate::*;
use keyword::Keyword;
use statement::Statement;
use token::TokenStream;

/// ToyScript Abstract Syntax Tree
pub struct Ast {
    items: Vec<Statement>,
}

impl Ast {
    #[inline]
    pub fn module(&self) -> &[Statement] {
        &self.items
    }

    #[inline]
    pub fn into_module(self) -> Vec<Statement> {
        self.items
    }

    pub fn from_tokens(tokens: &mut TokenStream<Keyword>) -> Result<Ast, CompileError> {
        let mut statements = Vec::new();

        loop {
            match Statement::parse(tokens, &[])? {
                Statement::Eof(_) => break,
                statement => statements.push(statement),
            }
        }

        Ok(Self { items: statements })
    }
}

impl core::fmt::Debug for Ast {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Ast: {:#?}", self.items)
    }
}
