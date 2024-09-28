//! Statements

use crate::{keyword::Keyword, *};
use ast::{
    block::Block,
    class::{ClassDeclaration, EnumDeclaration, TypeDeclaration},
    decorator::Decorator,
    expression::Expression,
    function::{FunctionDeclaration, FunctionSyntaxFlavor},
    variable::VariableDeclaration,
    Identifier,
};
use token::{TokenPosition, TokenStream};

#[derive(Debug)]
pub enum Statement {
    /// end of file marker
    Eof(TokenPosition),

    /// `{` statement... `}`
    Block(Block),

    /// `function`
    Function(Arc<FunctionDeclaration>),

    /// Variable Declaration
    Variable(VariableDeclaration),

    /// Type Declaration
    TypeAlias(Identifier, TypeDeclaration),

    /// Class Declaration
    Class(Arc<ClassDeclaration>),

    /// Enum Declaration
    Enum(Arc<EnumDeclaration>),

    /// `if` expr `{` block `}` `else` `if` expr `{` block `}` `else` `{` block `}`
    IfStatement(Box<[IfType]>),

    /// `for` `(` expr `;` expr `;` expr `)` `{` block `}`
    ForStatement(Box<ForStatement>),

    /// `while` expr `{` block `}`
    WhileStatement(Expression, Block),

    /// `return` expr
    ReturnStatement(Expression),

    /// expression
    Expression(Expression),

    /// `break`
    Break(TokenPosition),

    /// `continue`
    Continue(TokenPosition),
}

#[derive(Debug)]
pub enum IfType {
    If(Expression, Block),
    ElseIf(Expression, Block),
    Else(Block),
}

#[derive(Debug)]
pub struct ForStatement {
    pub(crate) init: ForInit,
    pub(crate) cond: Expression,
    pub(crate) step: Expression,
    pub(crate) block: Block,
}

#[derive(Debug)]
pub enum ForInit {
    Var(VariableDeclaration),
    Expr(Expression),
}

