#![cfg_attr(not(test), no_std)]

pub mod ast;
pub mod error;
pub mod keyword;
// pub mod types;

#[cfg(test)]
pub mod tests;

extern crate alloc;

#[allow(unused)]
pub(crate) use alloc::{
    borrow::Cow,
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
use keyword::Keyword;

use alloc::format;
use ast::{Ast, Identifier};
// use cg::CodeGen;
// use namespace::NamespaceOld;
use token::{Token, TokenStream, TokenType, Tokenizer};
// use types::TypeSystem;

pub struct ToyScript {
    //
}

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
        Err(CompileError::missing_eol(&Token::eof()))
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

pub(crate) fn expect_type(tokens: &mut TokenStream<Keyword>) -> Result<Identifier, CompileError> {
    tokens.skip_ignorable();
    let token = tokens.shift().unwrap();
    match token.token_type() {
        TokenType::Identifier => Identifier::parse(token, tokens),
        TokenType::Keyword(keyword) => {
            if keyword.is_type_identifier() {
                Ok(Identifier::from_keyword(*keyword, token.position()))
            } else {
                Err(CompileError::with_token(
                    CompileErrorKind::SyntaxError,
                    &token,
                    Some("Expected TypeIdentifier".to_string()),
                ))
            }
        }
        _ => Err(CompileError::with_token(
            CompileErrorKind::SyntaxError,
            &token,
            Some("Expected TypeIdentifier".to_string()),
        )),
    }
}
