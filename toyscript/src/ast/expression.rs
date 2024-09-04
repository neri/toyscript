//! Expressions

// use super::integer::*;
use super::Identifier;
use crate::{keyword::Keyword, *};
use ast::integer::Integer;
use core::ops::ControlFlow;
use token::{StringLiteralError, TokenPosition, TokenStream};

#[derive(Clone)]
pub struct Expression {
    item: Box<Unary>,
    position: TokenPosition,
}

#[derive(Clone)]
pub enum Unary {
    /// `()`
    Void(TokenPosition),

    /// Identifier
    Identifier(Identifier),

    /// Numeric Literal
    NumericLiteral(Integer, TokenPosition),

    /// String Literal
    StringLiteral(String, TokenPosition),

    /// Constant keyword
    Constant(Keyword, TokenPosition),

    /// `(` expression `)`
    Parenthesis(Expression),

    /// Unary Operator
    Unary(UnaryOperator, TokenPosition, Box<Unary>),

    /// Binary Operator
    Binary(BinaryOperator, TokenPosition, Box<Unary>, Box<Unary>),

    /// expression `[` expression `]`
    Subscript(Box<Unary>, Expression),

    /// identifier `.` identifier
    Member(Box<Unary>, Identifier),

    /// Invoke - (expression) `(` expression, ...`)`
    Invoke(Box<Unary>, Box<[Expression]>),
}

pub enum FlatUnary {
    Void(TokenPosition),

    Parenthesis(Expression),

    Value(Unary),

    Unary(UnaryOperator, TokenPosition),

    Binary(BinaryOperator, TokenPosition),

    Invoke(Box<[Expression]>),

    Subscript(Expression),

    Member(Identifier),
}

impl Expression {
    const DEFAULT_ENDING: [TokenType<Keyword>; 3] = [
        TokenType::NewLine,
        TokenType::Symbol(';'),
        TokenType::Symbol('}'),
    ];

    #[inline]
    pub fn empty_with_position(position: TokenPosition) -> Self {
        Self {
            item: Box::new(Unary::Void(position)),
            position,
        }
    }

    #[inline]
    pub fn from_uanary(value: Box<Unary>) -> Self {
        let position = value.position();
        Self {
            item: value,
            position,
        }
    }

    pub fn parse(
        tokens: &mut TokenStream<Keyword>,
        allowed_ending: &[TokenType<Keyword>],
    ) -> Result<Self, CompileError> {
        Self::_parse_unary(tokens, allowed_ending, true)
            .and_then(|(items, position)| Self::finalize(items, position))
    }

