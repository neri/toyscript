//! Identifiers
use crate::*;
use ast::{expect, literal::NumericLiteral};
use core::{convert::Infallible, ops::ControlFlow};
use token::*;
use types::ValType;

#[derive(Debug)]
pub struct Identifier {
    name: String,
    position: TokenPosition,
}

impl Identifier {
    pub fn from_token<T>(token: &Token<T>) -> Result<Self, ParseError> {
        if ValType::from_str(token.source()).is_none() && token.source().starts_with("$") {
            Ok(Self {
                name: token.source().to_owned(),
                position: token.position(),
            })
        } else {
            Err(ParseError::invalid_identifier(
                token.source(),
                token.position().into(),
            ))
        }
    }

    pub fn try_expect<T>(tokens: &mut TokenStream<T>) -> Result<Option<Self>, ParseError>
    where
        T: Copy + PartialEq + core::fmt::Debug + core::fmt::Display,
    {
        match tokens.transaction(|tokens| {
            let token = tokens.next_non_blank();
            match token.token_type() {
                TokenType::Identifier => {
                    match tokens
                        .transaction(|tokens| {
                            let next = tokens.peek_immed().unwrap();
                            let expected = [
                                TokenType::Whitespace,
                                TokenType::NewLine,
                                TokenType::CloseParenthesis,
                            ];
                            if next.token_type().is_ignorable()
                                || expected.contains(next.token_type())
                            {
                                ControlFlow::<Result<(), ParseError>, Infallible>::Break(Ok(()))
                            } else {
                                ControlFlow::Break(Err(ParseError::missing_token(&expected, &next)))
                            }
                        })
                        .unwrap_err()
                    {
                        Ok(_) => {}
                        Err(e) => return ControlFlow::Break(Err(e)),
                    }

                    match Self::from_token(&token) {
                        Ok(v) => ControlFlow::Continue(Ok(Some(v))),
                        Err(_) => ControlFlow::Break(Ok(None)),
                    }
                }
                _ => ControlFlow::Break(Ok(None)),
            }
        }) {
            Ok(v) => v,
            Err(v) => v,
        }
    }

    #[inline]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[inline]
    pub fn position(&self) -> TokenPosition {
        self.position
    }
}

#[derive(Debug)]
pub enum IndexToken {
    Num(NumericLiteral<u32>),
    Id(Identifier),
}

impl IndexToken {
    pub fn try_expect<T>(tokens: &mut TokenStream<T>) -> Result<Option<Self>, ParseError>
    where
        T: PartialEq + Copy + core::fmt::Debug + core::fmt::Display,
    {
        if let Some(id) = Identifier::try_expect(tokens)? {
            Ok(Some(Self::Id(id)))
        } else if let Some(num) = NumericLiteral::<u32>::try_expect(tokens)? {
            Ok(Some(Self::Num(num)))
        } else {
            Ok(None)
        }
    }

    pub fn expect<T>(tokens: &mut TokenStream<T>) -> Result<Self, ParseError>
    where
        T: PartialEq + Copy + core::fmt::Debug + core::fmt::Display,
    {
        let token = expect(tokens, &[TokenType::Identifier, TokenType::NumericLiteral])?;
        match token.token_type() {
            TokenType::Identifier => Ok(Self::Id(Identifier::from_token(&token)?)),
            TokenType::NumericLiteral => Ok(Self::Num(NumericLiteral::<u32>::from_token(&token)?)),
            _ => unreachable!(),
        }
    }

    pub fn position(&self) -> TokenPosition {
        match self {
            IndexToken::Num(v) => v.position(),
            IndexToken::Id(v) => v.position(),
        }
    }
}