impl Statement {
    pub fn parse(tokens: &mut TokenStream<Keyword>) -> Result<Self, CompileError> {
        let mut modifiers = Vec::new();
        let mut decorators = Vec::<Decorator>::new();
        loop {
            let token = tokens.next_non_blank();
            match token.token_type() {
                TokenType::Eof => {
                    if !modifiers.is_empty() {
                        return Err(CompileError::unexpected_token(&token));
                    }
                    if let Some(item) = decorators.first() {
                        return Err(CompileError::unexpected_token(
                            &tokens.get_raw(item.position()),
                        ));
                    }
                    return Ok(Self::Eof(token.position()));
                }
                TokenType::Keyword(keyword) => {
                    if keyword.is_modifier() {
                        modifiers.push(token);
                        continue;
                    }
                    match keyword {
                        Keyword::Function => {
                            let func_decl = FunctionDeclaration::parse(
                                FunctionSyntaxFlavor::Function,
                                decorators,
                                modifiers.as_slice(),
                                &token,
                                None,
                                tokens,
                            )?;
                            return Ok(Self::Function(Arc::new(func_decl)));
                        }
                        Keyword::Const | Keyword::Let | Keyword::Var => {
                            if let Some(item) = decorators.first() {
                                return Err(CompileError::unexpected_token(
                                    &tokens.get_raw(item.position()),
                                ));
                            }
                            let var_decl = VariableDeclaration::parse(
                                modifiers.as_slice(),
                                token,
                                tokens,
                                None,
                            )?;
                            return Ok(Self::Variable(var_decl));
                        }
                        Keyword::Class => {
                            let class_decl = ClassDeclaration::parse(
                                decorators,
                                modifiers.as_slice(),
                                token,
                                tokens,
                            )?;
                            return Ok(Self::Class(Arc::new(class_decl)));
                        }
                        Keyword::Enum => {
                            let enum_decl = EnumDeclaration::parse(
                                decorators,
                                modifiers.as_slice(),
                                token,
                                tokens,
                            )?;
                            return Ok(Self::Enum(Arc::new(enum_decl)));
                        }
                        Keyword::For => {
                            if !modifiers.is_empty() {
                                return Err(CompileError::unexpected_token(&token));
                            }
                            if let Some(item) = decorators.first() {
                                return Err(CompileError::unexpected_token(
                                    &tokens.get_raw(item.position()),
                                ));
                            }

                            expect_symbol(tokens, '(')?;

                            let init = if let Ok(token) =
                                expect(tokens, &[TokenType::Keyword(Keyword::Let)])
                            {
                                ForInit::Var(VariableDeclaration::parse(
                                    &[],
                                    token,
                                    tokens,
                                    Some(&[TokenType::Symbol(','), TokenType::Symbol(';')]),
                                )?)
                            } else {
                                ForInit::Expr(Expression::parse(tokens, ending_mode!(';'))?)
                            };

                            expect_symbol(tokens, ';')?;
                            let cond = Expression::parse(tokens, ending_mode!(';'))?;
                            expect_symbol(tokens, ';')?;
                            let step = Expression::parse(tokens, ending_mode!(')'))?;
                            expect_symbol(tokens, ')')?;

                            let begin_block = expect_symbol(tokens, '{')?;
                            let block = Block::parse(begin_block, tokens)?;
                            return Ok(Self::ForStatement(Box::new(ForStatement {
                                init,
                                cond,
                                step,
                                block,
                            })));
                        }

                        Keyword::Declare => {
                            let kind = tokens.next_non_blank();
                            match kind.token_type() {
                                TokenType::Keyword(keyword) => match keyword {
                                    Keyword::Function => {
                                        let func_decl = FunctionDeclaration::parse(
                                            FunctionSyntaxFlavor::Declare,
                                            decorators,
                                            &modifiers,
                                            &kind,
                                            None,
                                            tokens,
                                        )?;
                                        return Ok(Self::Function(Arc::new(func_decl)));
                                    }
                                    // Keyword::Const | Keyword::Let | Keyword::Var => {
                                    //     let var_decl = VariableDeclaration::parse_declare(
                                    //         modifiers.as_slice(),
                                    //         token,
                                    //         tokens,
                                    //     )?;
                                    //     return Ok(Self::Variable(var_decl));
                                    // }
                                    Keyword::Type => {
                                        if !modifiers.is_empty() {
                                            return Err(CompileError::unexpected_token(&token));
                                        }
                                        if let Some(item) = decorators.first() {
                                            return Err(CompileError::unexpected_token(
                                                &tokens.get_raw(item.position()),
                                            ));
                                        }
                                        let identifier = Identifier::from_tokens(tokens)?;
                                        expect_symbol(tokens, '=')?;
                                        let type_decl = TypeDeclaration::expect(tokens)?;
                                        expect_eol(tokens)?;
                                        return Ok(Self::TypeAlias(identifier, type_decl));
                                    }
                                    _ => return Err(CompileError::unexpected_token(&token)),
                                },
                                _ => return Err(CompileError::unexpected_token(&token)),
                            }
                        }
                        Keyword::Return => {
                            if !modifiers.is_empty() {
                                return Err(CompileError::unexpected_token(&token));
                            }
                            if let Some(item) = decorators.first() {
                                return Err(CompileError::unexpected_token(
                                    &tokens.get_raw(item.position()),
                                ));
                            }

                            if expect_eol(tokens).is_ok() {
                                return Ok(Self::ReturnStatement(Expression::empty_with_position(
                                    token.position(),
                                )));
                            } else {
                                let expr: Expression = Expression::parse(tokens, None)?;
                                expect_eol(tokens)?;
                                return Ok(Self::ReturnStatement(expr));
                            }
                        }
                        Keyword::If => {
                            if !modifiers.is_empty() {
                                return Err(CompileError::unexpected_token(&token));
                            }
                            if let Some(item) = decorators.first() {
                                return Err(CompileError::unexpected_token(
                                    &tokens.get_raw(item.position()),
                                ));
                            }

                            let expr = Expression::parse(tokens, ending_mode!('{'))?;
                            let begin_block = expect_symbol(tokens, '{')?;
                            let block = Block::parse(begin_block, tokens)?;
                            let mut statements = Vec::new();
                            statements.push(IfType::If(expr, block));
                            loop {
                                if tokens.expect_keyword(Keyword::Else).is_err() {
                                    break;
                                }
                                if tokens.expect(&[TokenType::Keyword(Keyword::If)]).is_ok() {
                                    let expr = Expression::parse(tokens, ending_mode!('{'))?;
                                    let begin_block = expect_symbol(tokens, '{')?;
                                    let block = Block::parse(begin_block, tokens)?;
                                    statements.push(IfType::ElseIf(expr, block));
                                } else {
                                    let begin_block = expect_symbol(tokens, '{')?;
                                    let block = Block::parse(begin_block, tokens)?;
                                    statements.push(IfType::Else(block));
                                }
                            }
                            return Ok(Self::IfStatement(statements.into_boxed_slice()));
                        }
                        Keyword::While => {
                            if !modifiers.is_empty() {
                                return Err(CompileError::unexpected_token(&token));
                            }
                            if let Some(item) = decorators.first() {
                                return Err(CompileError::unexpected_token(
                                    &tokens.get_raw(item.position()),
                                ));
                            }

                            let expr = Expression::parse(tokens, ending_mode!('{'))?;
                            let begin_block = expect_symbol(tokens, '{')?;
                            let block = Block::parse(begin_block, tokens)?;
                            return Ok(Self::WhileStatement(expr, block));
                        }
                        Keyword::Break => {
                            if !modifiers.is_empty() {
                                return Err(CompileError::unexpected_token(&token));
                            }
                            if let Some(item) = decorators.first() {
                                return Err(CompileError::unexpected_token(
                                    &tokens.get_raw(item.position()),
                                ));
                            }

                            expect_eol(tokens)?;
                            return Ok(Self::Break(token.position()));
                        }
                        Keyword::Continue => {
                            if !modifiers.is_empty() {
                                return Err(CompileError::unexpected_token(&token));
                            }
                            if let Some(item) = decorators.first() {
                                return Err(CompileError::unexpected_token(
                                    &tokens.get_raw(item.position()),
                                ));
                            }

                            expect_eol(tokens)?;
                            return Ok(Self::Continue(token.position()));
                        }
                        _ => {
                            if !modifiers.is_empty() {
                                return Err(CompileError::unexpected_token(&token));
                            }
                            if let Some(item) = decorators.first() {
                                return Err(CompileError::unexpected_token(
                                    &tokens.get_raw(item.position()),
                                ));
                            }

                            tokens.unshift();
                            let expr = Expression::parse(tokens, None)?;
                            expect_eol(tokens)?;
                            return Ok(Self::Expression(expr));
                        }
                    }
                }
                TokenType::Symbol('{') => {
                    if !modifiers.is_empty() {
                        return Err(CompileError::unexpected_token(&token));
                    }
                    if let Some(item) = decorators.first() {
                        return Err(CompileError::unexpected_token(
                            &tokens.get_raw(item.position()),
                        ));
                    }

                    let block = Block::parse(token, tokens)?;
                    return Ok(Self::Block(block));
                }
                TokenType::Symbol('@') => {
                    if !modifiers.is_empty() {
                        return Err(CompileError::unexpected_token(&token));
                    }
                    let decoration = Decorator::parse(tokens, token)?;
                    decorators.push(decoration);
                    continue;
                }
                _ => {
                    if !modifiers.is_empty() {
                        return Err(CompileError::unexpected_token(&token));
                    }
                    if let Some(item) = decorators.first() {
                        return Err(CompileError::unexpected_token(
                            &tokens.get_raw(item.position()),
                        ));
                    }

                    tokens.unshift();
                    let expr = Expression::parse(tokens, None)?;
                    expect_eol(tokens)?;
                    return Ok(Self::Expression(expr));
                }
            }
        }
    }
}