    fn _parse_unary(
        tokens: &mut TokenStream<Keyword>,
        allowed_ending: &[TokenType<Keyword>],
        allow_binary: bool,
    ) -> Result<(Vec<FlatUnary>, TokenPosition), CompileError> {
        let allowed_ending = if allowed_ending.is_empty() {
            &Self::DEFAULT_ENDING
        } else {
            allowed_ending
        };
        tokens.skip_ignorable();
        let position_start = tokens.peek().unwrap().position().start();

        let mut items = Vec::new();

        if tokens.expect_symbol('(').is_ok() {
            if tokens.expect_symbol(')').is_ok() {
                let position_end = tokens.peek().unwrap().position().end();
                items.push(FlatUnary::Void(TokenPosition::new(
                    position_start as u32,
                    position_end as u32,
                )));
            } else {
                let expr = Self::parse(tokens, &[TokenType::Symbol(')')])?;
                expect_symbol(tokens, ')')?;
                items.push(FlatUnary::Parenthesis(expr));
            }
        } else if let Some(token) = tokens.shift() {
            match token.token_type() {
                TokenType::Identifier => {
                    let identifier = Identifier::parse(token, tokens)?;
                    items.push(FlatUnary::Value(Unary::Identifier(identifier)));
                }
                TokenType::NumericLiteral => {
                    let v = Integer::from_str(token.source()).map_err(|_| {
                        CompileError::invalid_literal("Invalid number", token.position())
                    })?;
                    items.push(FlatUnary::Value(Unary::NumericLiteral(v, token.position())));
                }
                TokenType::StringLiteral(_) => {
                    let str = match token.string_literal() {
                        Ok((s, _t)) => s.into_owned(),
                        Err(err) => {
                            let err = match err {
                                StringLiteralError::NaT => CompileError::invalid_literal(
                                    "Invalid literal",
                                    token.position(),
                                ),
                                StringLiteralError::InvalidCharSequence(at) => {
                                    CompileError::invalid_literal(
                                        "Invalid character sequence",
                                        TokenPosition::new_at(token.position().start() + at),
                                    )
                                }
                                StringLiteralError::InvalidUnicodeChar(at) => {
                                    CompileError::invalid_literal(
                                        "Invalid unicode",
                                        TokenPosition::new_at(token.position().start() + at),
                                    )
                                }
                            };
                            return Err(err);
                        }
                    };
                    items.push(FlatUnary::Value(Unary::StringLiteral(
                        str,
                        token.position(),
                    )));
                }
                TokenType::Keyword(keyword) => {
                    if keyword.is_constant_value() {
                        items.push(FlatUnary::Value(Unary::Constant(
                            *keyword,
                            token.position(),
                        )));
                    } else {
                        return Err(CompileError::unexpected_token(&token));
                    }
                }
                TokenType::Eof => {
                    return Err(CompileError::missing_token(&token, allowed_ending));
                }
                _ => {
                    let position = token.position();
                    let op = UnaryOperator::parse_prefix(token, tokens)?;
                    let position = position.merged(&tokens.peek_last().unwrap().position());
                    let (trails, _) = Self::_parse_unary(tokens, allowed_ending, false)?;
                    items.extend(trails);
                    items.push(FlatUnary::Unary(op, position));
                }
            }
        }

        loop {
            if tokens.expect_symbol('.').is_ok() {
                let identifier = Identifier::from_tokens(tokens)?;
                items.push(FlatUnary::Member(identifier));
            } else if tokens.expect_symbol('[').is_ok() {
                let expr = Expression::parse(tokens, &[TokenType::Symbol(']')])?;
                expect_symbol(tokens, ']')?;
                items.push(FlatUnary::Subscript(expr));
            } else if tokens.expect_symbol('(').is_ok() {
                let mut params = Vec::new();
                loop {
                    if tokens.expect_symbol(')').is_ok() {
                        break;
                    }

                    let expr = Expression::parse(
                        tokens,
                        &[TokenType::Symbol(','), TokenType::Symbol(')')],
                    )?;
                    params.push(expr);

                    let _ = tokens.expect_symbol(',');
                }
                items.push(FlatUnary::Invoke(params.into_boxed_slice()));
            } else {
                break;
            }
        }

        if let Ok((op, position)) =
            tokens.transaction(|tokens| match UnaryOperator::parse_postfix(tokens) {
                Ok((op, position)) => ControlFlow::Continue((
                    op,
                    position.merged(&tokens.peek_last().unwrap().position()),
                )),
                Err(_) => ControlFlow::Break(()),
            })
        {
            items.push(FlatUnary::Unary(op, position));
        }

        if allow_binary {
            if let Ok((binop, position)) = tokens.transaction(|tokens| {
                if let Some(token) = tokens.shift() {
                    let position = token.position();
                    match BinaryOperator::parse(token, tokens) {
                        Ok(v) => ControlFlow::Continue((
                            v,
                            position.merged(&tokens.peek_last().unwrap().position()),
                        )),
                        Err(_) => ControlFlow::Break(()),
                    }
                } else {
                    ControlFlow::Break(())
                }
            }) {
                items.push(FlatUnary::Binary(binop, position));
                let (trails, _) = Self::_parse_unary(tokens, allowed_ending, true)?;
                items.extend(trails);
            }
        }

        let position_end = tokens.peek().unwrap().position().start();
        let token_position = TokenPosition::new(position_start as u32, position_end as u32);

        if !allowed_ending.contains(&TokenType::NewLine) {
            tokens.skip_ignorable();
        }
        if let Some(token) = tokens.peek() {
            if allowed_ending.contains(token.token_type()) {
                return Ok((items, token_position));
            }
        }
        if allow_binary {
            let token = tokens.peek().unwrap();
            if allowed_ending == Self::DEFAULT_ENDING {
                Err(CompileError::missing_eol(&token))
            } else {
                Err(CompileError::missing_token(&token, allowed_ending))
            }
        } else {
            Ok((items, token_position))
        }
    }

