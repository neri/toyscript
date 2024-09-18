use crate::*;
use token::TokenPosition;

#[derive(Debug)]
pub struct Decoration {
    identifier: Identifier,
    params: Vec<Token<Keyword>>,
    position: TokenPosition,
}

impl Decoration {
    pub fn parse(
        tokens: &mut TokenStream<Keyword>,
        decisive_token: Token<Keyword>,
    ) -> Result<Self, CompileError> {
        let start_token = &decisive_token;

        let token = tokens.next_immed().ok_or(CompileError::missing_token(
            &tokens.eof(),
            &[TokenType::Identifier],
        ))?;
        if !matches!(token.token_type(), TokenType::Identifier) {
            return Err(CompileError::missing_token(
                &token,
                &[TokenType::Identifier],
            ));
        }
        let identifier = Identifier::from_token(&token);

        let mut params = Vec::new();

        if expect_symbol(tokens, '(').is_ok() {
            loop {
                let token = tokens.next_non_blank();
                match token.token_type() {
                    TokenType::Symbol(')') => break,
                    TokenType::Keyword(_)
                    | TokenType::Identifier
                    | TokenType::NumericLiteral
                    | TokenType::FloatingNumberLiteral
                    | TokenType::StringLiteral(_) => {
                        params.push(token);
                    }
                    _ => {
                        return Err(CompileError::unexpected_token(&token));
                    }
                }

                let next = expect(tokens, &[TokenType::Symbol(','), TokenType::Symbol(')')])?;
                if matches!(next.token_type(), TokenType::Symbol(')')) {
                    break;
                }
            }
        }

        let position = start_token
            .position()
            .merged(&tokens.peek_last().unwrap().position());

        Ok(Self {
            identifier,
            params,
            position,
        })
    }

    #[inline]
    pub fn position(&self) -> TokenPosition {
        self.position
    }

    #[inline]
    pub fn identifier(&self) -> &Identifier {
        &self.identifier
    }

    #[inline]
    pub fn params(&self) -> &[Token<Keyword>] {
        &self.params
    }
}
