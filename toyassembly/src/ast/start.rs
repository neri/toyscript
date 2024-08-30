//! Memory section
use super::{keyword::Keyword, ModuleName};
use crate::*;
use ast::*;
use identifier::IndexToken;

#[derive(Debug)]
pub struct Start(pub IndexToken);

impl ModuleName for Start {
    const IDENTIFIER: Keyword = Keyword::Start;

    fn from_tokens(tokens: &mut TokenStream<Keyword>) -> Result<Self, ParseError> {
        let index = IndexToken::expect(tokens)?;

        expect(tokens, &[TokenType::CloseParenthesis])?;

        Ok(Self(index))
    }
}
