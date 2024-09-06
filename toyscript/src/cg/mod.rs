//! ToyScript Code Generator

pub mod scope;

use crate::*;
use ast::{
    block::Block,
    expression::{BinaryOperator, Expression, Unary, UnaryOperator},
    integer::Integer,
    statement::{IfType, Statement},
};
use keyword::ModifierFlag;
use scope::{VariableDescriptor, VariableScope, VariableStorage};
use tir;
use token::TokenPosition;
use types::{InferredType, Primitive, Resolve, TypeDescriptor};

pub struct CodeGen;

impl CodeGen {
    pub fn generate(ast: &Ast, types: &TypeSystem) -> Result<tir::Module, CompileError> {
        let mut module = tir::Module::new(types.name());

        for statement in ast.program() {
            match statement {
                Statement::Function(func_decl) => {
                    let func_desc = types.function(func_decl.identifier().as_str()).ok_or(
                        CompileError::internal_inconsistency(
                            &format!("Function Declaration"),
                            func_decl.identifier().id_position().into(),
                        ),
                    )?;
                    if func_decl.modifiers().contains(ModifierFlag::IMPORT) {
                        continue;
                    } else {
                        let function = Self::generate_function(func_decl, func_desc, types)?;
                        module.add_function(function);
                    }
                }
                Statement::Eof(_) | Statement::TypeAlias(_, _) => (),

                Statement::Block(_)
                | Statement::Class(_)
                | Statement::Expression(_)
                | Statement::IfStatement(_)
                | Statement::ReturnStatement(_)
                | Statement::Variable(_)
                | Statement::WhileStatement(_, _)
                | Statement::Enum(_)
                | Statement::ForStatement(_, _) => {
                    return Err(CompileError::out_of_context(
                        format!("{:#?}", statement).as_str(),
                        TokenPosition::empty(),
                    ))
                }
            }
        }

        Ok(module)
    }

    fn generate_function(
        func_decl: &ast::function::FunctionDeclaration,
        func_desc: &types::function::FunctionDescriptor,
        types: &TypeSystem,
    ) -> Result<tir::Function, CompileError> {
        let var_storage = VariableStorage::new(types);
        let mut scope = var_storage.root_scope();

        for (param, type_desc) in func_decl.parameters().iter().zip(func_desc.param_types()) {
            let var = VariableDescriptor::from_parameter(param, type_desc);
            scope.declare_local(var)?;
        }

        let return_type = func_desc.result_type().clone();

        let codes = if func_decl.modifiers().contains(ModifierFlag::IMPORT) {
            tir::Assembler::new()
        } else {
            let mut codes = tir::Assembler::new();
            let block_type = Self::process_block(
                &mut codes.stream(),
                func_decl.body(),
                &mut scope.scoped(),
                &return_type,
            )?;

            if !match block_type {
                Some(ref v) => v.is_never() || *v == return_type,
                None => return_type.is_void(),
            } {
                if block_type.is_some() {
                    return Err(CompileError::type_mismatch(
                        &return_type,
                        &block_type.unwrap_or(types.builtin_void()),
                        func_decl.position(),
                    ));
                } else {
                    return Err(CompileError::return_required(
                        &return_type,
                        func_decl.position(),
                    ));
                }
            }

            codes.finalize()?
        };

        let signature = func_desc.signature();
        let exports = (func_desc.modifiers().contains(ModifierFlag::EXPORT))
            .then(|| func_desc.identifier().as_str());

        let mut params = Vec::new();
        for type_desc in func_desc.param_types() {
            params.push(
                types
                    .resolve_primitive(type_desc)
                    .unwrap_or(Primitive::Void),
            );
        }

        let mut locals = Vec::new();
        let vars = var_storage.into_vars();
        let vars = vars.iter().skip(params.len());
        for var in vars {
            let type_desc = var
                .optimistic_inferred_type()
                .ok_or(CompileError::could_not_infer(var.identifier()))?;
            locals.push(
                types
                    .resolve_primitive(type_desc)
                    .unwrap_or(Primitive::Void),
            );
        }

        let function = tir::Function::new(
            signature,
            exports,
            params.as_slice(),
            &[types.resolve_primitive(return_type.identifier()).unwrap()],
            locals.as_slice(),
            codes,
        );

        Ok(function)
    }

