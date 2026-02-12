use gbasic_common::ast::*;
use gbasic_common::error::GBasicError;
use gbasic_common::types::Type;
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::targets::{
    CodeModel, FileType, InitializationConfig, RelocMode, Target, TargetMachine,
};
use inkwell::types::BasicMetadataTypeEnum;
use inkwell::values::{BasicMetadataValueEnum, BasicValueEnum, FunctionValue, PointerValue};
use inkwell::OptimizationLevel;
use std::collections::HashMap;
use std::path::Path;
use std::process::Command;

/// Variable info: alloca pointer + type
struct VarInfo<'ctx> {
    ptr: PointerValue<'ctx>,
    ty: Type,
}

pub struct Codegen<'ctx> {
    context: &'ctx Context,
    module: Module<'ctx>,
    builder: Builder<'ctx>,
    variables: Vec<HashMap<String, VarInfo<'ctx>>>,
    current_function: Option<FunctionValue<'ctx>>,
}

impl<'ctx> Codegen<'ctx> {
    pub fn new(context: &'ctx Context) -> Self {
        let module = context.create_module("gbasic");
        let builder = context.create_builder();
        Self {
            context,
            module,
            builder,
            variables: vec![HashMap::new()],
            current_function: None,
        }
    }

    fn push_scope(&mut self) {
        self.variables.push(HashMap::new());
    }

    fn pop_scope(&mut self) {
        self.variables.pop();
    }

    fn insert_var(&mut self, name: String, info: VarInfo<'ctx>) {
        if let Some(scope) = self.variables.last_mut() {
            scope.insert(name, info);
        }
    }

