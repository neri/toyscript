use super::scope::{Scope, VariableStorage};
use crate::*;
use ast::{
    block::Block,
    expression::{BinaryOperator, Expression, Unary, UnaryOperator},
    float::Float,
    integer::Integer,
    statement::{IfType, Statement},
    variable::VariableDeclaration,
};
use keyword::ModifierFlag;
use token::TokenPosition;
use toyir::{self, FuncTempIndex, Primitive};
use types::{
    function::{FunctionBody, FunctionDescriptor},
    var::VariableDescriptor,
    InferredType, ResolvePrimitive, TypeDescriptor,
};

pub struct FunctionGenerator;

impl FunctionGenerator {
    pub fn generate(
        func_desc: &FunctionDescriptor,
        body: &Block,
        types: &TypeSystem,
    ) -> Result<toyir::Function, CompileError> {
        let var_storage = VariableStorage::new(types);
        let mut scope = var_storage.root_scope();

        let signature = func_desc.signature();
        let exports =
            (func_desc.modifiers().contains(ModifierFlag::EXPORT)).then(|| func_desc.identifier());

        let return_type = func_desc.result_type().clone();

        let mut builder = toyir::Function::new(
            FuncTempIndex::new(func_desc.index().as_u32()),
            signature,
            exports,
            Some((
                return_type.identifier(),
                scope.types().storage_type(&return_type),
            )),
        )?;

        for (id, type_desc) in func_desc.params() {
            let mut var = VariableDescriptor::from_parameter(id, type_desc);
            let infered_type = var
                .inferred_type()
                .strict_type()
                .ok_or(CompileError::could_not_infer(var.identifier()))?;
            let primitive_type = scope.types().storage_type(infered_type);
            var.set_index(builder.declare_param(
                &var.identifier().to_string(),
                infered_type.identifier(),
                primitive_type,
            )?);
            scope.declare_local(var)?;
        }

        let block_type = Self::process_block(
            &mut builder.assembler(),
            body,
            &mut scope.scoped(None, None),
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
                    func_desc.position(),
                ));
            } else {
                return Err(CompileError::return_required(
                    &return_type,
                    func_desc.position(),
                ));
            }
        }

        for var_desc in var_storage.into_vars().iter().skip(func_desc.param_len()) {
            let type_desc = var_desc
                .inferred_type()
                .strict_type()
                .ok_or(CompileError::could_not_infer(var_desc.identifier()))?;
            builder.declare_local(
                var_desc.index(),
                &var_desc.identifier().to_string(),
                type_desc.identifier(),
                types.storage_type(type_desc),
                var_desc.is_mutable(),
            )?;
        }

        let function = builder.build().map_err(|err| {
            CompileError::internal_inconsistency(
                &format!("Internal Assembler Error: {:?}", err),
                ErrorPosition::Unspecified,
            )
        })?;

        Ok(function)
    }

    fn process_var(
        asm: &mut toyir::FunctionAssembler,
        scope: &mut Scope,
        var_decl: &VariableDeclaration,
    ) -> Result<(), CompileError> {
        for var_decl in var_decl.varibales() {
            let type_desc = match var_decl.type_decl() {
                Some(type_desc) => scope
                    .types()
                    .get(type_desc.identifier().as_str())
                    .ok_or(CompileError::identifier_not_found(type_desc.identifier()))
                    .and_then(|v| {
                        if v.is_special_type() {
                            Err(CompileError::invalid_type(type_desc.identifier()))
                        } else {
                            Ok(Some(v))
                        }
                    })?,
                None => None,
            };
            let mut var_desc = VariableDescriptor::from_var_decl(var_decl, type_desc.as_ref());
            var_desc.set_index(asm.alloc_local());
            let localidx = var_desc.index();

            let expected_type = var_desc.inferred_type().optimistic_type();
            if let Some(expr) = var_decl.assignment() {
                let expr_position = expr.position();
                let expr_type = Self::process_expression(asm, expr, expected_type, scope)?;

                if let Some(ref expr_type) = expr_type {
                    var_desc.infer(expr_type, expr_position)?;
                }

                scope.declare_local(var_desc)?;

                asm.ir_local_set(localidx)?;
            } else {
                if !var_desc.is_mutable() {
                    return Err(CompileError::must_assignment(
                        var_desc.identifier().as_str(),
                        var_decl.position(),
                    ));
                }
                scope.declare_local(var_desc)?;
            }
        }

        Ok(())
    }

    fn process_block(
        asm: &mut toyir::FunctionAssembler,
        block: &Block,
        scope: &mut Scope,
        return_type: &Arc<TypeDescriptor>,
    ) -> Result<Option<Arc<TypeDescriptor>>, CompileError> {
        let builtin_boolean = scope.types().builtin_boolean();

        let mut has_to_break = false;
        let mut block_type = None;
        for statement in block.statements().iter() {
            match statement {
                Statement::Eof(_) => break,

                Statement::Variable(var_decl) => {
                    Self::process_var(asm, scope, var_decl)?;
                }

                Statement::Block(block) => {
                    let mut scope = scope.scoped(None, None);
                    let child_block_type =
                        Self::process_block(asm, block, &mut scope, return_type)?;
                    let child_block_type = scope.types().canonical(child_block_type.as_ref());

                    if child_block_type.is_never() {
                        has_to_break = true;
                    }
                }

                Statement::IfStatement(if_types) => {
                    let else_exists = if_types.len() > 1;
                    let outer_block_index = asm.ir_block();
                    let mut block_indexes = Vec::new();
                    for _ in 1..if_types.len() {
                        let block_index = asm.ir_block();
                        block_indexes.push(block_index);
                    }

                    let mut has_else = false;
                    let mut may_break = true;
                    for if_type in if_types {
                        match if_type {
                            IfType::If(expr, block) => {
                                let mut scope = scope.scoped(None, None);

                                Self::process_expression(
                                    asm,
                                    expr,
                                    Some(&builtin_boolean),
                                    &mut scope,
                                )?;
                                asm.ir_invert()?;

                                let this_block = if else_exists {
                                    let this_block = block_indexes.pop().ok_or(
                                        CompileError::internal_inconsistency(
                                            &"broken if block",
                                            ErrorPosition::Unspecified,
                                        ),
                                    )?;
                                    asm.ir_br_if(this_block)?;
                                    Some(this_block)
                                } else {
                                    asm.ir_br_if(outer_block_index)?;
                                    None
                                };

                                let child_block_type =
                                    Self::process_block(asm, block, &mut scope, return_type)?;
                                let child_block_type =
                                    scope.types().canonical(child_block_type.as_ref());

                                if let Some(this_block) = this_block {
                                    asm.ir_br(outer_block_index)?;
                                    asm.ir_end(this_block)?;
                                }

                                if !child_block_type.is_never() {
                                    may_break = false;
                                }
                            }
                            IfType::ElseIf(expr, block) => {
                                let mut scope = scope.scoped(None, None);

                                Self::process_expression(
                                    asm,
                                    expr,
                                    Some(&builtin_boolean),
                                    &mut scope,
                                )?;
                                asm.ir_invert()?;

                                let this_block = block_indexes.pop().ok_or(
                                    CompileError::internal_inconsistency(
                                        &"broken if block",
                                        ErrorPosition::Unspecified,
                                    ),
                                )?;
                                asm.ir_br_if(this_block)?;

                                let child_block_type =
                                    Self::process_block(asm, block, &mut scope, return_type)?;
                                let child_block_type =
                                    scope.types().canonical(child_block_type.as_ref());
                                asm.ir_br(outer_block_index)?;
                                asm.ir_end(this_block)?;

                                if !child_block_type.is_never() {
                                    may_break = false;
                                }
                            }
                            IfType::Else(block) => {
                                has_else = true;
                                let mut scope = scope.scoped(None, None);
                                let child_block_type =
                                    Self::process_block(asm, block, &mut scope, return_type)?;
                                let child_block_type =
                                    scope.types().canonical(child_block_type.as_ref());

                                if !child_block_type.is_never() {
                                    may_break = false;
                                }
                            }
                        }
                    }

                    if may_break && has_else {
                        has_to_break = true;
                    }

                    asm.ir_end(outer_block_index)?;
                }

                Statement::WhileStatement(expr, block) => {
                    let break_index = asm.ir_block();
                    let loop_index = asm.ir_loop();
                    let mut scope = scope.scoped(Some(break_index), Some(loop_index));

                    Self::process_expression(asm, expr, Some(&builtin_boolean), &mut scope)?;
                    asm.ir_invert()?;
                    asm.ir_br_if(break_index)?;

                    let block_type = Self::process_block(asm, block, &mut scope, return_type)?;
                    let block_type = scope.types().canonical(block_type.as_ref());

                    if block_type.is_never() {
                        has_to_break = true;
                    } else {
                        asm.ir_br(loop_index)?;
                    }
                    asm.ir_end(loop_index)?;
                    asm.ir_end(break_index)?;
                }

                Statement::ForStatement(for_statement) => {
                    let mut scope = scope.scoped(None, None);
                    let break_index = asm.ir_block();

                    match for_statement.init {
                        ast::statement::ForInit::Var(ref var_decl) => {
                            Self::process_var(asm, &mut scope, var_decl)?;
                        }
                        ast::statement::ForInit::Expr(ref expr) => {
                            let expr_type = scope.types().canonical(
                                Self::process_expression(asm, expr, None, &scope)?.as_ref(),
                            );
                            if !expr_type.is_special_type() {
                                asm.ir_drop()?;
                            }
                        }
                    }

                    let loop_index = asm.ir_loop();
                    let continue_index = asm.ir_block();
                    let mut scope = scope.scoped(Some(break_index), Some(continue_index));

                    if for_statement.cond.is_empty() {
                        // for ever loop
                    } else {
                        let expr_type = scope.types().canonical(
                            Self::process_expression(
                                asm,
                                &for_statement.cond,
                                Some(&builtin_boolean),
                                &scope,
                            )?
                            .as_ref(),
                        );

                        if expr_type.is_never() {
                            // never
                        } else {
                            asm.ir_invert()?;
                            asm.ir_br_if(break_index)?;
                        }
                    }

                    let _block_type =
                        Self::process_block(asm, &for_statement.block, &mut scope, return_type)?;
                    // let block_type = scope.types().canonical(block_type.as_ref());

                    asm.ir_end(continue_index)?;

                    let expr_type = scope.types().canonical(
                        Self::process_expression(asm, &for_statement.step, None, &scope)?.as_ref(),
                    );
                    if !expr_type.is_special_type() {
                        asm.ir_drop()?;
                    }

                    asm.ir_br(loop_index)?;
                    asm.ir_end(loop_index)?;
                    asm.ir_end(break_index)?;
                }

                Statement::Expression(expr) => {
                    let expr_type = scope
                        .types()
                        .canonical(Self::process_expression(asm, expr, None, scope)?.as_ref());

                    if !expr_type.is_special_type() {
                        asm.ir_drop()?;
                    }
                    if expr_type.is_never() {
                        has_to_break = true;
                    }
                }

                Statement::ReturnStatement(expr, _) => {
                    let _expr_type = scope.types().canonical(
                        Self::process_expression(asm, expr, Some(&return_type), scope)?.as_ref(),
                    );

                    asm.ir_return()?;

                    has_to_break = true;
                }

                Statement::Break(position) => {
                    if let Some(target) = scope.break_index() {
                        asm.ir_br(target)?;
                        has_to_break = true;
                    } else {
                        return Err(CompileError::out_of_context("", *position));
                    }
                }

                Statement::Continue(position) => {
                    if let Some(target) = scope.continue_index() {
                        asm.ir_br(target)?;
                        has_to_break = true;
                    } else {
                        return Err(CompileError::out_of_context("", *position));
                    }
                }

                Statement::Enum(_)
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
        asm: &mut toyir::FunctionAssembler,
        expr: &Expression,
        expected_type: Option<&Arc<TypeDescriptor>>,
        scope: &Scope,
    ) -> Result<Option<Arc<TypeDescriptor>>, CompileError> {
        let (item, result_type) = Self::infer_expression(expr, expected_type, scope)?;
        Self::generate_unary(asm, &item, scope)
            .map(|_| result_type.optimistic_type().map(|v| v.clone()))
    }

    pub fn infer_expression(
        expr: &Expression,
        expected_type: Option<&Arc<TypeDescriptor>>,
        scope: &Scope,
    ) -> Result<(Unary, InferredType), CompileError> {
        let (item, result_type) = Self::infer_unary(
            &expected_type
                .map(|v| InferredType::Inferred(v.clone()))
                .unwrap_or(InferredType::Unknown),
            &expr.item(),
            scope,
        )?;
        let result_opt_type = result_type.optimistic_type().map(|v| v.clone());
        match (expected_type, result_opt_type) {
            (Some(a1), Some(a2)) => {
                if *a1 != a2 {
                    return Err(CompileError::type_mismatch(&a1, &a2, expr.position()));
                }
            }
            _ => {}
        }
        Ok((item, result_type))
    }

    fn infer_unary(
        inferred_to: &InferredType,
        item: &Unary,
        scope: &Scope,
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

            Unary::FloatingPointLiteral(value, position) => {
                let (value, inferred_to) =
                    scope.types().infer_float(value, &inferred_to, *position)?;

                Ok((Unary::FloatingPointLiteral(value, *position), inferred_to))
            }

            Unary::Parenthesis(ref expr) => {
                let (unary, inferred_type) = Self::infer_unary(inferred_to, expr.item(), scope)?;

                Ok((
                    Unary::Parenthesis(Expression::from_uanary(Box::new(unary.clone()))),
                    inferred_type,
                ))
            }

            Unary::TypeAssertion(ref value, ref type_desc, position) => {
                let target = scope
                    .types()
                    .get(&type_desc.identifier().to_string())
                    .map(|v| v.clone())
                    .ok_or(CompileError::invalid_type(type_desc.identifier()))?;

                let (value, src_type) = Self::infer_unary(&InferredType::Unknown, value, scope)?;

                scope
                    .types()
                    .try_convert_type(None, &src_type, &target, *position)?;

                Ok((
                    Unary::TypeAssertion(Box::new(value), type_desc.clone(), *position),
                    InferredType::Inferred(target),
                ))
            }

            Unary::Binary(op, position, ref lhs, ref rhs) => match op {
                BinaryOperator::Assign
                | BinaryOperator::AddAssign
                | BinaryOperator::SubAssign
                | BinaryOperator::MulAssign
                | BinaryOperator::DivAssign
                | BinaryOperator::RemAssign
                | BinaryOperator::BitAndAssign
                | BinaryOperator::BitOrAssign
                | BinaryOperator::BitXorAssign
                | BinaryOperator::ShlAssign
                | BinaryOperator::ShrAssign
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
                | BinaryOperator::Ge
                | BinaryOperator::Identical
                | BinaryOperator::NotIdentical => {
                    let cmp_type = scope.types().infer_as(
                        inferred_to,
                        &scope.types().builtin_boolean(),
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
                        &scope.types().builtin_boolean(),
                        position.merged(&lhs.position()).merged(&rhs.position()),
                    )?;
                    let (lhs, inferred_to) = Self::infer_unary(&inferred_to, lhs, scope)?;
                    let (rhs, inferred_to) = Self::infer_unary(&inferred_to, rhs, scope)?;

                    Ok((
                        Unary::Binary(*op, *position, Box::new(lhs), Box::new(rhs)),
                        inferred_to,
                    ))
                }

                BinaryOperator::Exponentiation => Err(CompileError::todo(None, (*position).into())),
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
                            Err(err) => Err(CompileError::literal_overflow(err.as_str(), position)),
                        }
                    }

                    _ => {
                        let (value, inferred_to) = Self::infer_unary(inferred_to, value, scope)?;
                        Ok((
                            Unary::Unary(*op, *position, Box::new(value)),
                            inferred_to.clone(),
                        ))
                    }
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
                    Err(CompileError::todo(None, (*position).into()))
                }
            },

            Unary::Invoke(callee, _args) => {
                let identifier = match callee.as_ref() {
                    Unary::Identifier(v) => Ok(v),
                    _ => Err(CompileError::todo(None, callee.position().into())),
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

            Unary::New(type_decl, _args, position) => {
                let type_desc = scope
                    .types()
                    .get(type_decl.identifier().as_str())
                    .ok_or(CompileError::identifier_not_found(type_decl.identifier()))?;

                let inferred_type = scope.types().infer_as(inferred_to, &type_desc, *position)?;

                Ok((item.clone(), inferred_type))
            }

            Unary::CharLiteral(_, _) => Ok((
                item.clone(),
                InferredType::Inferred(scope.types().builtin_char()),
            )),

            Unary::StringLiteral(_, _) => Ok((
                item.clone(),
                InferredType::Inferred(scope.types().builtin_string()),
            )),

            Unary::Subscript(_, _) | Unary::Member(_, _) | Unary::OptionalChain(_, _) => {
                // TODO:
                Err(CompileError::todo(None, item.position().into()))
            }

            Unary::Constant(keyword, position) => match keyword {
                Keyword::True | Keyword::False => {
                    let inferred = scope.types().infer_as(
                        inferred_to,
                        &scope.types().builtin_boolean(),
                        item.position(),
                    )?;

                    Ok((item.clone(), inferred))
                }

                Keyword::Null | Keyword::Undefined => Ok((item.clone(), InferredType::Unknown)),

                _ => Err(CompileError::todo(None, (*position).into())),
            },
        }
    }

    fn generate_unary(
        asm: &mut toyir::FunctionAssembler,
        item: &Unary,
        scope: &Scope,
    ) -> Result<InferredType, CompileError> {
        match item {
            Unary::Void(_) => Ok(InferredType::Inferred(scope.types().builtin_void())),

            Unary::Identifier(identifier) => {
                let localidx = scope
                    .resolve_local(identifier.as_str())
                    .ok_or(CompileError::identifier_not_found(&identifier))?;

                let var_desc = scope.get_desc_local(localidx);

                asm.ir_local_get(localidx);

                Ok(var_desc.inferred_type().clone())
            }

            Unary::Parenthesis(expr) => Self::generate_unary(asm, expr.item(), scope),

            Unary::TypeAssertion(ref value, ref type_desc, position) => {
                let target = scope
                    .types()
                    .get(&type_desc.identifier().to_string())
                    .map(|v| v.clone())
                    .ok_or(CompileError::invalid_type(type_desc.identifier()))?;

                let src_type = Self::generate_unary(asm, value, scope)?;

                scope
                    .types()
                    .try_convert_type(Some(asm), &src_type, &target, *position)?;

                Ok(InferredType::Inferred(target))
            }

            Unary::Unary(op, position, ref value) => match op {
                UnaryOperator::Plus => Self::generate_unary(asm, value, scope),

                UnaryOperator::Minus => asm.ir_neg(|asm| {
                    let result_type = Self::generate_unary(asm, value, scope)?;
                    let type2 = result_type
                        .optimistic_type()
                        .ok_or(CompileError::could_not_infer2(item.position()))?;
                    if type2.is_special_type() {
                        return Err(CompileError::invalid_type2(
                            type2.identifier(),
                            item.position(),
                        ));
                    }
                    let type2 = scope
                        .types()
                        .resolve_primitive(type2)
                        .ok_or(CompileError::could_not_infer2(item.position()))?;
                    Ok((type2, result_type))
                }),

                UnaryOperator::BitNot | UnaryOperator::LogicalNot => {
                    let result_type = Self::generate_unary(asm, value, scope)?;
                    let type2 = result_type
                        .optimistic_type()
                        .ok_or(CompileError::could_not_infer2(item.position()))?;
                    if type2.is_special_type() {
                        return Err(CompileError::invalid_type2(
                            type2.identifier(),
                            item.position(),
                        ));
                    } else if type2.is_boolean() {
                        asm.ir_invert()?;
                    } else {
                        asm.emit_unop(toyir::Op::Not)?;
                    }
                    Ok(result_type)
                }
                UnaryOperator::PreIncrement | UnaryOperator::PreDecrement => {
                    let identifier = match **value {
                        Unary::Identifier(ref v) => Ok(v),
                        _ => Err(CompileError::lvalue_required(value.position())),
                    }?;
                    let localidx = scope
                        .resolve_local(identifier.as_str())
                        .ok_or(CompileError::identifier_not_found(identifier))?;
                    let var_desc = scope.get_desc_local(localidx);
                    if !var_desc.is_mutable() {
                        return Err(CompileError::cannot_assign(identifier));
                    }

                    asm.ir_local_get(localidx);

                    match op {
                        UnaryOperator::PreIncrement => asm.emit_unop(toyir::Op::Inc)?,
                        UnaryOperator::PreDecrement => asm.emit_unop(toyir::Op::Dec)?,
                        _ => unreachable!(),
                    }

                    asm.ir_local_tee(localidx)?;

                    Ok(var_desc.inferred_type().clone())
                }

                UnaryOperator::PostIncrement | UnaryOperator::PostDecrement => {
                    let identifier = match **value {
                        Unary::Identifier(ref v) => Ok(v),
                        _ => Err(CompileError::lvalue_required(value.position())),
                    }?;
                    let localidx = scope
                        .resolve_local(identifier.as_str())
                        .ok_or(CompileError::identifier_not_found(identifier))?;
                    let var_desc = scope.get_desc_local(localidx);
                    if !var_desc.is_mutable() {
                        return Err(CompileError::cannot_assign(identifier));
                    }

                    asm.ir_local_get(localidx);
                    asm.ir_local_get(localidx);

                    match op {
                        UnaryOperator::PostIncrement => asm.emit_unop(toyir::Op::Inc)?,
                        UnaryOperator::PostDecrement => asm.emit_unop(toyir::Op::Dec)?,
                        _ => unreachable!(),
                    }

                    asm.ir_local_set(localidx)?;

                    Ok(var_desc.inferred_type().clone())
                }

                // UnaryOperator::Ref | UnaryOperator::Deref => {
                //     return Err(CompileError::todo(None, *position))
                // }
                #[allow(dead_code)]
                _ => return Err(CompileError::todo(None, (*position).into())),
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
                    let lhs_type = Self::generate_unary_canonical(asm, lhs, scope)?;
                    let _rhs_type = Self::generate_unary_canonical(asm, rhs, scope)?;
                    let is_signed = lhs_type
                        .strict_type()
                        .and_then(|v| scope.types().resolve_primitive(v))
                        .map(|v| v.is_signed())
                        .unwrap_or(false);
                    asm.emit_binop(op.to_ir(is_signed))?;
                    Ok(InferredType::Inferred(scope.types().builtin_boolean()))
                }

                BinaryOperator::Add
                | BinaryOperator::Sub
                | BinaryOperator::Mul
                | BinaryOperator::BitAnd
                | BinaryOperator::BitOr
                | BinaryOperator::BitXor => {
                    let lhs_type = Self::generate_unary(asm, lhs, scope)?;
                    let _rhs_type = Self::generate_unary(asm, rhs, scope)?;
                    let is_signed = lhs_type
                        .strict_type()
                        .and_then(|v| scope.types().resolve_primitive(v))
                        .map(|v| v.is_signed())
                        .unwrap_or(false);
                    asm.emit_binop(op.to_ir(is_signed))?;
                    Ok(lhs_type)
                }

                BinaryOperator::Div | BinaryOperator::Rem => {
                    let lhs_type = Self::generate_unary_canonical(asm, lhs, scope)?;
                    let _rhs_type = Self::generate_unary_canonical(asm, rhs, scope)?;
                    let is_signed = lhs_type
                        .strict_type()
                        .and_then(|v| scope.types().resolve_primitive(v))
                        .map(|v| v.is_signed())
                        .unwrap_or(false);
                    asm.emit_binop(op.to_ir(is_signed))?;
                    Ok(lhs_type)
                }

                BinaryOperator::Shl => {
                    let lhs_type = Self::generate_unary(asm, lhs, scope)?;
                    let _rhs_type =
                        Self::generate_unary_canonical_for_shift(asm, rhs, scope, &lhs_type)?;
                    let is_signed = lhs_type
                        .strict_type()
                        .and_then(|v| scope.types().resolve_primitive(v))
                        .map(|v| v.is_signed())
                        .unwrap_or(false);
                    asm.emit_binop(op.to_ir(is_signed))?;
                    Ok(lhs_type)
                }

                BinaryOperator::Shr => {
                    let lhs_type = Self::generate_unary_canonical(asm, lhs, scope)?;
                    let _rhs_type =
                        Self::generate_unary_canonical_for_shift(asm, rhs, scope, &lhs_type)?;
                    let is_signed = lhs_type
                        .strict_type()
                        .and_then(|v| scope.types().resolve_primitive(v))
                        .map(|v| v.is_signed())
                        .unwrap_or(false);
                    asm.emit_binop(op.to_ir(is_signed))?;
                    Ok(lhs_type)
                }

                BinaryOperator::Assign
                | BinaryOperator::AddAssign
                | BinaryOperator::SubAssign
                | BinaryOperator::MulAssign
                | BinaryOperator::DivAssign
                | BinaryOperator::RemAssign
                | BinaryOperator::BitAndAssign
                | BinaryOperator::BitOrAssign
                | BinaryOperator::BitXorAssign
                | BinaryOperator::ShlAssign
                | BinaryOperator::ShrAssign => {
                    let identifier = match **lhs {
                        Unary::Identifier(ref v) => Ok(v),
                        _ => Err(CompileError::lvalue_required(lhs.position())),
                    }?;
                    let localidx = scope
                        .resolve_local(identifier.as_str())
                        .ok_or(CompileError::identifier_not_found(identifier))?;
                    let var_desc = scope.get_desc_local(localidx);
                    if !var_desc.is_mutable() {
                        return Err(CompileError::cannot_assign(identifier));
                    }
                    if let Some(inferred_type) = var_desc.inferred_type().optimistic_type() {
                        if inferred_type.is_special_type() {
                            return Err(CompileError::invalid_type2(
                                inferred_type.identifier(),
                                rhs.position(),
                            ));
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
                        asm.ir_local_get(localidx);
                    }

                    if matches!(
                        assign_operator,
                        BinaryOperator::Div | BinaryOperator::Rem | BinaryOperator::Shr
                    ) {
                        Self::generate_unary_canonical(asm, rhs, scope)?;
                    } else {
                        Self::generate_unary(asm, rhs, scope)?;
                    }

                    if !matches!(assign_operator, BinaryOperator::Assign) {
                        let is_signed = var_desc
                            .inferred_type()
                            .strict_type()
                            .and_then(|v| scope.types().resolve_primitive(v))
                            .map(|v| v.is_signed())
                            .unwrap_or(false);
                        asm.emit_binop(assign_operator.to_ir(is_signed))?;
                    }
                    asm.ir_local_tee(localidx)?;

                    Ok(var_desc.inferred_type().clone())
                }

                BinaryOperator::LogicalAnd | BinaryOperator::LogicalOr => {
                    return Err(CompileError::todo(None, item.position().into()))
                }

                BinaryOperator::Exponentiation => {
                    return Err(CompileError::todo(None, item.position().into()))
                }
            },

            Unary::Constant(constant, _position) => match constant {
                Keyword::True => {
                    asm.ir_bool_const(true);
                    Ok(InferredType::Inferred(scope.types().builtin_boolean()))
                }
                Keyword::False => {
                    asm.ir_bool_const(false);
                    Ok(InferredType::Inferred(scope.types().builtin_boolean()))
                }

                Keyword::Null | Keyword::Undefined => {
                    asm.ir_i32_const(0);
                    Ok(InferredType::Unknown)
                }

                _ => {
                    return Err(CompileError::todo(
                        Some(format!("The constant '{:?}' is not supported", constant)),
                        item.position().into(),
                    ))
                }
            },
            Unary::NumericLiteral(number, _position) => match number {
                Integer::I8(v) => {
                    asm.ir_i32_const(*v as i32);
                    Ok(InferredType::Inferred(
                        scope.types().primitive_type(Primitive::I8),
                    ))
                }
                Integer::U8(v) => {
                    asm.ir_i32_const(*v as i32);
                    Ok(InferredType::Inferred(
                        scope.types().primitive_type(Primitive::U8),
                    ))
                }
                Integer::I16(v) => {
                    asm.ir_i32_const(*v as i32);
                    Ok(InferredType::Inferred(
                        scope.types().primitive_type(Primitive::I16),
                    ))
                }
                Integer::U16(v) => {
                    asm.ir_i32_const(*v as i32);
                    Ok(InferredType::Inferred(
                        scope.types().primitive_type(Primitive::U16),
                    ))
                }
                Integer::I32(v) => {
                    asm.ir_i32_const(*v);
                    Ok(InferredType::Inferred(
                        scope.types().primitive_type(Primitive::I32),
                    ))
                }
                Integer::U32(v) => {
                    asm.ir_i32_const(*v as i32);
                    Ok(InferredType::Inferred(
                        scope.types().primitive_type(Primitive::U32),
                    ))
                }
                Integer::I64(v) => {
                    asm.ir_i64_const(*v);
                    Ok(InferredType::Inferred(
                        scope.types().primitive_type(Primitive::I64),
                    ))
                }
                Integer::U64(v) => {
                    asm.ir_i64_const(*v as i64);
                    Ok(InferredType::Inferred(
                        scope.types().primitive_type(Primitive::U64),
                    ))
                }
            },
            Unary::FloatingPointLiteral(number, _position) => match number {
                Float::F32(v) => {
                    asm.ir_f32_const(*v);
                    Ok(InferredType::Inferred(
                        scope.types().primitive_type(Primitive::F32),
                    ))
                }
                Float::F64(v) => {
                    asm.ir_f64_const(*v);
                    Ok(InferredType::Inferred(
                        scope.types().primitive_type(Primitive::F64),
                    ))
                }
            },
            Unary::CharLiteral(c, _position) => {
                asm.ir_i32_const(*c as i32);
                Ok(InferredType::Inferred(scope.types().builtin_char()))
            }
            Unary::StringLiteral(_, _position) => {
                // TODO:
                return Err(CompileError::todo(None, item.position().into()));
            }

            Unary::Invoke(callee, args) => {
                let identifier = match callee.as_ref() {
                    Unary::Identifier(v) => Ok(v),
                    _ => Err(CompileError::todo(None, callee.position().into())),
                }?;

                let func_desc = scope
                    .types()
                    .function(identifier.as_str())
                    .ok_or(CompileError::identifier_not_found(&identifier))?;

                if func_desc.param_len() != args.len() {
                    return Err(CompileError::function_parameter_number_mismatch(
                        func_desc.param_len(),
                        args.len(),
                        identifier.id_position(),
                    ));
                }

                for ((_id, parameter), expr) in func_desc.params().iter().zip(args.iter()) {
                    Self::process_expression(asm, expr, Some(parameter), scope)?;
                }

                match func_desc.body() {
                    FunctionBody::Block(_) => {
                        asm.ir_call(
                            func_desc.index().as_usize(),
                            func_desc.param_len(),
                            if func_desc.result_type().is_special_type() {
                                0
                            } else {
                                1
                            },
                        )?;
                    }
                    FunctionBody::Inline(emitter) => {
                        emitter(asm)?;
                    }
                }

                if func_desc.result_type().is_never() {
                    asm.ir_unreachable();
                }

                Ok(InferredType::Inferred(func_desc.result_type().clone()))
            }

            Unary::Subscript(u, expr) => {
                // TODO:
                return Err(CompileError::todo(
                    None,
                    u.position().merged(&expr.position()).into(),
                ));
            }
            Unary::Member(_u, identifier) => {
                // TODO:
                return Err(CompileError::todo(None, identifier.id_position().into()));
            }
            Unary::OptionalChain(_u, identifier) => {
                // TODO:
                return Err(CompileError::todo(None, identifier.id_position().into()));
            }
            Unary::New(_c, _p, position) => {
                // TODO:
                return Err(CompileError::todo(None, (*position).into()));
            }
        }
    }

    fn generate_unary_canonical(
        asm: &mut toyir::FunctionAssembler,
        item: &Unary,
        scope: &Scope,
    ) -> Result<InferredType, CompileError> {
        let result_type = Self::generate_unary(asm, item, scope)?;

        let Some(inferred_type) = result_type.optimistic_type() else {
            return Ok(result_type);
        };
        let Some(primitive) = scope.types().resolve_primitive(inferred_type) else {
            return Ok(result_type);
        };

        match primitive {
            Primitive::I8 | Primitive::U8 | Primitive::I16 | Primitive::U16 => {
                asm.ir_cast(Primitive::I32.type_id(), primitive.type_id())?;
            }
            _ => {}
        }
        Ok(result_type)
    }

    fn generate_unary_canonical_for_shift(
        asm: &mut toyir::FunctionAssembler,
        item: &Unary,
        scope: &Scope,
        lhs_type: &InferredType,
    ) -> Result<InferredType, CompileError> {
        let result_type = Self::generate_unary(asm, item, scope)?;

        let _ = lhs_type;

        // TODO: TBD

        Ok(result_type)
    }
}
