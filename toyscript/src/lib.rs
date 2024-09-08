//! ToyScript
#![cfg_attr(not(test), no_std)]

pub mod ast;
pub mod cg;
pub mod error;
pub mod keyword;
pub mod types;

#[cfg(test)]
pub mod tests;

extern crate alloc;

#[allow(unused)]
pub(crate) use alloc::{
    borrow::{Cow, ToOwned},
    boxed::Box,
    collections::BTreeMap,
    rc::Rc,
    string::{String, ToString},
    sync::Arc,
    vec::Vec,
};
#[allow(unused)]
pub(crate) use core::{cell::RefCell, str};
pub use error::*;

use alloc::format;
use ast::{identifier::Identifier, Ast};
use keyword::Keyword;
use token::{Token, TokenStream, TokenType, Tokenizer};
use types::TypeSystem;

pub struct ToyScript;

impl ToyScript {
    fn _from_src<F, R>(file_name: &str, src: Vec<u8>, kernel: F) -> Result<R, String>
    where
        F: FnOnce(&mut TokenStream<Keyword>) -> Result<R, CompileError>,
    {
        let src = Arc::new(src);
        let tokens = Tokenizer::new(src.clone(), Keyword::from_str).map_err(|e| {
            let position = ErrorPosition::CharAt(e.position().0, e.position().1);
            CompileError::with_kind(CompileErrorKind::TokenParseError(e), position)
                .to_detail_string(file_name, &src, &[])
        })?;

        kernel(&mut tokens.stream())
            .map_err(|e| e.to_detail_string(file_name, &src, tokens.line_positions()))
    }

    pub fn debug_ast(file_name: &str, src: Vec<u8>) -> Result<String, String> {
        Self::_from_src(file_name, src, |tokens| Ast::from_tokens(tokens))
            .map(|v| format!("{:#?}", v))
    }

    pub fn debug_ir(file_name: &str, src: Vec<u8>) -> Result<String, String> {
        Self::_from_src(file_name, src, |tokens| {
            let ast = Ast::from_tokens(tokens)?;

            let types = TypeSystem::new(file_name, &ast)?;

            let ir_module = cg::CodeGen::generate(&ast, &types)?;

            Ok(ir_module)
        })
        .map(|v| format!("{:#?}", v))
    }
}

#[inline]
pub(crate) fn expect(
    tokens: &mut TokenStream<Keyword>,
    expected: &[TokenType<Keyword>],
) -> Result<Token<Keyword>, CompileError> {
    tokens
        .expect(expected)
        .map_err(|e| CompileError::missing_token(&e, expected))
}

#[inline]
pub(crate) fn expect_symbol(
    tokens: &mut TokenStream<Keyword>,
    expected: char,
) -> Result<Token<Keyword>, CompileError> {
    expect(tokens, &[TokenType::Symbol(expected)])
}

pub(crate) fn try_expect_eol(
    tokens: &mut TokenStream<Keyword>,
) -> Result<Token<Keyword>, CompileError> {
    if let Some(token) = tokens.peek() {
        match token.token_type() {
            TokenType::NewLine | TokenType::Symbol(';') => {
                return Ok(token);
            }
            TokenType::Symbol('}') => return Ok(token),
            _ => (),
        }
        Err(CompileError::missing_eol(&token))
    } else {
        Err(CompileError::missing_eol(&tokens.eof()))
    }
}

pub(crate) fn expect_eol(tokens: &mut TokenStream<Keyword>) -> Result<(), CompileError> {
    try_expect_eol(tokens).map(|token| match token.token_type() {
        TokenType::NewLine | TokenType::Symbol(';') => {
            tokens.shift();
        }
        TokenType::Symbol('}') => (),
        _ => (),
    })
}
