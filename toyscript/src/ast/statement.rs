//! Statements

use crate::{keyword::Keyword, *};
use ast::{
    block::Block,
    class::{ClassDeclaration, EnumDeclaration, TypeDescriptor},
    expression::Expression,
    function::FunctionDeclaration,
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
    Function(FunctionDeclaration),

    /// Variable Declaration
    Variable(VariableDeclaration),

    /// Type Declaration
    TypeAlias(Identifier, TypeDescriptor),

    /// Class Declaration
    Class(ClassDeclaration),

    /// Enum Declaration
    Enum(EnumDeclaration),

    /// `if` expr `{` block `}` [[`else` `if` `{` block `}`]... `else` `{` block `}`]
    IfStatement(Vec<IfType>),

    /// `while` expr `{` block `}`
    WhileStatement(Expression, Block),

    /// `return` expr
    ReturnStatement(Expression),

    /// expression
    Expression(Expression),
}

#[derive(Debug)]
pub enum IfType {
    If(Expression, Block),
    ElseIf(Expression, Block),
    Else(Block),
}

impl Statement {
    pub fn parse(tokens: &mut TokenStream<Keyword>) -> Result<Self, CompileError> {
        let mut modifiers = Vec::new();
        loop {
            let token = tokens.next_non_blank();
            match token.token_type() {
                TokenType::Eof => {
                    if !modifiers.is_empty() {
                        return Err(CompileError::unexpected_token(&token));
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
                            let func_decl =
                                FunctionDeclaration::parse(modifiers.as_slice(), token, tokens)?;
                            return Ok(Self::Function(func_decl));
                        }
                        Keyword::Const | Keyword::Let | Keyword::Var => {
                            let var_decl =
                                VariableDeclaration::parse(modifiers.as_slice(), token, tokens)?;
                            return Ok(Self::Variable(var_decl));
                        }
                        Keyword::Class => {
                            let class_decl =
                                ClassDeclaration::parse(modifiers.as_slice(), token, tokens)?;
                            return Ok(Self::Class(class_decl));
                        }
                        Keyword::Enum => {
                            let enum_decl =
                                EnumDeclaration::parse(modifiers.as_slice(), token, tokens)?;
                            return Ok(Self::Enum(enum_decl));
                        }
                        Keyword::Declare => {
                            let kind = tokens.next_non_blank();
                            match kind.token_type() {
                                TokenType::Keyword(keyword) => match keyword {
                                    // Keyword::Function => {
                                    //     let func_decl =
                                    //         FunctionDeclaration::parse(&[token], kind, tokens)?;
                                    //     return Ok(Self::Function(func_decl));
                                    // }
                                    // Keyword::Const | Keyword::Let | Keyword::Var => {
                                    //     let var_decl = VariableDeclaration::parse_declare(
                                    //         modifiers.as_slice(),
                                    //         token,
                                    //         tokens,
                                    //     )?;
                                    //     return Ok(Self::Variable(var_decl));
                                    // }
                                    Keyword::Type => {
                                        let identifier = Identifier::from_tokens(tokens)?;
                                        expect_symbol(tokens, '=')?;
                                        let type_desc = TypeDescriptor::expect(tokens)?;
                                        expect_eol(tokens)?;
                                        return Ok(Self::TypeAlias(identifier, type_desc));
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
                            if expect_eol(tokens).is_ok() {
                                return Ok(Self::ReturnStatement(Expression::empty_with_position(
                                    token.position(),
                                )));
                            } else {
                                let expr: Expression = Expression::parse(tokens, &[])?;
                                expect_eol(tokens)?;
                                return Ok(Self::ReturnStatement(expr));
                            }
                        }
                        Keyword::If => {
                            if !modifiers.is_empty() {
                                return Err(CompileError::unexpected_token(&token));
                            }
                            let expr = Expression::parse(tokens, &[TokenType::Symbol('{')])?;
                            let begin_block = expect_symbol(tokens, '{')?;
                            let block = Block::parse(begin_block, tokens)?;
                            let mut statements = Vec::new();
                            statements.push(IfType::If(expr, block));
                            loop {
                                if tokens.expect_keyword(Keyword::Else).is_err() {
                                    break;
                                }
                                if tokens.expect(&[TokenType::Keyword(Keyword::If)]).is_ok() {
                                    let expr =
                                        Expression::parse(tokens, &[TokenType::Symbol('{')])?;
                                    let begin_block = expect_symbol(tokens, '{')?;
                                    let block = Block::parse(begin_block, tokens)?;
                                    statements.push(IfType::ElseIf(expr, block));
                                } else {
                                    let begin_block = expect_symbol(tokens, '{')?;
                                    let block = Block::parse(begin_block, tokens)?;
                                    statements.push(IfType::Else(block));
                                }
                            }
                            return Ok(Self::IfStatement(statements));
                        }
                        Keyword::While => {
                            if !modifiers.is_empty() {
                                return Err(CompileError::unexpected_token(&token));
                            }
                            let expr = Expression::parse(tokens, &[TokenType::Symbol('{')])?;
                            let begin_block = expect_symbol(tokens, '{')?;
                            let block = Block::parse(begin_block, tokens)?;
                            return Ok(Self::WhileStatement(expr, block));
                        }
                        _ => {
                            if !modifiers.is_empty() {
                                return Err(CompileError::unexpected_token(&token));
                            }
                            tokens.unshift();
                            let expr = Expression::parse(tokens, &[])?;
                            expect_eol(tokens)?;
                            return Ok(Self::Expression(expr));
                        }
                    }
                }
                TokenType::Symbol('{') => {
                    if !modifiers.is_empty() {
                        return Err(CompileError::unexpected_token(&token));
                    }
                    let block = Block::parse(token, tokens)?;
                    return Ok(Self::Block(block));
                }
                _ => {
                    if !modifiers.is_empty() {
                        return Err(CompileError::unexpected_token(&token));
                    }
                    tokens.unshift();
                    let expr = Expression::parse(tokens, &[])?;
                    expect_eol(tokens)?;
                    return Ok(Self::Expression(expr));
                }
            }
        }
    }
}
