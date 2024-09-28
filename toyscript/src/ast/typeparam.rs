use super::*;
use class::TypeDeclaration;
use token::TokenStream;

#[derive(Debug, Clone)]
pub struct TypeParameter {
    identifier: Identifier,
    extends: Option<TypeDeclaration>,
}

impl TypeParameter {
    pub fn parse(tokens: &mut TokenStream<Keyword>) -> Result<Vec<TypeParameter>, CompileError> {
        let mut type_params = Vec::new();

        if tokens.expect_symbol('<').is_err() {
            return Ok(type_params);
        }

        loop {
            let identifier = Identifier::from_tokens(tokens)?;
            let extends = if tokens.expect_keyword(Keyword::Extends).is_ok() {
                Some(TypeDeclaration::expect(tokens)?)
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
    pub fn extends(&self) -> Option<&TypeDeclaration> {
        self.extends.as_ref()
    }
}