    fn finalize(src: Vec<FlatUnary>, base_position: TokenPosition) -> Result<Self, CompileError> {
        // Convert to Reverse Polish Notation
        let mut rpn_items = Vec::new();
        let mut operators = Vec::new();
        for item in src.into_iter() {
            match item {
                FlatUnary::Unary(op, _) => match op {
                    _ => rpn_items.push(item),
                },

                FlatUnary::Binary(op, _) => {
                    while let Some((pri, _)) = operators.last() {
                        if if op.is_right_associative() {
                            op.priority() < *pri
                        } else {
                            op.priority() <= *pri
                        } {
                            let (_, op) = operators.pop().unwrap();
                            rpn_items.push(op);
                            continue;
                        } else {
                            break;
                        }
                    }
                    operators.push((op.priority(), item));
                }

                _ => rpn_items.push(item),
            }
        }
        for (_pri, item) in operators.into_iter().rev() {
            rpn_items.push(item);
        }

        // conver to ast
        let mut items = Vec::new();

        for item in rpn_items {
            match item {
                FlatUnary::Void(position) => items.push(Unary::Void(position)),

                FlatUnary::Parenthesis(expr) => items.push(Unary::Parenthesis(expr)),

                FlatUnary::Value(value) => items.push(value),

                FlatUnary::Unary(op, position) => {
                    let value = items
                        .pop()
                        .ok_or(CompileError::out_of_value_stack(position))?;
                    items.push(Unary::Unary(op, position, Box::new(value)));
                }

                FlatUnary::Subscript(expr) => {
                    let value = items
                        .pop()
                        .ok_or(CompileError::out_of_value_stack(expr.position()))?;
                    items.push(Unary::Subscript(Box::new(value), expr));
                }

                FlatUnary::Member(identifier) => {
                    let value = items
                        .pop()
                        .ok_or(CompileError::out_of_value_stack(identifier.id_position()))?;
                    items.push(Unary::Member(Box::new(value), identifier));
                }

                FlatUnary::Binary(op, position) => {
                    let rhs = items
                        .pop()
                        .ok_or(CompileError::out_of_value_stack(position))?;
                    let lhs = items
                        .pop()
                        .ok_or(CompileError::out_of_value_stack(position))?;
                    items.push(Unary::Binary(op, position, Box::new(lhs), Box::new(rhs)));
                }

                FlatUnary::Invoke(params) => {
                    let position = Expression::reduce_positions(&params);
                    let caller = items
                        .pop()
                        .ok_or(CompileError::out_of_value_stack(position))?;
                    items.push(Unary::Invoke(Box::new(caller), params));
                }
            }
        }
        if items.len() != 1 {
            return Err(CompileError::internal_inconsistency(
                &format!("Expression value stack mismatch"),
                base_position.into(),
            ));
        }
        let item = Box::new(items[0].clone());

        Ok(Self {
            item,
            position: base_position,
        })
    }

    fn _find_first_rvalue(
        items: &mut Vec<FlatUnary>,
        index: usize,
        base_position: TokenPosition,
    ) -> Result<usize, CompileError> {
        let mut index = index;
        loop {
            let item = items
                .get_mut(index)
                .ok_or(CompileError::out_of_value_stack(base_position))?;

            match item {
                FlatUnary::Binary(_, _) => {
                    if index > 1 {
                        let index = Self::_find_first_rvalue(items, index - 1, base_position)?;
                        return Self::_find_first_rvalue(items, index - 1, base_position);
                    } else {
                        return Err(CompileError::out_of_value_stack(base_position));
                    }
                }

                FlatUnary::Subscript(_)
                | FlatUnary::Invoke(_)
                | FlatUnary::Member(_)
                | FlatUnary::Unary(_, _) => {
                    if index > 0 {
                        index -= 1;
                        continue;
                    } else {
                        return Err(CompileError::out_of_value_stack(base_position));
                    }
                }

                _ => return Ok(index),
            }
        }
    }