    fn lookup_var(&self, name: &str) -> Option<&VarInfo<'ctx>> {
        for scope in self.variables.iter().rev() {
            if let Some(v) = scope.get(name) {
                return Some(v);
            }
        }
        None
    }

    fn declare_runtime_functions(&self) {
        let i8_ptr_type = self.context.ptr_type(inkwell::AddressSpace::default());
        let i64_type = self.context.i64_type();
        let f64_type = self.context.f64_type();
        let void_type = self.context.void_type();

        // runtime_print(s: *const i8)
        let print_ty = void_type.fn_type(&[i8_ptr_type.into()], false);
        self.module.add_function("runtime_print", print_ty, None);

        // runtime_print_int(v: i64)
        let print_int_ty = void_type.fn_type(&[i64_type.into()], false);
        self.module
            .add_function("runtime_print_int", print_int_ty, None);

        // runtime_print_float(v: f64)
        let print_float_ty = void_type.fn_type(&[f64_type.into()], false);
        self.module
            .add_function("runtime_print_float", print_float_ty, None);

        // No-newline variants for string interpolation
        let part_str_ty = void_type.fn_type(&[i8_ptr_type.into()], false);
        self.module.add_function("runtime_print_str_part", part_str_ty, None);

        let part_int_ty = void_type.fn_type(&[i64_type.into()], false);
        self.module.add_function("runtime_print_int_part", part_int_ty, None);

        let part_float_ty = void_type.fn_type(&[f64_type.into()], false);
        self.module.add_function("runtime_print_float_part", part_float_ty, None);

        let newline_ty = void_type.fn_type(&[], false);
        self.module.add_function("runtime_print_newline", newline_ty, None);
    }

    pub fn compile(
        context: &'ctx Context,
        program: &Program,
        output_path: &str,
        dump_ir: bool,
    ) -> Result<(), GBasicError> {
        let mut cg = Codegen::new(context);
        cg.declare_runtime_functions();

        // First pass: declare all top-level functions
        for stmt in &program.statements {
            if let Statement::Function(func) = stmt {
                cg.declare_function(func)?;
            }
        }

        // Build main function wrapping top-level statements
        let i32_type = cg.context.i32_type();
        let main_fn_type = i32_type.fn_type(&[], false);
        let main_fn = cg.module.add_function("main", main_fn_type, None);
        let entry = cg.context.append_basic_block(main_fn, "entry");
        cg.builder.position_at_end(entry);
        cg.current_function = Some(main_fn);

        for stmt in &program.statements {
            match stmt {
                Statement::Function(func) => {
                    // Will be codegen'd separately after main
                    cg.codegen_function_body(func)?;
                }
                _ => {
                    cg.codegen_statement(stmt)?;
                }
            }
        }

        // Return 0 from main (only if no terminator yet)
        if cg
            .builder
            .get_insert_block()
            .unwrap()
            .get_terminator()
            .is_none()
        {
            cg.builder
                .build_return(Some(&i32_type.const_int(0, false)))
                .unwrap();
        }

        // Verify
        cg.module
            .verify()
            .map_err(|e| GBasicError::CodegenError {
                message: format!("LLVM verification failed: {}", e.to_string()),
            })?;

        if dump_ir {
            cg.module.print_to_stderr();
            return Ok(());
        }

        // Emit and link
        cg.emit_and_link(output_path)?;
        Ok(())
    }

    fn declare_function(&mut self, func: &FunctionDecl) -> Result<(), GBasicError> {
        let ret_type = func.return_type.clone().unwrap_or(Type::Void);
        let param_types: Vec<BasicMetadataTypeEnum> = func
            .params
            .iter()
            .map(|p| self.type_to_llvm_meta(&p.type_ann.clone().unwrap_or(Type::Unknown)))
            .collect();

        let fn_type = match &ret_type {
            Type::Void => self.context.void_type().fn_type(&param_types, false),
            Type::Int => self.context.i64_type().fn_type(&param_types, false),
            Type::Float => self.context.f64_type().fn_type(&param_types, false),
            Type::Bool => self.context.bool_type().fn_type(&param_types, false),
            _ => self.context.i64_type().fn_type(&param_types, false),
        };

        self.module.add_function(&func.name.name, fn_type, None);
        Ok(())
    }

    fn codegen_function_body(&mut self, func: &FunctionDecl) -> Result<(), GBasicError> {
        let function = self
            .module
            .get_function(&func.name.name)
            .ok_or_else(|| GBasicError::CodegenError {
                message: format!("function '{}' not declared", func.name.name),
            })?;

        // Save current state
        let prev_fn = self.current_function;
        let prev_block = self.builder.get_insert_block();

        let entry = self.context.append_basic_block(function, "entry");
        self.builder.position_at_end(entry);
        self.current_function = Some(function);
        self.push_scope();

        // Alloca params
        for (i, param) in func.params.iter().enumerate() {
            let param_val = function.get_nth_param(i as u32).unwrap();
            let ty = param.type_ann.clone().unwrap_or(Type::Unknown);
            let alloca = self.build_alloca_for_type(&ty, &param.name.name);
            self.builder.build_store(alloca, param_val).unwrap();
            self.insert_var(
                param.name.name.clone(),
                VarInfo { ptr: alloca, ty },
            );
        }

        let stmts = &func.body.statements;
        let ret_type = func.return_type.clone().unwrap_or(Type::Void);

        // Codegen all statements except possibly the last (which may be implicit return)
        let last_is_expr = matches!(stmts.last(), Some(Statement::Expression { .. }))
            && !matches!(ret_type, Type::Void);

        let count = if last_is_expr { stmts.len() - 1 } else { stmts.len() };
        for stmt in &stmts[..count] {
            self.codegen_statement(stmt)?;
        }

        // If the last statement is an expression in a non-void function, use it as implicit return
        if last_is_expr {
            if let Some(Statement::Expression { expr, .. }) = stmts.last() {
                if self.builder.get_insert_block().unwrap().get_terminator().is_none() {
                    let val = self.codegen_expression(expr)?;
                    if let Some(v) = val {
                        self.builder.build_return(Some(&v)).unwrap();
                    } else {
                        self.builder.build_return(None).unwrap();
                    }
                }
            }
        }

        // Add implicit return if still needed
        if self
            .builder
            .get_insert_block()
            .unwrap()
            .get_terminator()
            .is_none()
        {
            match ret_type {
                Type::Void => {
                    self.builder.build_return(None).unwrap();
                }
                Type::Int => {
                    self.builder
                        .build_return(Some(&self.context.i64_type().const_int(0, false)))
                        .unwrap();
                }
                Type::Float => {
                    self.builder
                        .build_return(Some(&self.context.f64_type().const_float(0.0)))
                        .unwrap();
                }
                Type::Bool => {
                    self.builder
                        .build_return(Some(&self.context.bool_type().const_int(0, false)))
                        .unwrap();
                }
                _ => {
                    self.builder.build_return(None).unwrap();
                }
            }
        }

        self.pop_scope();
        self.current_function = prev_fn;
        if let Some(bb) = prev_block {
            self.builder.position_at_end(bb);
        }

        Ok(())
    }

    fn codegen_statement(&mut self, stmt: &Statement) -> Result<(), GBasicError> {
        match stmt {
            Statement::Let { name, value, .. } => {
                let val = self.codegen_expression(value)?;
                let ty = self.infer_expr_type(value);
                match val {
                    Some(v) => {
                        let alloca = self.build_alloca_for_type(&ty, &name.name);
                        self.builder.build_store(alloca, v).unwrap();
                        self.insert_var(name.name.clone(), VarInfo { ptr: alloca, ty });
                    }
                    None => {} // void expression in let — skip
                }
            }
            Statement::Expression { expr, .. } => {
                self.codegen_expression(expr)?;
            }
            Statement::Return { value, .. } => {
                if let Some(val_expr) = value {
                    let val = self.codegen_expression(val_expr)?;
                    match val {
                        Some(v) => {
                            self.builder.build_return(Some(&v)).unwrap();
                        }
                        None => {
                            self.builder.build_return(None).unwrap();
                        }
                    }
                } else {
                    self.builder.build_return(None).unwrap();
                }
            }
            Statement::If {
                condition,
                then_block,
                else_block,
                ..
            } => {
                let cond = self.codegen_expression(condition)?.unwrap().into_int_value();
                let function = self.current_function.unwrap();
                let then_bb = self.context.append_basic_block(function, "then");
                let else_bb = self.context.append_basic_block(function, "else");
                let merge_bb = self.context.append_basic_block(function, "merge");

                self.builder
                    .build_conditional_branch(cond, then_bb, else_bb)
                    .unwrap();

                // then
                self.builder.position_at_end(then_bb);
                self.push_scope();
                for s in &then_block.statements {
                    self.codegen_statement(s)?;
                }
                self.pop_scope();
                if self
                    .builder
                    .get_insert_block()
                    .unwrap()
                    .get_terminator()
                    .is_none()
                {
                    self.builder.build_unconditional_branch(merge_bb).unwrap();
                }

                // else
                self.builder.position_at_end(else_bb);
                if let Some(eb) = else_block {
                    self.push_scope();
                    for s in &eb.statements {
                        self.codegen_statement(s)?;
                    }
                    self.pop_scope();
                }
                if self
                    .builder
                    .get_insert_block()
                    .unwrap()
                    .get_terminator()
                    .is_none()
                {
                    self.builder.build_unconditional_branch(merge_bb).unwrap();
                }

                self.builder.position_at_end(merge_bb);
            }
            Statement::While {
                condition, body, ..
            } => {
                let function = self.current_function.unwrap();
                let cond_bb = self.context.append_basic_block(function, "while_cond");
                let body_bb = self.context.append_basic_block(function, "while_body");
                let exit_bb = self.context.append_basic_block(function, "while_exit");

                self.builder.build_unconditional_branch(cond_bb).unwrap();
                self.builder.position_at_end(cond_bb);
                let cond = self.codegen_expression(condition)?.unwrap().into_int_value();
                self.builder
                    .build_conditional_branch(cond, body_bb, exit_bb)
                    .unwrap();

                self.builder.position_at_end(body_bb);
                self.push_scope();
                for s in &body.statements {
                    self.codegen_statement(s)?;
                }
                self.pop_scope();
                if self
                    .builder
                    .get_insert_block()
                    .unwrap()
                    .get_terminator()
                    .is_none()
                {
                    self.builder.build_unconditional_branch(cond_bb).unwrap();
                }

                self.builder.position_at_end(exit_bb);
            }
            Statement::Block(block) => {
                self.push_scope();
                for s in &block.statements {
                    self.codegen_statement(s)?;
                }
                self.pop_scope();
            }
            Statement::Function(_) => {
                // Already handled in top-level pass
            }
            _ => {
                // For/Match/Break/Continue — not yet implemented
            }
        }
        Ok(())
    }

    fn codegen_expression(
        &mut self,
        expr: &Expression,
    ) -> Result<Option<BasicValueEnum<'ctx>>, GBasicError> {
        match expr {
            Expression::Literal(lit) => match &lit.kind {
                LiteralKind::Int(v) => Ok(Some(
                    self.context
                        .i64_type()
                        .const_int(*v as u64, true)
                        .into(),
                )),
                LiteralKind::Float(v) => Ok(Some(
                    self.context.f64_type().const_float(*v).into(),
                )),
                LiteralKind::Bool(v) => Ok(Some(
                    self.context
                        .bool_type()
                        .const_int(if *v { 1 } else { 0 }, false)
                        .into(),
                )),
                LiteralKind::String(s) => {
                    let global = self.builder.build_global_string_ptr(s, "str").unwrap();
                    Ok(Some(global.as_pointer_value().into()))
                }
            },
            Expression::Identifier(id) => {
                let var = self.lookup_var(&id.name).ok_or_else(|| {
                    GBasicError::CodegenError {
                        message: format!("undefined variable '{}'", id.name),
                    }
                })?;
                let llvm_type = self.type_to_llvm_basic(&var.ty);
                let ptr = var.ptr;
                let val = self.builder.build_load(llvm_type, ptr, &id.name).unwrap();
                Ok(Some(val))
            }
            Expression::BinaryOp {
                left, op, right, ..
            } => {
                let lv = self.codegen_expression(left)?.unwrap();
                let rv = self.codegen_expression(right)?.unwrap();
                let left_ty = self.infer_expr_type(left);

                let result = match left_ty {
                    Type::Int => self.codegen_int_binop(lv.into_int_value(), op, rv.into_int_value()),
                    Type::Float => {
                        self.codegen_float_binop(lv.into_float_value(), op, rv.into_float_value())
                    }
                    Type::Bool => {
                        self.codegen_int_binop(lv.into_int_value(), op, rv.into_int_value())
                    }
                    _ => Err(GBasicError::CodegenError {
                        message: format!("unsupported binary op on {left_ty}"),
                    }),
                }?;
                Ok(Some(result))
            }
            Expression::UnaryOp { op, operand, .. } => {
                let val = self.codegen_expression(operand)?.unwrap();
                let ty = self.infer_expr_type(operand);
                match op {
                    UnaryOp::Neg => match ty {
                        Type::Int => Ok(Some(
                            self.builder
                                .build_int_neg(val.into_int_value(), "neg")
                                .unwrap()
                                .into(),
                        )),
                        Type::Float => Ok(Some(
                            self.builder
                                .build_float_neg(val.into_float_value(), "neg")
                                .unwrap()
                                .into(),
                        )),
                        _ => Err(GBasicError::CodegenError {
                            message: "cannot negate non-numeric".into(),
                        }),
                    },
                    UnaryOp::Not => Ok(Some(
                        self.builder
                            .build_not(val.into_int_value(), "not")
                            .unwrap()
                            .into(),
                    )),
                }
            }
            Expression::Call { callee, args, .. } => {
                self.codegen_call(callee, args)
            }
            Expression::Assignment { target, value, .. } => {
                let val = self.codegen_expression(value)?.unwrap();
                if let Expression::Identifier(id) = target.as_ref() {
                    let var = self.lookup_var(&id.name).ok_or_else(|| {
                        GBasicError::CodegenError {
                            message: format!("undefined variable '{}'", id.name),
                        }
                    })?;
                    let ptr = var.ptr;
                    self.builder.build_store(ptr, val).unwrap();
                }
                Ok(Some(val))
            }
            Expression::StringInterp { parts, .. } => {
                // For standalone string interp (not inside print), emit parts and return empty string
                // This is a simplified approach — full concat would need a runtime allocator
                for part in parts {
                    match part {
                        StringPart::Lit(s) => {
                            let global = self.builder.build_global_string_ptr(s, "str_part").unwrap();
                            let print_fn = self.module.get_function("runtime_print_str_part").unwrap();
                            self.builder.build_call(print_fn, &[global.as_pointer_value().into()], "").unwrap();
                        }
                        StringPart::Expr(e) => {
                            self.emit_print_expr_part(e)?;
                        }
                    }
                }
                let newline_fn = self.module.get_function("runtime_print_newline").unwrap();
                self.builder.build_call(newline_fn, &[], "").unwrap();
                let empty = self.builder.build_global_string_ptr("", "empty").unwrap();
                Ok(Some(empty.as_pointer_value().into()))
            }
            Expression::MethodChain { .. }
            | Expression::Array { .. }
            | Expression::Index { .. }
            | Expression::FieldAccess { .. } => {
                // Not yet implemented — return null pointer
                let null = self.context.ptr_type(inkwell::AddressSpace::default()).const_null();
                Ok(Some(null.into()))
            }
        }
    }

    fn codegen_call(
        &mut self,
        callee: &Expression,
        args: &[Expression],
    ) -> Result<Option<BasicValueEnum<'ctx>>, GBasicError> {
        // Special-case: print builtin
        if let Expression::Identifier(id) = callee {
            if id.name == "print" && args.len() == 1 {
                return self.codegen_print(&args[0]);
            }

            // Regular function call
            let function = self
                .module
                .get_function(&id.name)
                .ok_or_else(|| GBasicError::CodegenError {
                    message: format!("undefined function '{}'", id.name),
                })?;

            let mut compiled_args: Vec<BasicMetadataValueEnum> = Vec::new();
            for arg in args {
                let val = self.codegen_expression(arg)?.unwrap();
                compiled_args.push(val.into());
            }

            let call = self
                .builder
                .build_call(function, &compiled_args, "call")
                .unwrap();

            Ok(call.try_as_basic_value().left())
        } else {
            Err(GBasicError::CodegenError {
                message: "only direct function calls supported".into(),
            })
        }
    }

    /// Emit a single expression as a print part (no newline)
    fn emit_print_expr_part(
        &mut self,
        expr: &Expression,
    ) -> Result<(), GBasicError> {
        let ty = self.infer_expr_type(expr);
        let val = self.codegen_expression(expr)?;
        match ty {
            Type::String | Type::Unknown => {
                let f = self.module.get_function("runtime_print_str_part").unwrap();
                self.builder.build_call(f, &[val.unwrap().into()], "").unwrap();
            }
            Type::Int => {
                let f = self.module.get_function("runtime_print_int_part").unwrap();
                self.builder.build_call(f, &[val.unwrap().into()], "").unwrap();
            }
            Type::Float => {
                let f = self.module.get_function("runtime_print_float_part").unwrap();
                self.builder.build_call(f, &[val.unwrap().into()], "").unwrap();
            }
            Type::Bool => {
                let bool_val = val.unwrap().into_int_value();
                let i64_val = self.builder.build_int_z_extend(bool_val, self.context.i64_type(), "bool_ext").unwrap();
                let f = self.module.get_function("runtime_print_int_part").unwrap();
                self.builder.build_call(f, &[i64_val.into()], "").unwrap();
            }
            _ => {
                if let Some(v) = val {
                    let f = self.module.get_function("runtime_print_str_part").unwrap();
                    self.builder.build_call(f, &[v.into()], "").unwrap();
                }
            }
        }
        Ok(())
    }

    fn codegen_print(
        &mut self,
        arg: &Expression,
    ) -> Result<Option<BasicValueEnum<'ctx>>, GBasicError> {
        // Special-case: print(StringInterp) — emit parts + newline
        if let Expression::StringInterp { parts, .. } = arg {
            for part in parts {
                match part {
                    StringPart::Lit(s) => {
                        let global = self.builder.build_global_string_ptr(s, "str_part").unwrap();
                        let f = self.module.get_function("runtime_print_str_part").unwrap();
                        self.builder.build_call(f, &[global.as_pointer_value().into()], "").unwrap();
                    }
                    StringPart::Expr(e) => {
                        self.emit_print_expr_part(e)?;
                    }
                }
            }
            let newline_fn = self.module.get_function("runtime_print_newline").unwrap();
            self.builder.build_call(newline_fn, &[], "").unwrap();
            return Ok(None);
        }

        let arg_ty = self.infer_expr_type(arg);
        let val = self.codegen_expression(arg)?;

        match arg_ty {
            Type::String => {
                let print_fn = self.module.get_function("runtime_print").unwrap();
                self.builder
                    .build_call(print_fn, &[val.unwrap().into()], "")
                    .unwrap();
            }
            Type::Int => {
                let print_fn = self.module.get_function("runtime_print_int").unwrap();
                self.builder
                    .build_call(print_fn, &[val.unwrap().into()], "")
                    .unwrap();
            }
            Type::Float => {
                let print_fn = self.module.get_function("runtime_print_float").unwrap();
                self.builder
                    .build_call(print_fn, &[val.unwrap().into()], "")
                    .unwrap();
            }
            Type::Bool => {
                // Convert bool to int and print as int
                let bool_val = val.unwrap().into_int_value();
                let i64_val = self
                    .builder
                    .build_int_z_extend(bool_val, self.context.i64_type(), "bool_ext")
                    .unwrap();
                let print_fn = self.module.get_function("runtime_print_int").unwrap();
                self.builder
                    .build_call(print_fn, &[i64_val.into()], "")
                    .unwrap();
            }
            _ => {
                // Fallback: try print as string pointer
                if let Some(v) = val {
                    let print_fn = self.module.get_function("runtime_print").unwrap();
                    self.builder
                        .build_call(print_fn, &[v.into()], "")
                        .unwrap();
                }
            }
        }
        Ok(None)
    }

    fn codegen_int_binop(
        &self,
        lv: inkwell::values::IntValue<'ctx>,
        op: &BinaryOp,
        rv: inkwell::values::IntValue<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, GBasicError> {
        Ok(match op {
            BinaryOp::Add => self.builder.build_int_add(lv, rv, "add").unwrap().into(),
            BinaryOp::Sub => self.builder.build_int_sub(lv, rv, "sub").unwrap().into(),
            BinaryOp::Mul => self.builder.build_int_mul(lv, rv, "mul").unwrap().into(),
            BinaryOp::Div => self
                .builder
                .build_int_signed_div(lv, rv, "div")
                .unwrap()
                .into(),
            BinaryOp::Mod => self
                .builder
                .build_int_signed_rem(lv, rv, "rem")
                .unwrap()
                .into(),
            BinaryOp::Eq => self
                .builder
                .build_int_compare(inkwell::IntPredicate::EQ, lv, rv, "eq")
                .unwrap()
                .into(),
            BinaryOp::Neq => self
                .builder
                .build_int_compare(inkwell::IntPredicate::NE, lv, rv, "neq")
                .unwrap()
                .into(),
            BinaryOp::Lt => self
                .builder
                .build_int_compare(inkwell::IntPredicate::SLT, lv, rv, "lt")
                .unwrap()
                .into(),
            BinaryOp::Gt => self
                .builder
                .build_int_compare(inkwell::IntPredicate::SGT, lv, rv, "gt")
                .unwrap()
                .into(),
            BinaryOp::Le => self
                .builder
                .build_int_compare(inkwell::IntPredicate::SLE, lv, rv, "le")
                .unwrap()
                .into(),
            BinaryOp::Ge => self
                .builder
                .build_int_compare(inkwell::IntPredicate::SGE, lv, rv, "ge")
                .unwrap()
                .into(),
            BinaryOp::And => self.builder.build_and(lv, rv, "and").unwrap().into(),
            BinaryOp::Or => self.builder.build_or(lv, rv, "or").unwrap().into(),
        })
    }

    fn codegen_float_binop(
        &self,
        lv: inkwell::values::FloatValue<'ctx>,
        op: &BinaryOp,
        rv: inkwell::values::FloatValue<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, GBasicError> {
        Ok(match op {
            BinaryOp::Add => self.builder.build_float_add(lv, rv, "add").unwrap().into(),
            BinaryOp::Sub => self.builder.build_float_sub(lv, rv, "sub").unwrap().into(),
            BinaryOp::Mul => self.builder.build_float_mul(lv, rv, "mul").unwrap().into(),
            BinaryOp::Div => self.builder.build_float_div(lv, rv, "div").unwrap().into(),
            BinaryOp::Mod => self.builder.build_float_rem(lv, rv, "rem").unwrap().into(),
            BinaryOp::Eq => self
                .builder
                .build_float_compare(inkwell::FloatPredicate::OEQ, lv, rv, "eq")
                .unwrap()
                .into(),
            BinaryOp::Neq => self
                .builder
                .build_float_compare(inkwell::FloatPredicate::ONE, lv, rv, "neq")
                .unwrap()
                .into(),
            BinaryOp::Lt => self
                .builder
                .build_float_compare(inkwell::FloatPredicate::OLT, lv, rv, "lt")
                .unwrap()
                .into(),
            BinaryOp::Gt => self
                .builder
                .build_float_compare(inkwell::FloatPredicate::OGT, lv, rv, "gt")
                .unwrap()
                .into(),
            BinaryOp::Le => self
                .builder
                .build_float_compare(inkwell::FloatPredicate::OLE, lv, rv, "le")
                .unwrap()
                .into(),
            BinaryOp::Ge => self
                .builder
                .build_float_compare(inkwell::FloatPredicate::OGE, lv, rv, "ge")
                .unwrap()
                .into(),
            _ => {
                return Err(GBasicError::CodegenError {
                    message: format!("unsupported float op: {op}"),
                })
            }
        })
    }

    fn infer_expr_type(&self, expr: &Expression) -> Type {
        match expr {
            Expression::Literal(lit) => match &lit.kind {
                LiteralKind::Int(_) => Type::Int,
                LiteralKind::Float(_) => Type::Float,
                LiteralKind::String(_) => Type::String,
                LiteralKind::Bool(_) => Type::Bool,
            },
            Expression::Identifier(id) => {
                self.lookup_var(&id.name)
                    .map(|v| v.ty.clone())
                    .unwrap_or(Type::Unknown)
            }
            Expression::BinaryOp { left, op, .. } => {
                match op {
                    BinaryOp::Eq | BinaryOp::Neq | BinaryOp::Lt | BinaryOp::Gt
                    | BinaryOp::Le | BinaryOp::Ge | BinaryOp::And | BinaryOp::Or => Type::Bool,
                    _ => self.infer_expr_type(left),
                }
            }
            Expression::UnaryOp { op, operand, .. } => match op {
                UnaryOp::Not => Type::Bool,
                UnaryOp::Neg => self.infer_expr_type(operand),
            },
            Expression::Call { callee, .. } => {
                if let Expression::Identifier(id) = callee.as_ref() {
                    if id.name == "print" {
                        return Type::Void;
                    }
                    if let Some(func) = self.module.get_function(&id.name) {
                        let ret = func.get_type().get_return_type();
                        return match ret {
                            None => Type::Void,
                            Some(t) if t.is_int_type() => {
                                let int_t = t.into_int_type();
                                if int_t.get_bit_width() == 1 {
                                    Type::Bool
                                } else {
                                    Type::Int
                                }
                            }
                            Some(t) if t.is_float_type() => Type::Float,
                            _ => Type::Unknown,
                        };
                    }
                }
                Type::Unknown
            }
            Expression::StringInterp { .. } => Type::String,
            Expression::Assignment { value, .. } => self.infer_expr_type(value),
            Expression::Array { .. } => Type::Unknown,
            _ => Type::Unknown,
        }
    }

    fn build_alloca_for_type(
        &self,
        ty: &Type,
        name: &str,
    ) -> PointerValue<'ctx> {
        match ty {
            Type::Int => self
                .builder
                .build_alloca(self.context.i64_type(), name)
                .unwrap(),
            Type::Float => self
                .builder
                .build_alloca(self.context.f64_type(), name)
                .unwrap(),
            Type::Bool => self
                .builder
                .build_alloca(self.context.bool_type(), name)
                .unwrap(),
            Type::String | Type::Unknown => self
                .builder
                .build_alloca(
                    self.context.ptr_type(inkwell::AddressSpace::default()),
                    name,
                )
                .unwrap(),
            _ => self
                .builder
                .build_alloca(self.context.i64_type(), name)
                .unwrap(),
        }
    }

    fn type_to_llvm_basic(
        &self,
        ty: &Type,
    ) -> inkwell::types::BasicTypeEnum<'ctx> {
        match ty {
            Type::Int => self.context.i64_type().into(),
            Type::Float => self.context.f64_type().into(),
            Type::Bool => self.context.bool_type().into(),
            Type::String => self
                .context
                .ptr_type(inkwell::AddressSpace::default())
                .into(),
            Type::Unknown => self
                .context
                .ptr_type(inkwell::AddressSpace::default())
                .into(),
            _ => self.context.i64_type().into(),
        }
    }

    fn type_to_llvm_meta(
        &self,
        ty: &Type,
    ) -> BasicMetadataTypeEnum<'ctx> {
        match ty {
            Type::Int => self.context.i64_type().into(),
            Type::Float => self.context.f64_type().into(),
            Type::Bool => self.context.bool_type().into(),
            Type::String | Type::Unknown => self
                .context
                .ptr_type(inkwell::AddressSpace::default())
                .into(),
            _ => self.context.i64_type().into(),
        }
    }

    fn emit_and_link(&self, output_path: &str) -> Result<(), GBasicError> {
        Target::initialize_native(&InitializationConfig::default()).map_err(|e| {
            GBasicError::CodegenError {
                message: format!("failed to init native target: {e}"),
            }
        })?;

        let triple = TargetMachine::get_default_triple();
        let target = Target::from_triple(&triple).map_err(|e| GBasicError::CodegenError {
            message: format!("failed to get target: {e}"),
        })?;
        let machine = target
            .create_target_machine(
                &triple,
                "generic",
                "",
                OptimizationLevel::Default,
                RelocMode::PIC,
                CodeModel::Default,
            )
            .ok_or_else(|| GBasicError::CodegenError {
                message: "failed to create target machine".into(),
            })?;

        let obj_path_str = format!("{output_path}.o");
        let obj_path = Path::new(&obj_path_str);
        machine
            .write_to_file(&self.module, FileType::Object, obj_path)
            .map_err(|e| GBasicError::CodegenError {
                message: format!("failed to write object file: {e}"),
            })?;

        // Find workspace root: try exe dir ancestors, then CARGO_MANIFEST_DIR, then cwd
        let workspace_root = std::env::current_exe()
            .ok()
            .and_then(|exe| {
                // exe is typically in target/debug/gbasic, so go up 3 levels
                let mut p = exe.as_path();
                for _ in 0..3 {
                    p = p.parent()?;
                }
                // Verify it looks like our workspace
                if p.join("Cargo.toml").exists() {
                    Some(p.to_path_buf())
                } else {
                    None
                }
            })
            .or_else(|| {
                std::env::var("CARGO_MANIFEST_DIR").ok().map(|d| {
                    Path::new(&d)
                        .parent()
                        .unwrap()
                        .parent()
                        .unwrap()
                        .to_path_buf()
                })
            })
            .unwrap_or_else(|| std::env::current_dir().unwrap());

        // Try release first, then debug
        let (target_dir, runtime_lib) = {
            let release_dir = workspace_root.join("target/release");
            let release_lib = release_dir.join("libgbasic_runtime_desktop.a");
            let debug_dir = workspace_root.join("target/debug");
            let debug_lib = debug_dir.join("libgbasic_runtime_desktop.a");
            if release_lib.exists() {
                (release_dir, release_lib)
            } else {
                (debug_dir, debug_lib)
            }
        };

        // Find SDL2 bundled lib
        let build_dir = target_dir.join("build");
        let mut sdl2_lib_dir = None;
        if let Ok(entries) = std::fs::read_dir(&build_dir) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.starts_with("sdl2-sys-") {
                    let lib_path = entry.path().join("out/lib");
                    if lib_path.exists() {
                        sdl2_lib_dir = Some(lib_path);
                    }
                }
            }
        }

        let mut cmd = Command::new("cc");
        cmd.arg(&obj_path_str)
            .arg("-o")
            .arg(output_path);

        if runtime_lib.exists() {
            cmd.arg(runtime_lib.to_str().unwrap());

            if let Some(ref sdl2_dir) = sdl2_lib_dir {
                cmd.arg(format!("-L{}", sdl2_dir.display()))
                    .arg(format!("-Wl,-rpath,{}", sdl2_dir.display()))
                    .arg("-lSDL2")
                    .arg("-framework").arg("Cocoa")
                    .arg("-framework").arg("IOKit")
                    .arg("-framework").arg("CoreVideo")
                    .arg("-framework").arg("CoreAudio")
                    .arg("-framework").arg("AudioToolbox")
                    .arg("-framework").arg("Carbon")
                    .arg("-framework").arg("ForceFeedback")
                    .arg("-framework").arg("GameController")
                    .arg("-framework").arg("CoreHaptics")
                    .arg("-framework").arg("Metal")
                    .arg("-liconv");
            }
        }

        let status = cmd.status().map_err(|e| GBasicError::CodegenError {
            message: format!("failed to run linker: {e}"),
        })?;

        if !status.success() {
            return Err(GBasicError::CodegenError {
                message: format!("linking failed with status: {status}"),
            });
        }

        // Clean up object file
        let _ = std::fs::remove_file(&obj_path_str);

        Ok(())
    }
}
