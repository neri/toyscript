//! Function Declarations

use super::{block::Block, typeparam::TypeParameter, Identifier};
use crate::{keyword::ModifierFlag, *};
use ast::{class::TypeDescriptor, decoration::Decoration};
use token::{Token, TokenPosition, TokenStream};

#[derive(Debug)]
pub struct FunctionDeclaration {
    decorations: Vec<Decoration>,
    modifiers: ModifierFlag,
    identifier: Identifier,
    import_from: Option<(String, String)>,
    type_params: Vec<TypeParameter>,
    parameters: Box<[Parameter]>,
    result_type: Option<TypeDescriptor>,
    body: Block,
    position: TokenPosition,
}

pub enum FunctionSyntaxFlavor {
    Function,
    Declare,
    Class,
}

impl FunctionDeclaration {
    pub fn parse(
        flavor: FunctionSyntaxFlavor,
        decorations: Vec<Decoration>,
        modifier_tokens: &[Token<Keyword>],
        decisive_token: Token<Keyword>,
        tokens: &mut TokenStream<Keyword>,
    ) -> Result<Self, CompileError> {
        let start_token = modifier_tokens
            .iter()
            .min_by(|a, b| a.position().start().cmp(&b.position().start()))
            .unwrap_or(&decisive_token);

        let modifiers = ModifierFlag::from_tokens(modifier_tokens, &[ModifierFlag::EXPORT])
            .map_err(|token| CompileError::unexpected_token(&token))?;

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
            let var_type = TypeDescriptor::expect(tokens)?;
            parameters.push(Parameter::new(var_name, var_type));

            let next = expect(tokens, &[TokenType::Symbol(','), TokenType::Symbol(')')])?;
            if matches!(next.token_type(), TokenType::Symbol(')')) {
                break;
            }
        }

        let result_type = if tokens.expect_symbol(':').is_ok() {
            Some(TypeDescriptor::expect(tokens)?)
        } else {
            None
        };

        let block;
        let import_from;
        match flavor {
            FunctionSyntaxFlavor::Function => {
                let block_begin = expect_symbol(tokens, '{')?;
                block = Block::parse(block_begin, tokens)?;
                import_from = None;
            }
            FunctionSyntaxFlavor::Declare => {
                block = Block::empty();
                import_from = Some(("env".to_owned(), identifier.as_string()));
            }
            FunctionSyntaxFlavor::Class => {
                // TODO:
                let block_begin = expect_symbol(tokens, '{')?;
                block = Block::parse(block_begin, tokens)?;
                import_from = None;
            }
        }

        let position = start_token
            .position()
            .merged(&tokens.peek_last().unwrap().position());

        Ok(FunctionDeclaration {
            decorations,
            modifiers,
            identifier,
            import_from,
            type_params,
            parameters: parameters.into_boxed_slice(),
            result_type,
            body: block,
            position,
        })
    }

    #[inline]
    pub fn decorations(&self) -> &[Decoration] {
        &self.decorations
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
    pub fn import_from(&self) -> Option<(&str, &str)> {
        self.import_from
            .as_ref()
            .map(|v| (v.0.as_str(), v.1.as_str()))
    }

    #[inline]
    pub fn type_params(&self) -> &[TypeParameter] {
        &self.type_params
    }

    #[inline]
    pub fn parameters(&self) -> &[Parameter] {
        &self.parameters
    }

    #[inline]
    pub fn result_type(&self) -> Option<&TypeDescriptor> {
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

#[derive(Debug, PartialEq)]
pub struct Parameter {
    identifier: Identifier,
    type_desc: TypeDescriptor,
}

impl Parameter {
    #[inline]
    pub fn new(identifier: Identifier, type_desc: TypeDescriptor) -> Self {
        Self {
            identifier,
            type_desc,
        }
    }

    #[inline]
    pub fn identifier(&self) -> &Identifier {
        &self.identifier
    }

    #[inline]
    pub fn type_desc(&self) -> &TypeDescriptor {
        &self.type_desc
    }
}