    #[inline]
    pub fn item(&self) -> &Unary {
        &self.item
    }

    #[inline]
    pub fn position(&self) -> TokenPosition {
        self.position
    }

    pub fn reduce_positions(slice: &[Self]) -> TokenPosition {
        let mut iter = slice.iter();
        if let Some(first) = iter.next() {
            iter.fold(first.position(), |a, b| a.merged(&b.position()))
        } else {
            TokenPosition::empty()
        }
    }
}

impl core::fmt::Debug for Expression {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Expression: {:#?}", self.item)
    }
}

impl core::fmt::Debug for Unary {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Void(_) => write!(f, "Void"),
            Self::Parenthesis(arg0) => f.debug_tuple("Parenthesis").field(arg0).finish(),
            Self::Subscript(arg0, arg1) => {
                f.debug_tuple("Subscript").field(arg0).field(arg1).finish()
            }
            Self::Invoke(arg0, arg1) => f.debug_tuple("Invoke").field(arg0).field(arg1).finish(),
            Self::Member(arg0, arg1) => f.debug_tuple("Member").field(arg0).field(arg1).finish(),
            Self::Constant(arg0, _) => write!(f, "{:#?}", arg0),
            Self::Unary(op, _, value) => f.debug_tuple(&format!("{:?}", op)).field(value).finish(),
            Self::NumericLiteral(arg0, _) => f.debug_tuple("NumericLiteral").field(arg0).finish(),
            Self::StringLiteral(arg0, _) => f.debug_tuple("StringLiteral").field(arg0).finish(),
            Self::Binary(op, _, lhs, rhs) => f
                .debug_tuple(&format!("{:?}", op))
                .field(lhs)
                .field(rhs)
                .finish(),
            Self::Identifier(identifier) => write!(f, "Identifier({:?})", identifier.as_str()),
        }
    }
}

impl Unary {
    pub fn position(&self) -> TokenPosition {
        match self {
            Unary::Void(position)
            | Unary::Constant(_, position)
            | Unary::Unary(_, position, _)
            | Unary::NumericLiteral(_, position)
            | Unary::StringLiteral(_, position)
            | Unary::Binary(_, position, _, _) => *position,

            Unary::Parenthesis(expr) | Unary::Subscript(_, expr) => expr.position(),

            Unary::Invoke(expr, args) => {
                if args.len() > 0 {
                    expr.position().merged(&Expression::reduce_positions(args))
                } else {
                    expr.position()
                }
            }

            Unary::Identifier(identifier) | Unary::Member(_, identifier) => {
                identifier.id_position()
            }
        }
    }

