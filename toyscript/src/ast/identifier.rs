use token::ArcStringSlice;

use super::*;
use crate::*;

#[derive(Clone)]
pub struct Identifier {
    str: ArcStringSlice,
}

impl Identifier {
    #[inline]
    pub fn from_token(token: &Token<Keyword>) -> Self {
        Self {
            str: token.source_arc().clone(),
        }
    }

    #[inline]
    pub fn new(identifier: &str, id_position: TokenPosition) -> Self {
        Self {
            str: ArcStringSlice::from_str(identifier, id_position),
        }
    }

    #[inline]
    pub fn from_keyword(keyword: Keyword, id_position: TokenPosition) -> Self {
        Self {
            str: ArcStringSlice::from_str(keyword.as_str(), id_position),
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
    pub fn as_string(&self) -> String {
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

// impl Clone for Identifier {
//     #[inline]
//     fn clone(&self) -> Self {
//         Self {
//             str: self.str.clone()
//         }
//     }
// }

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
