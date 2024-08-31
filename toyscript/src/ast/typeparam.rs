use super::*;
use token::TokenStream;

#[derive(Debug)]
pub struct TypeParameter {
    identifier: Identifier,
    extends: Option<Identifier>,
}

impl TypeParameter {
    pub fn parse(tokens: &mut TokenStream<Keyword>) -> Result<Vec<TypeParameter>, CompileError> {
        let mut type_params = Vec::new();

        if tokens.expect_symbol('<').is_err() {
            return Ok(type_params);
        }

        loop {
            let identifier = Identifier::from_tokens(tokens)?;
            let extends: Option<Identifier> = if tokens.expect_keyword(Keyword::Extends).is_ok() {
                let extends = expect_type(tokens)?;
                Some(extends)
            } else {
                None
            };
            let type_param = Self {
                identifier,
                extends,
            };
            type_params.push(type_param);

            let next = expect(tokens, &[TokenType::Symbol('>'), TokenType::Symbol(',')])?;
            if matches!(next.token_type(), TokenType::Symbol('>')) {
                return Ok(type_params);
            }
        }
    }

    #[inline]
    pub fn identifier(&self) -> &Identifier {
        &self.identifier
    }

    #[inline]
    pub fn extends(&self) -> Option<&Identifier> {
        self.extends.as_ref()
    }
}
