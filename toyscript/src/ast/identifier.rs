use crate::*;
use token::{ArcStringSlice, TokenPosition};

#[derive(Clone)]
pub struct Identifier {
    str: ArcStringSlice,
}

impl Identifier {
    #[inline]
    pub fn from_token(token: &Token<Keyword>) -> Self {
        Self {
            str: token.as_arc().clone(),
        }
    }

    #[inline]
    pub fn new(identifier: &str) -> Self {
        let position = TokenPosition::new(0, identifier.len() as u32);
        Self {
            str: ArcStringSlice::from_str(identifier, position),
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
                let identifier = Identifier::from_token(&leading);
                return Ok(identifier);
            }
            _ => (),
        }
        return Err(CompileError::unexpected_token(&leading));
    }

    #[inline]
    pub fn as_str(&self) -> &str {
        self.str.as_str()
    }

    #[inline]
    pub fn to_string(&self) -> String {
        self.str.to_string()
    }

    #[inline]
    pub fn id_position(&self) -> TokenPosition {
        self.str.position()
    }
}

impl PartialEq for Identifier {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.str.as_str() == other.str.as_str()
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
        write!(f, "Identifier({})", self.as_str())
    }
}
