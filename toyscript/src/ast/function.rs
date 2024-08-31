//! Function Declarations

use super::{block::Block, typeparam::TypeParameter, Identifier};
use crate::{keyword::ModifierFlag, *};
use token::{Token, TokenPosition, TokenStream};

#[derive(Debug)]
pub struct FunctionDeclaration {
    modifiers: ModifierFlag,
    identifier: Identifier,
    type_params: Vec<TypeParameter>,
    parameters: Box<[Parameter]>,
    result_type: Option<Identifier>,
    body: Block,
    position: TokenPosition,
}

impl FunctionDeclaration {
    pub fn parse(
        modifier_tokens: &[Token<Keyword>],
        decisive_token: Token<Keyword>,
        tokens: &mut TokenStream<Keyword>,
    ) -> Result<Self, CompileError> {
        let start_token = modifier_tokens
            .iter()
            .min_by(|a, b| a.position().start().cmp(&b.position().start()))
            .unwrap_or(&decisive_token);

        let modifiers =
            ModifierFlag::from_keywords(modifier_tokens.iter().filter_map(
                |v| match v.token_type() {
                    TokenType::Keyword(keyword) => Some(*keyword),
                    _ => None,
                },
            ))
            .map_err(|err| {
                let token = modifier_tokens
                    .iter()
                    .find_map(|v| match v.token_type() {
                        TokenType::Keyword(keyword) => (*keyword == err).then(|| v),
                        _ => None,
                    })
                    .unwrap();
                CompileError::unexpected_token(token)
            })?;

        let identifier = Identifier::from_tokens(tokens)?;

        let type_params = TypeParameter::parse(tokens)?;

        expect_symbol(tokens, '(')?;
        let mut parameters = Vec::new();
        loop {
            let id_or_end = expect(tokens, &[TokenType::Identifier, TokenType::Symbol(')')])?;
            let var_name = match id_or_end.token_type() {
                TokenType::Identifier => Identifier::parse(id_or_end, tokens)?,
                TokenType::Symbol(')') => break,
                _ => unreachable!(),
            };

            expect_symbol(tokens, ':')?;
            let var_type = expect_type(tokens)?;
            parameters.push(Parameter::new(var_name, var_type));

            let next = expect(tokens, &[TokenType::Symbol(','), TokenType::Symbol(')')])?;
            if matches!(next.token_type(), TokenType::Symbol(')')) {
                break;
            }
        }

        let result_type = if tokens.expect_symbol(':').is_ok() {
            let result_type = expect_type(tokens)?;
            Some(result_type)
        } else {
            None
        };

        let block = if modifiers.contains(ModifierFlag::DECLARE) {
            expect_eol(tokens)?;
            Block::empty()
        } else {
            let block_begin = expect_symbol(tokens, '{')?;
            let block = Block::parse(block_begin, tokens)?;
            block
        };

        let position = start_token
            .position()
            .merged(&tokens.peek_last().unwrap().position());

        Ok(FunctionDeclaration {
            modifiers,
            identifier,
            type_params,
            parameters: parameters.into_boxed_slice(),
            result_type,
            body: block,
            position,
        })
    }

    #[inline]
    pub fn modifiers(&self) -> ModifierFlag {
        self.modifiers
    }

    #[inline]
    pub fn identifier(&self) -> &Identifier {
        &self.identifier
    }

    #[inline]
    pub fn parameters(&self) -> &[Parameter] {
        &self.parameters
    }

    #[inline]
    pub fn result_type(&self) -> Option<&Identifier> {
        self.result_type.as_ref()
    }

    #[inline]
    pub fn position(&self) -> TokenPosition {
        self.position
    }

    #[inline]
    pub fn body(&self) -> &Block {
        &self.body
    }
}

#[allow(dead_code)]
#[derive(Debug, PartialEq)]
pub struct Parameter {
    identifier: Identifier,
    type_id: Identifier,
}

impl Parameter {
    #[inline]
    pub fn new(identifier: Identifier, type_id: Identifier) -> Self {
        Self {
            identifier,
            type_id,
        }
    }

    #[inline]
    pub fn identifier(&self) -> &Identifier {
        &self.identifier
    }

    #[inline]
    pub fn type_id(&self) -> &Identifier {
        &self.type_id
    }
}