    pub fn reduce_positions(slice: &[Self]) -> Option<TokenPosition> {
        let mut iter = slice.iter();
        iter.next().map(|v| {
            let mut result = v.position();
            for item in iter {
                result = result.merged(&item.position());
            }
            result
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum UnaryOperator {
    /// "+"
    Plus,
    /// "-"
    Minus,
    /// "!"
    LogicalNot,
    /// "~"
    BitNot,

    /// "++A"
    PreIncrement,
    /// "--A"
    PreDecrement,

    /// "A++"
    PostIncrement,
    /// "A--"
    PostDecrement,

    /// "&A"
    Ref,
    /// "*A"
    Deref,
}

impl UnaryOperator {
    pub fn parse_prefix(
        leading: Token<Keyword>,
        tokens: &mut TokenStream<Keyword>,
    ) -> Result<Self, CompileError> {
        match leading.token_type() {
            TokenType::Symbol(symbol) => match symbol {
                '+' => {
                    if tokens.expect_immed_symbol('+').is_ok() {
                        return Ok(Self::PreIncrement);
                    } else {
                        return Ok(Self::Plus);
                    }
                }
                '-' => {
                    if tokens.expect_immed_symbol('-').is_ok() {
                        return Ok(Self::PreDecrement);
                    } else {
                        return Ok(Self::Minus);
                    }
                }

                '!' => return Ok(Self::LogicalNot),
                '~' => return Ok(Self::BitNot),

                '*' => return Ok(Self::Deref),
                '&' => return Ok(Self::Ref),
                _ => (),
            },
            _ => (),
        }
        Err(CompileError::unexpected_token(&leading))
    }

    pub fn parse_postfix(
        tokens: &mut TokenStream<Keyword>,
    ) -> Result<(Self, TokenPosition), CompileError> {
        let token = tokens.shift().unwrap();
        match token.token_type() {
            TokenType::Symbol(symbol) => match symbol {
                '+' => {
                    if tokens.expect_immed_symbol('+').is_ok() {
                        return Ok((Self::PostIncrement, token.position()));
                    }
                }
                '-' => {
                    if tokens.expect_immed_symbol('-').is_ok() {
                        return Ok((Self::PostDecrement, token.position()));
                    }
                }
                _ => (),
            },
            _ => (),
        }
        Err(CompileError::unexpected_token(&token))
    }

    pub fn priority(&self) -> OperatorPriority {
        match self {
            UnaryOperator::Plus => OperatorPriority::PrefixUnary,
            UnaryOperator::Minus => OperatorPriority::PrefixUnary,
            UnaryOperator::LogicalNot => OperatorPriority::PrefixUnary,
            UnaryOperator::BitNot => OperatorPriority::PrefixUnary,
            UnaryOperator::PreIncrement => OperatorPriority::PrefixUnary,
            UnaryOperator::PreDecrement => OperatorPriority::PrefixUnary,
            UnaryOperator::PostIncrement => OperatorPriority::PostfixUnary,
            UnaryOperator::PostDecrement => OperatorPriority::PostfixUnary,
            UnaryOperator::Ref => OperatorPriority::PrefixUnary,
            UnaryOperator::Deref => OperatorPriority::PrefixUnary,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum BinaryOperator {
    /// "="
    Assign,
    /// "+="
    AssignAdd,
    /// "-="
    AssignSub,
    /// "*="
    AssignMul,
    /// "/="
    AssignDiv,
    /// "%="
    AssignRem,
    /// "&="
    AssignBitAnd,
    /// "|="
    AssignBitOr,
    /// "^="
    AssignBitXor,
    /// "<<="
    AssignShl,
    /// ">>="
    AssignShr,

    /// "&&"
    LogicalAnd,
    /// "||"
    LogicalOr,
    /// "==="
    Identical,
    /// "!=="
    NotIdentical,

    /// "=="
    Eq,
    /// "!="
    Ne,
    /// "<"
    Lt,
    /// ">"
    Gt,
    /// "<="
    Le,
    /// ">="
    Ge,

    /// "+"
    Add,
    /// "-"
    Sub,
    /// "*"
    Mul,
    /// "/"
    Div,
    /// "%"
    Rem,

    /// "&"
    BitAnd,
    /// "|"
    BitOr,
    /// "^"
    BitXor,

    /// "<<"
    Shl,
    /// ">>"
    Shr,

    /// "**"
    Exponentiation,
}

impl BinaryOperator {
    pub fn parse(
        leading: Token<Keyword>,
        tokens: &mut TokenStream<Keyword>,
    ) -> Result<BinaryOperator, CompileError> {
        match leading.token_type() {
            TokenType::Symbol(symbol) => match symbol {
                '+' => {
                    if tokens.expect_immed_symbol('=').is_ok() {
                        return Ok(BinaryOperator::AssignAdd);
                    } else if tokens.expect_immed_symbol('+').is_err() {
                        return Ok(BinaryOperator::Add);
                    }
                }
                '-' => {
                    if tokens.expect_immed_symbol('=').is_ok() {
                        return Ok(BinaryOperator::AssignSub);
                    } else if tokens.expect_immed_symbol('-').is_err() {
                        return Ok(BinaryOperator::Sub);
                    }
                }
                '*' => {
                    if tokens.expect_immed_symbol('=').is_ok() {
                        return Ok(BinaryOperator::AssignMul);
                    } else if tokens.expect_immed_symbol('*').is_ok() {
                        return Ok(BinaryOperator::Exponentiation);
                    } else {
                        return Ok(BinaryOperator::Mul);
                    }
                }
                '/' => {
                    if tokens.expect_immed_symbol('=').is_ok() {
                        return Ok(BinaryOperator::AssignDiv);
                    } else {
                        return Ok(BinaryOperator::Div);
                    }
                }
                '%' => {
                    if tokens.expect_immed_symbol('=').is_ok() {
                        return Ok(BinaryOperator::AssignRem);
                    } else {
                        return Ok(BinaryOperator::Rem);
                    }
                }
                '!' => {
                    if tokens.expect_immed_symbol('=').is_ok() {
                        if tokens.expect_immed_symbol('=').is_ok() {
                            return Ok(BinaryOperator::NotIdentical);
                        } else {
                            return Ok(BinaryOperator::Ne);
                        }
                    }
                }
                '=' => {
                    if tokens.expect_immed_symbol('=').is_ok() {
                        if tokens.expect_immed_symbol('=').is_ok() {
                            return Ok(BinaryOperator::Identical);
                        } else {
                            return Ok(BinaryOperator::Eq);
                        }
                    } else {
                        return Ok(BinaryOperator::Assign);
                    }
                }
                '<' => {
                    if tokens.expect_immed_symbol('=').is_ok() {
                        return Ok(BinaryOperator::Le);
                    } else if tokens.expect_immed_symbol('<').is_ok() {
                        if tokens.expect_immed_symbol('=').is_ok() {
                            return Ok(BinaryOperator::AssignShl);
                        } else {
                            return Ok(BinaryOperator::Shl);
                        }
                    } else {
                        return Ok(BinaryOperator::Lt);
                    }
                }
                '>' => {
                    if tokens.expect_immed_symbol('=').is_ok() {
                        return Ok(BinaryOperator::Ge);
                    } else if tokens.expect_immed_symbol('>').is_ok() {
                        if tokens.expect_immed_symbol('=').is_ok() {
                            return Ok(BinaryOperator::AssignShr);
                        } else {
                            return Ok(BinaryOperator::Shr);
                        }
                    } else {
                        return Ok(BinaryOperator::Gt);
                    }
                }

                '&' => {
                    if tokens.expect_immed_symbol('=').is_ok() {
                        return Ok(BinaryOperator::AssignBitAnd);
                    } else if tokens.expect_immed_symbol('&').is_ok() {
                        return Ok(BinaryOperator::LogicalAnd);
                    } else {
                        return Ok(BinaryOperator::BitAnd);
                    }
                }
                '|' => {
                    if tokens.expect_immed_symbol('=').is_ok() {
                        return Ok(BinaryOperator::AssignBitOr);
                    } else if tokens.expect_immed_symbol('|').is_ok() {
                        return Ok(BinaryOperator::LogicalOr);
                    } else {
                        return Ok(BinaryOperator::BitOr);
                    }
                }
                '^' => {
                    if tokens.expect_immed_symbol('=').is_ok() {
                        return Ok(BinaryOperator::AssignBitXor);
                    } else {
                        return Ok(BinaryOperator::BitXor);
                    }
                }

                _ => (),
            },
            _ => (),
        }
        Err(CompileError::unexpected_token(&leading))
    }

    #[inline]
    pub fn assign_operator(&self) -> Option<BinaryOperator> {
        match self {
            BinaryOperator::Assign => Some(BinaryOperator::Assign),
            BinaryOperator::AssignAdd => Some(BinaryOperator::Add),
            BinaryOperator::AssignSub => Some(BinaryOperator::Sub),
            BinaryOperator::AssignMul => Some(BinaryOperator::Mul),
            BinaryOperator::AssignDiv => Some(BinaryOperator::Div),
            BinaryOperator::AssignRem => Some(BinaryOperator::Rem),
            BinaryOperator::AssignBitAnd => Some(BinaryOperator::BitAnd),
            BinaryOperator::AssignBitOr => Some(BinaryOperator::BitOr),
            BinaryOperator::AssignBitXor => Some(BinaryOperator::BitXor),
            BinaryOperator::AssignShl => Some(BinaryOperator::Shl),
            BinaryOperator::AssignShr => Some(BinaryOperator::Shr),
            _ => None,
        }
    }

    pub fn priority(&self) -> OperatorPriority {
        match self {
            BinaryOperator::Assign
            | BinaryOperator::AssignAdd
            | BinaryOperator::AssignSub
            | BinaryOperator::AssignMul
            | BinaryOperator::AssignDiv
            | BinaryOperator::AssignRem
            | BinaryOperator::AssignBitAnd
            | BinaryOperator::AssignBitOr
            | BinaryOperator::AssignBitXor
            | BinaryOperator::AssignShl
            | BinaryOperator::AssignShr => OperatorPriority::Assignment,

            BinaryOperator::LogicalAnd => OperatorPriority::LogicalAnd,

            BinaryOperator::LogicalOr => OperatorPriority::LogicalOr,

            BinaryOperator::Identical
            | BinaryOperator::NotIdentical
            | BinaryOperator::Eq
            | BinaryOperator::Ne => OperatorPriority::Equals,

            BinaryOperator::Lt | BinaryOperator::Gt | BinaryOperator::Le | BinaryOperator::Ge => {
                OperatorPriority::Compare
            }

            BinaryOperator::Add | BinaryOperator::Sub => OperatorPriority::AddSub,

            BinaryOperator::Mul | BinaryOperator::Div | BinaryOperator::Rem => {
                OperatorPriority::MulDiv
            }

            BinaryOperator::BitAnd => OperatorPriority::BitAnd,

            BinaryOperator::BitOr => OperatorPriority::BitOr,

            BinaryOperator::BitXor => OperatorPriority::BitXor,

            BinaryOperator::Shl | BinaryOperator::Shr => OperatorPriority::BitShift,

            BinaryOperator::Exponentiation => OperatorPriority::Exponentiation,
        }
    }

    pub fn is_right_associative(&self) -> bool {
        matches!(
            self,
            BinaryOperator::Assign
                | BinaryOperator::AssignAdd
                | BinaryOperator::AssignSub
                | BinaryOperator::AssignMul
                | BinaryOperator::AssignDiv
                | BinaryOperator::AssignRem
                | BinaryOperator::AssignBitAnd
                | BinaryOperator::AssignBitOr
                | BinaryOperator::AssignBitXor
                | BinaryOperator::AssignShl
                | BinaryOperator::AssignShr
                | BinaryOperator::Exponentiation
        )
    }

    pub fn to_ir(&self) -> tir::Op {
        match self {
            BinaryOperator::Identical => tir::Op::Eq,
            BinaryOperator::NotIdentical => tir::Op::Ne,
            BinaryOperator::Eq => tir::Op::Eq,
            BinaryOperator::Ne => tir::Op::Ne,
            BinaryOperator::Lt => tir::Op::Lt,
            BinaryOperator::Gt => tir::Op::Gt,
            BinaryOperator::Le => tir::Op::Le,
            BinaryOperator::Ge => tir::Op::Ge,

            BinaryOperator::Add => tir::Op::Add,
            BinaryOperator::Sub => tir::Op::Sub,
            BinaryOperator::Mul => tir::Op::Mul,
            BinaryOperator::Div => tir::Op::Div,
            BinaryOperator::Rem => tir::Op::Rem,
            BinaryOperator::BitAnd => tir::Op::And,
            BinaryOperator::BitOr => tir::Op::Or,
            BinaryOperator::BitXor => tir::Op::Xor,
            BinaryOperator::Shl => tir::Op::Shl,
            BinaryOperator::Shr => tir::Op::Shr,

            BinaryOperator::Assign
            | BinaryOperator::AssignAdd
            | BinaryOperator::AssignSub
            | BinaryOperator::AssignMul
            | BinaryOperator::AssignDiv
            | BinaryOperator::AssignRem
            | BinaryOperator::AssignBitAnd
            | BinaryOperator::AssignBitOr
            | BinaryOperator::AssignBitXor
            | BinaryOperator::AssignShl
            | BinaryOperator::AssignShr
            | BinaryOperator::LogicalAnd
            | BinaryOperator::LogicalOr
            | BinaryOperator::Exponentiation => unreachable!(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum OperatorPriority {
    Comma,
    Assignment,
    Ternary,
    LogicalOr,
    LogicalAnd,
    BitOr,
    BitXor,
    BitAnd,
    Equals,
    Compare,
    BitShift,
    AddSub,
    MulDiv,
    Exponentiation,
    PrefixUnary,
    PostfixUnary,
    New,
    Member,
    Group,
}