    fn process_block(
        codes: &mut tir::CodeStream,
        block: &Block,
        scope: &mut VariableScope,
        return_type: &Arc<TypeDescriptor>,
    ) -> Result<Option<Arc<TypeDescriptor>>, CompileError> {
        let builtin_bool = scope.types().builtin_bool();

        let mut has_to_break = false;
        let mut block_type = None;
        for statement in block.statements() {
            match statement {
                Statement::Eof(_) => break,

                Statement::Variable(var_decl) => {
                    for var_decl in var_decl.varibales() {
                        let type_desc = match var_decl.type_desc() {
                            Some(type_desc) => scope
                                .types()
                                .get(type_desc.identifier().as_str())
                                .ok_or(CompileError::identifier_not_found(type_desc.identifier()))
                                .and_then(|v| {
                                    if v.is_special() {
                                        Err(CompileError::invalid_type(type_desc.identifier()))
                                    } else {
                                        Ok(Some(v))
                                    }
                                })?,
                            None => None,
                        };
                        let mut var_desc = VariableDescriptor::from_var_decl(var_decl, type_desc);
                        let expected_type = var_desc.optimistic_inferred_type();
                        if let Some(expr) = var_decl.assignment() {
                            let expr_position = expr.position();
                            let expr_type =
                                Self::process_expression(codes, expr, expected_type, scope)?;

                            if let Some(ref expr_type) = expr_type {
                                var_desc.infer(expr_type, expr_position)?;
                            }
                            let var_idx = scope.declare_local(var_desc)?;

                            codes.ir_local_set(var_idx)?;
                        } else {
                            scope.declare_local(var_desc)?;
                        }
                    }
                }

                Statement::Block(block) => {
                    let mut scope = scope.scoped();
                    let child_block_type =
                        Self::process_block(codes, block, &mut scope, return_type)?;

                    if child_block_type.map(|v| v.is_never()).unwrap_or(false) {
                        has_to_break = true;
                    }
                }

                Statement::IfStatement(if_types) => {
                    let else_exists = if_types.len() > 1;
                    let outer_block_index = codes.ir_block();
                    let mut block_indexes = Vec::new();
                    for _ in 1..if_types.len() {
                        let block_index = codes.ir_block();
                        block_indexes.push(block_index);
                    }

                    let mut has_else = false;
                    let mut may_break = true;
                    for if_type in if_types {
                        match if_type {
                            IfType::If(expr, block) => {
                                let mut scope = scope.scoped();

                                Self::process_expression(
                                    codes,
                                    expr,
                                    Some(&builtin_bool),
                                    &mut scope,
                                )?;
                                codes.ir_invert()?;

                                let this_block = if else_exists {
                                    let this_block = block_indexes.pop().ok_or(
                                        CompileError::internal_inconsistency(
                                            &"broken if block",
                                            ErrorPosition::Unspecified,
                                        ),
                                    )?;
                                    codes.ir_br_if(this_block)?;
                                    Some(this_block)
                                } else {
                                    codes.ir_br_if(outer_block_index)?;
                                    None
                                };

                                let child_block_type =
                                    Self::process_block(codes, block, &mut scope, return_type)?;

                                if let Some(this_block) = this_block {
                                    codes.ir_br(outer_block_index)?;
                                    codes.ir_end(this_block)?;
                                }

                                if !child_block_type.map(|v| v.is_never()).unwrap_or(false) {
                                    may_break = false;
                                }
                            }
                            IfType::ElseIf(expr, block) => {
                                let mut scope = scope.scoped();

                                Self::process_expression(
                                    codes,
                                    expr,
                                    Some(&builtin_bool),
                                    &mut scope,
                                )?;
                                codes.ir_invert()?;

                                let this_block = block_indexes.pop().ok_or(
                                    CompileError::internal_inconsistency(
                                        &"broken if block",
                                        ErrorPosition::Unspecified,
                                    ),
                                )?;
                                codes.ir_br_if(this_block)?;

                                let child_block_type =
                                    Self::process_block(codes, block, &mut scope, return_type)?;
                                codes.ir_br(outer_block_index)?;
                                codes.ir_end(this_block)?;

                                if !child_block_type.map(|v| v.is_never()).unwrap_or(false) {
                                    may_break = false;
                                }
                            }
                            IfType::Else(block) => {
                                has_else = true;
                                let mut scope = scope.scoped();
                                let child_block_type =
                                    Self::process_block(codes, block, &mut scope, return_type)?;

                                if !child_block_type.map(|v| v.is_never()).unwrap_or(false) {
                                    may_break = false;
                                }
                            }
                        }
                    }

                    if may_break && has_else {
                        has_to_break = true;
                    }

                    codes.ir_end(outer_block_index)?;
                }

                Statement::WhileStatement(expr, block) => {
                    let break_index = codes.ir_block();
                    let loop_index = codes.ir_loop();
                    let mut scope = scope.scoped();

                    Self::process_expression(codes, expr, Some(&builtin_bool), &mut scope)?;
                    codes.ir_invert()?;
                    codes.ir_br_if(break_index)?;

                    let block_type = Self::process_block(codes, block, &mut scope, return_type)?;

                    if block_type.map(|v| v.is_never()).unwrap_or(false) {
                        has_to_break = true;
                    } else {
                        codes.ir_br(loop_index)?;
                    }
                    codes.ir_end(loop_index)?;
                    codes.ir_end(break_index)?;
                }

                Statement::Expression(expr) => {
                    let expr_type = scope
                        .types()
                        .canonical_opt(Self::process_expression(codes, expr, None, scope)?);

                    if expr_type.as_ref().map(|v| v.is_special()).unwrap_or(true) {
                    } else {
                        codes.ir_drop()?;
                    }
                    if expr_type.map(|v| v.is_never()).unwrap_or(false) {
                        has_to_break = true;
                    }
                }

                Statement::ReturnStatement(expr) => {
                    let expr_type =
                        Self::process_expression(codes, expr, Some(&return_type), scope)?
                            .unwrap_or(scope.types().builtin_void());

                    codes.ir_return(if expr_type.is_special() { 0 } else { 1 })?;

                    has_to_break = true;
                }

                Statement::ForStatement(_, _)
                | Statement::Enum(_)
                | Statement::TypeAlias(_, _)
                | Statement::Function(_)
                | &Statement::Class(_) => {
                    return Err(CompileError::out_of_context(
                        format!("{:#?}", statement).as_str(),
                        TokenPosition::empty(),
                    ))
                }
            }

            if has_to_break {
                block_type = Some(scope.types().builtin_never());
                break;
            }
        }

        Ok(block_type)
    }

