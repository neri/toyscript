use super::*;
use crate::*;

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
        write!(f, "Identifier({})", self.as_str())
    }
}