    fn process_expression(
        codes: &mut tir::CodeStream,
        expr: &Expression,
        expected_type: Option<&Arc<TypeDescriptor>>,
        scope: &VariableScope,
    ) -> Result<Option<Arc<TypeDescriptor>>, CompileError> {
        let (item, result_type) = Self::infer_unary(
            &expected_type
                .map(|v| InferredType::Inferred(v.clone()))
                .unwrap_or(InferredType::Unknown),
            &expr.item(),
            scope,
        )?;
        Self::generate_unary(codes, &item, scope)
            .map(|_| result_type.optimistic_type().map(|v| v.clone()))
    }

    fn infer_unary(
        inferred_to: &InferredType,
        item: &Unary,
        scope: &VariableScope,
    ) -> Result<(Unary, InferredType), CompileError> {
        match item {
            Unary::Void(_) => {
                let inferred = scope.types().infer_as(
                    inferred_to,
                    &scope.types().builtin_void(),
                    item.position(),
                )?;

                Ok((item.clone(), inferred))
            }

            Unary::Identifier(ref identifier) => {
                let var_idx = scope
                    .resolve_local(identifier.as_str())
                    .ok_or(CompileError::identifier_not_found(identifier))?;
                let inferred_type =
                    scope.infer_local(var_idx, inferred_to, identifier.id_position())?;

                Ok((item.clone(), inferred_type))
            }

            Unary::NumericLiteral(value, position) => {
                let (value, inferred_to) =
                    scope
                        .types()
                        .infer_integer(value, &inferred_to, *position)?;

                Ok((Unary::NumericLiteral(value, *position), inferred_to))
            }

            Unary::Parenthesis(ref expr) => {
                let (unary, inferred_type) = Self::infer_unary(inferred_to, expr.item(), scope)?;

                Ok((
                    Unary::Parenthesis(Expression::from_uanary(Box::new(unary.clone()))),
                    inferred_type,
                ))
            }

            Unary::Binary(op, position, ref lhs, ref rhs) => match op {
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
                | BinaryOperator::Identical
                | BinaryOperator::NotIdentical
                | BinaryOperator::Add
                | BinaryOperator::Sub
                | BinaryOperator::Mul
                | BinaryOperator::Div
                | BinaryOperator::Rem
                | BinaryOperator::BitAnd
                | BinaryOperator::BitOr
                | BinaryOperator::BitXor
                | BinaryOperator::Shl
                | BinaryOperator::Shr => {
                    let (lhs, inferred_to) = Self::infer_unary(inferred_to, lhs, scope)?;
                    let (rhs, inferred_to) = Self::infer_unary(&inferred_to, rhs, scope)?;
                    let (lhs, inferred_to) = Self::infer_unary(&inferred_to, &lhs, scope)?;

                    Ok((
                        Unary::Binary(*op, *position, Box::new(lhs), Box::new(rhs)),
                        inferred_to,
                    ))
                }

                BinaryOperator::Eq
                | BinaryOperator::Ne
                | BinaryOperator::Lt
                | BinaryOperator::Gt
                | BinaryOperator::Le
                | BinaryOperator::Ge => {
                    let cmp_type = scope.types().infer_as(
                        inferred_to,
                        &scope.types().builtin_bool(),
                        position.merged(&lhs.position()).merged(&rhs.position()),
                    )?;
                    let (lhs, lhs_type) = Self::infer_unary(&InferredType::Unknown, lhs, scope)?;
                    let (rhs, rhs_type) = Self::infer_unary(&lhs_type, rhs, scope)?;
                    let (lhs, _lhs_type) = Self::infer_unary(&rhs_type, &lhs, scope)?;

                    Ok((
                        Unary::Binary(*op, *position, Box::new(lhs), Box::new(rhs)),
                        cmp_type,
                    ))
                }

                BinaryOperator::LogicalAnd | BinaryOperator::LogicalOr => {
                    let inferred_to = scope.types().infer_as(
                        inferred_to,
                        &scope.types().builtin_bool(),
                        position.merged(&lhs.position()).merged(&rhs.position()),
                    )?;
                    let (lhs, inferred_to) = Self::infer_unary(&inferred_to, lhs, scope)?;
                    let (rhs, inferred_to) = Self::infer_unary(&inferred_to, rhs, scope)?;

                    Ok((
                        Unary::Binary(*op, *position, Box::new(lhs), Box::new(rhs)),
                        inferred_to,
                    ))
                }

                BinaryOperator::Exponentiation => Err(CompileError::todo(None, *position)),
            },

            Unary::Unary(op, position, ref value) => match op {
                UnaryOperator::Plus | UnaryOperator::Minus => match **value {
                    Unary::NumericLiteral(integer, int_position) => {
                        let position = position.merged(&int_position);
                        match integer.int_to_signed(matches!(op, UnaryOperator::Minus)) {
                            Ok(value) => scope
                                .types()
                                .infer_integer(&value, inferred_to, position)
                                .map(|(value, inferred_to)| {
                                    (Unary::NumericLiteral(value, position), inferred_to)
                                }),
                            Err(err) => Err(CompileError::literal_overflow(
                                &scope.types().primitive_type(err),
                                position,
                            )),
                        }
                    }

                    _ => Ok((item.clone(), inferred_to.clone())),
                },

                UnaryOperator::LogicalNot
                | UnaryOperator::BitNot
                | UnaryOperator::PreIncrement
                | UnaryOperator::PreDecrement
                | UnaryOperator::PostIncrement
                | UnaryOperator::PostDecrement => {
                    let (value, inferred_to) = Self::infer_unary(inferred_to, value, scope)?;

                    Ok((
                        Unary::Unary(*op, *position, Box::new(value)),
                        inferred_to.clone(),
                    ))
                }

                UnaryOperator::Ref | UnaryOperator::Deref => {
                    Err(CompileError::todo(None, *position))
                }
            },

            Unary::Invoke(callee, _args) => {
                let identifier = match **callee {
                    Unary::Identifier(ref v) => Ok(v),
                    _ => Err(CompileError::todo(None, callee.position())),
                }?;

                let func_desc = scope
                    .types()
                    .function(identifier.as_str())
                    .ok_or(CompileError::identifier_not_found(&identifier))?;

                let inferred_type = scope.types().infer_as(
                    inferred_to,
                    func_desc.result_type(),
                    callee.position(),
                )?;

                Ok((item.clone(), inferred_type))
            }

            Unary::Subscript(_, _) | Unary::Member(_, _) | Unary::StringLiteral(_, _) => {
                Err(CompileError::todo(None, item.position()))
            }

            Unary::Constant(keyword, position) => match keyword {
                Keyword::True | &Keyword::False => {
                    let inferred = scope.types().infer_as(
                        inferred_to,
                        &scope.types().builtin_bool(),
                        item.position(),
                    )?;

                    Ok((item.clone(), inferred))
                }
                _ => Err(CompileError::todo(None, *position)),
            },
        }
    }

    fn generate_unary(
        codes: &mut tir::CodeStream,
        item: &Unary,
        scope: &VariableScope,
    ) -> Result<(), CompileError> {
        match item {
            Unary::Void(_) => (),

            Unary::Identifier(identifier) => {
                let var_idx = scope
                    .resolve_local(identifier.as_str())
                    .ok_or(CompileError::identifier_not_found(&identifier))?;
                codes.ir_local_get(var_idx);
            }

            Unary::Parenthesis(expr) => {
                Self::generate_unary(codes, expr.item(), scope)?;
            }

            Unary::Unary(op, position, ref value) => match op {
                UnaryOperator::Plus => {
                    Self::generate_unary(codes, value, scope)?;
                }

                UnaryOperator::Minus => {
                    Self::generate_unary(codes, value, scope)?;
                    codes.ir_neg()?;
                }

                UnaryOperator::BitNot => {
                    Self::generate_unary(codes, value, scope)?;
                    codes.ir_not()?;
                }

                UnaryOperator::LogicalNot => {
                    Self::generate_unary(codes, value, scope)?;
                    codes.ir_invert()?;
                }

                UnaryOperator::PreIncrement | UnaryOperator::PreDecrement => {
                    let identifier = match **value {
                        Unary::Identifier(ref v) => Ok(v),
                        _ => Err(CompileError::lvalue_required(value.position())),
                    }?;
                    let var_idx = scope
                        .resolve_local(identifier.as_str())
                        .ok_or(CompileError::identifier_not_found(identifier))?;
                    let var_desc = scope.get_desc_local(var_idx);
                    if !var_desc.is_mutable() {
                        return Err(CompileError::cannot_assign(identifier));
                    }

                    codes.ir_local_get(var_idx);

                    match op {
                        UnaryOperator::PreIncrement => codes.ir_inc()?,
                        UnaryOperator::PreDecrement => codes.ir_dec()?,
                        _ => unreachable!(),
                    }

                    codes.ir_local_tee(var_idx)?;
                }

                UnaryOperator::PostIncrement | UnaryOperator::PostDecrement => {
                    let identifier = match **value {
                        Unary::Identifier(ref v) => Ok(v),
                        _ => Err(CompileError::lvalue_required(value.position())),
                    }?;
                    let var_idx = scope
                        .resolve_local(identifier.as_str())
                        .ok_or(CompileError::identifier_not_found(identifier))?;
                    let var_desc = scope.get_desc_local(var_idx);
                    if !var_desc.is_mutable() {
                        return Err(CompileError::cannot_assign(identifier));
                    }

                    codes.ir_local_get(var_idx);
                    codes.ir_local_get(var_idx);

                    match op {
                        UnaryOperator::PostIncrement => codes.ir_inc()?,
                        UnaryOperator::PostDecrement => codes.ir_dec()?,
                        _ => unreachable!(),
                    }

                    codes.ir_local_set(var_idx)?;
                }

                // UnaryOperator::Ref | UnaryOperator::Deref => {
                //     return Err(CompileError::todo(None, *position))
                // }
                #[allow(dead_code)]
                _ => return Err(CompileError::todo(None, *position)),
            },

            Unary::Binary(op, _position, ref lhs, ref rhs) => match op {
                BinaryOperator::Eq
                | BinaryOperator::Ne
                | BinaryOperator::Lt
                | BinaryOperator::Gt
                | BinaryOperator::Le
                | BinaryOperator::Ge
                | BinaryOperator::Identical
                | BinaryOperator::NotIdentical => {
                    Self::generate_unary(codes, lhs, scope)?;
                    Self::generate_unary(codes, rhs, scope)?;
                    codes.emit_binop(op.to_ir())?;
                }

                BinaryOperator::Add
                | BinaryOperator::Sub
                | BinaryOperator::Mul
                | BinaryOperator::Div
                | BinaryOperator::Rem
                | BinaryOperator::BitAnd
                | BinaryOperator::BitOr
                | BinaryOperator::BitXor
                | BinaryOperator::Shl
                | BinaryOperator::Shr => {
                    Self::generate_unary(codes, lhs, scope)?;
                    Self::generate_unary(codes, rhs, scope)?;
                    codes.emit_binop(op.to_ir())?;
                }

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
                | BinaryOperator::AssignShr => {
                    let identifier = match **lhs {
                        Unary::Identifier(ref v) => Ok(v),
                        _ => Err(CompileError::lvalue_required(lhs.position())),
                    }?;
                    let var_idx = scope
                        .resolve_local(identifier.as_str())
                        .ok_or(CompileError::identifier_not_found(identifier))?;
                    let var_desc = scope.get_desc_local(var_idx);
                    if !var_desc.is_mutable() {
                        return Err(CompileError::cannot_assign(identifier));
                    }
                    if let Some(inferred_type) = var_desc.inferred_type().optimistic_type() {
                        if inferred_type.is_special() {
                            return Err(CompileError::invalid_type(&Identifier::new(
                                inferred_type.identifier(),
                                rhs.position(),
                            )));
                        }
                    } else {
                        return Err(CompileError::could_not_infer(identifier));
                    }

                    let Some(assign_operator) = op.assign_operator() else {
                        return Err(CompileError::internal_inconsistency(
                            "Invalid Operator",
                            item.position().into(),
                        ));
                    };

                    if !matches!(assign_operator, BinaryOperator::Assign) {
                        codes.ir_local_get(var_idx);
                    }

                    Self::generate_unary(codes, rhs, scope)?;

                    if !matches!(assign_operator, BinaryOperator::Assign) {
                        codes.emit_binop(assign_operator.to_ir())?;
                    }
                    codes.ir_local_tee(var_idx)?;
                }

                BinaryOperator::LogicalAnd | BinaryOperator::LogicalOr => {
                    return Err(CompileError::todo(None, item.position()))
                }

                BinaryOperator::Exponentiation => {
                    return Err(CompileError::todo(None, item.position()))
                }
            },

            Unary::Constant(constant, _position) => {
                match constant {
                    Keyword::True => codes.ir_bool_const(true),
                    Keyword::False => codes.ir_bool_const(false),
                    _ => {
                        return Err(CompileError::todo(
                            Some(format!("The constant '{:?}' is not supported", constant)),
                            item.position(),
                        ))
                    }
                };
            }

            Unary::NumericLiteral(number, _position) => match number {
                Integer::I8(v) => codes.ir_i32_const(*v as i32),
                Integer::U8(v) => codes.ir_i32_const(*v as i32),
                Integer::I16(v) => codes.ir_i32_const(*v as i32),
                Integer::U16(v) => codes.ir_i32_const(*v as i32),
                Integer::I32(v) => codes.ir_i32_const(*v),
                Integer::U32(v) => codes.ir_i32_const(*v as i32),
                Integer::I64(v) => codes.ir_i64_const(*v),
                Integer::U64(v) => codes.ir_i64_const(*v as i64),
            },

            Unary::StringLiteral(_, _position) => {
                // TODO:
                return Err(CompileError::todo(None, item.position()));
            }
            Unary::Subscript(u, expr) => {
                // TODO:
                return Err(CompileError::todo(
                    None,
                    u.position().merged(&expr.position()),
                ));
            }
            Unary::Member(_u, identifier) => {
                // TODO:
                return Err(CompileError::todo(None, identifier.id_position()));
            }

            Unary::Invoke(ref callee, args) => {
                let identifier = match **callee {
                    Unary::Identifier(ref v) => Ok(v),
                    _ => Err(CompileError::todo(None, callee.position())),
                }?;

                let func_desc = scope
                    .types()
                    .function(identifier.as_str())
                    .ok_or(CompileError::identifier_not_found(&identifier))?;

                if func_desc.param_types().len() != args.len() {
                    return Err(CompileError::function_parameter_number_mismatch(
                        func_desc.param_types().len(),
                        args.len(),
                        identifier.id_position(),
                    ));
                }

                for (parameter, expr) in func_desc.param_types().iter().zip(args.iter()) {
                    Self::process_expression(codes, expr, Some(parameter), scope)?;
                }

                codes.ir_call(
                    func_desc.function_index().as_usize(),
                    func_desc.param_types().len(),
                    if func_desc.result_type().is_special() {
                        0
                    } else {
                        1
                    },
                )?;
                if func_desc.result_type().is_never() {
                    codes.ir_unreachable();
                }
            }
        }

        Ok(())
    }
}
