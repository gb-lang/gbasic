use gbasic_common::ast::*;
use gbasic_common::error::GBasicError;
use gbasic_common::span::Span;
use gbasic_common::types::Type;
use inkwell::basic_block::BasicBlock;
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::targets::{
    CodeModel, FileType, InitializationConfig, RelocMode, Target, TargetMachine,
};
use inkwell::types::{BasicMetadataTypeEnum, BasicType};
use inkwell::values::{BasicMetadataValueEnum, BasicValueEnum, FunctionValue, PointerValue};
use inkwell::OptimizationLevel;
use std::collections::HashMap;
use std::path::Path;
use std::process::Command;

/// LLVM type descriptor for namespace method signatures
#[derive(Debug, Clone, Copy, PartialEq)]
enum LType {
    I64,
    F64,
    Bool,
    Ptr, // *const i8
    Void,
}

impl LType {
    fn to_gbasic_type(self) -> Type {
        match self {
            LType::I64 => Type::Int,
            LType::F64 => Type::Float,
            LType::Bool => Type::Bool,
            LType::Ptr => Type::String,
            LType::Void => Type::Void,
        }
    }
}

/// Unified namespace method entry: signature + runtime function name.
struct MethodEntry {
    params: Vec<LType>,
    ret: LType,
    runtime_name: String,
}

/// Multi-word method names to snake_case for runtime function naming.
fn method_to_snake(method: &str) -> &str {
    match method {
        "setpixel" => "set_pixel",
        "drawrect" => "draw_rect",
        "drawline" => "draw_line",
        "drawcircle" => "draw_circle",
        "keypressed" => "key_pressed",
        "mousex" => "mouse_x",
        "mousey" => "mouse_y",
        "readfile" => "read_file",
        "writefile" => "write_file",
        "framebegin" => "frame_begin",
        "frameend" => "frame_end",
        "frametime" => "frame_time",
        "spriteload" => "sprite_load",
        "spriteat" => "sprite_at",
        "spritescale" => "sprite_scale",
        "spritedraw" => "sprite_draw",
        "effectload" => "effect_load",
        "effectplay" => "effect_play",
        "effectvolume" => "effect_volume",
        other => other,
    }
}

/// Single source of truth for namespace method signatures and runtime names.
fn get_namespace_method(namespace: NamespaceRef, method: &str) -> Option<MethodEntry> {
    use LType::*;
    use NamespaceRef::*;
    let (params, ret) = match (namespace, method) {
        // Math
        (Math, "sin" | "cos" | "sqrt" | "abs" | "floor" | "ceil") => (vec![F64], F64),
        (Math, "pow" | "max" | "min") => (vec![F64, F64], F64),
        (Math, "random" | "pi") => (vec![], F64),
        // Screen
        (Screen, "init") => (vec![I64, I64], Void),
        (Screen, "clear") => (vec![I64, I64, I64], Void),
        (Screen, "setpixel") => (vec![I64, I64, I64, I64, I64], Void),
        (Screen, "drawrect") => (vec![I64, I64, I64, I64, I64, I64, I64], Void),
        (Screen, "drawline") => (vec![I64, I64, I64, I64, I64, I64, I64], Void),
        (Screen, "present") => (vec![], Void),
        (Screen, "width" | "height") => (vec![], I64),
        (Screen, "drawcircle") => (vec![I64, I64, I64, I64, I64, I64], Void),
        (Screen, "spriteload") => (vec![Ptr], I64),
        (Screen, "spriteat") => (vec![I64, F64, F64], I64),
        (Screen, "spritescale") => (vec![I64, F64], I64),
        (Screen, "spritedraw") => (vec![I64], Void),
        // Input
        (Input, "keypressed") => (vec![Ptr], Bool),
        (Input, "mousex" | "mousey") => (vec![], I64),
        (Input, "poll") => (vec![], Void),
        // System
        (System, "time") => (vec![], F64),
        (System, "sleep") => (vec![I64], Void),
        (System, "exit") => (vec![I64], Void),
        (System, "framebegin") => (vec![], Void),
        (System, "frameend") => (vec![], Void),
        (System, "frametime") => (vec![], F64),
        // Sound
        (Sound, "beep") => (vec![I64, I64], Void),
        (Sound, "effectload") => (vec![Ptr], I64),
        (Sound, "effectplay") => (vec![Ptr], Void),
        (Sound, "effectvolume") => (vec![Ptr, F64], Void),
        // Memory
        (Memory, "set") => (vec![Ptr, I64], Void),
        (Memory, "get") => (vec![Ptr], I64),
        // IO
        (IO, "print") => (vec![Ptr], Void),
        (IO, "printinteger") => (vec![I64], Void),
        (IO, "readfile") => (vec![Ptr], Ptr),
        (IO, "writefile") => (vec![Ptr, Ptr], Void),
        _ => return None,
    };
    // Special-case runtime names that don't follow the convention
    let runtime_name = match (namespace, method) {
        (IO, "print") => "runtime_print".to_string(),
        (IO, "printinteger") => "runtime_print_int".to_string(),
        _ => {
            let ns = match namespace {
                Screen => "screen", Sound => "sound", Input => "input",
                Math => "math", System => "system", Memory => "memory", IO => "io",
            };
            format!("runtime_{ns}_{}", method_to_snake(method))
        }
    };
    Some(MethodEntry { params, ret, runtime_name })
}

/// Variable info: alloca pointer + type
struct VarInfo<'ctx> {
    ptr: PointerValue<'ctx>,
    ty: Type,
}

/// Named color RGB constants.
fn named_color(name: &str) -> Option<(u8, u8, u8)> {
    match name {
        "black" => Some((0, 0, 0)),
        "white" => Some((255, 255, 255)),
        "red" => Some((255, 0, 0)),
        "green" => Some((0, 255, 0)),
        "blue" => Some((0, 0, 255)),
        "yellow" => Some((255, 255, 0)),
        "orange" => Some((255, 165, 0)),
        "purple" => Some((128, 0, 128)),
        "pink" => Some((255, 192, 203)),
        "cyan" => Some((0, 255, 255)),
        "gray" | "grey" => Some((128, 128, 128)),
        "brown" => Some((139, 69, 19)),
        _ => None,
    }
}

/// Resolve nested field access chain to a property path string.
/// E.g. `paddle.position.x` → ("paddle", "position.x")
fn resolve_field_chain(expr: &Expression) -> Option<(String, String)> {
    match expr {
        Expression::FieldAccess { object, field, .. } => {
            match object.as_ref() {
                Expression::Identifier(id) => Some((id.name.clone(), field.name.clone())),
                Expression::FieldAccess { object: inner_obj, field: inner_field, .. } => {
                    if let Expression::Identifier(id) = inner_obj.as_ref() {
                        Some((id.name.clone(), format!("{}.{}", inner_field.name, field.name)))
                    } else {
                        None
                    }
                }
                _ => None,
            }
        }
        _ => None,
    }
}

pub struct Codegen<'ctx> {
    context: &'ctx Context,
    module: Module<'ctx>,
    builder: Builder<'ctx>,
    variables: Vec<HashMap<String, VarInfo<'ctx>>>,
    current_function: Option<FunctionValue<'ctx>>,
    /// Stack of (continue_target, break_target) for loops
    loop_exit_stack: Vec<(BasicBlock<'ctx>, BasicBlock<'ctx>)>,
    /// Whether we're inside an auto-framed while-true loop
    in_auto_frame: bool,
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
            loop_exit_stack: Vec::new(),
            in_auto_frame: false,
        }
    }

    fn needs_terminator(&self) -> bool {
        self.builder.get_insert_block().unwrap().get_terminator().is_none()
    }

    /// Ensure an integer value is i1 for use as a branch condition.
    /// If already i1, returns as-is. Otherwise compares != 0.
    fn ensure_i1(&self, val: inkwell::values::IntValue<'ctx>) -> inkwell::values::IntValue<'ctx> {
        if val.get_type().get_bit_width() == 1 {
            val
        } else {
            let zero = val.get_type().const_int(0, false);
            self.builder.build_int_compare(inkwell::IntPredicate::NE, val, zero, "tobool").unwrap()
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

    /// Declare (or reuse) a runtime function and call it. Returns the call site value.
    fn call_runtime(
        &self,
        name: &str,
        param_types: &[LType],
        ret: LType,
        args: &[BasicMetadataValueEnum<'ctx>],
    ) -> Option<BasicValueEnum<'ctx>> {
        let function = if let Some(f) = self.module.get_function(name) {
            f
        } else {
            let params: Vec<BasicMetadataTypeEnum> = param_types.iter().map(|t| self.ltype_to_meta(*t)).collect();
            let fn_type = match ret {
                LType::Void => self.context.void_type().fn_type(&params, false),
                LType::I64 => self.context.i64_type().fn_type(&params, false),
                LType::F64 => self.context.f64_type().fn_type(&params, false),
                LType::Bool => self.context.i64_type().fn_type(&params, false),
                LType::Ptr => self.context.ptr_type(inkwell::AddressSpace::default()).fn_type(&params, false),
            };
            self.module.add_function(name, fn_type, None)
        };
        let label = if ret == LType::Void { "" } else { "rt_call" };
        let result = self.builder.build_call(function, args, label).unwrap();
        match ret {
            LType::Void => None,
            _ => result.try_as_basic_value().left(),
        }
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

        // runtime_string_concat(a: *const i8, b: *const i8) -> *const i8
        let concat_ty = i8_ptr_type.fn_type(&[i8_ptr_type.into(), i8_ptr_type.into()], false);
        self.module.add_function("runtime_string_concat", concat_ty, None);
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
        if cg.needs_terminator() {
            cg.builder
                .build_return(Some(&i32_type.const_int(0, false)))
                .unwrap();
        }

        // Verify
        cg.module
            .verify()
            .map_err(|e| GBasicError::CodegenError {
                span: None, message: format!("LLVM verification failed: {}", e.to_string()),
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
                span: None, message: format!("function '{}' not declared", func.name.name),
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
                if self.needs_terminator() {
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
        if self.needs_terminator() {
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
                let cond_raw = self.codegen_expression(condition)?.unwrap().into_int_value();
                let cond = self.ensure_i1(cond_raw);
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
                if self.needs_terminator() {
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
                if self.needs_terminator() {
                    self.builder.build_unconditional_branch(merge_bb).unwrap();
                }

                self.builder.position_at_end(merge_bb);
            }
            Statement::While {
                condition, body, ..
            } => {
                // Detect `while true` at top-level for implicit frame management
                let is_while_true = matches!(
                    condition,
                    Expression::Literal(Literal { kind: LiteralKind::Bool(true), .. })
                );
                let auto_frame = is_while_true && !self.in_auto_frame;

                if auto_frame {
                    // Ensure screen is initialized
                    self.call_runtime("ensure_screen_init", &[], LType::Void, &[]);
                    self.in_auto_frame = true;
                }

                let function = self.current_function.unwrap();
                let cond_bb = self.context.append_basic_block(function, "while_cond");
                let body_bb = self.context.append_basic_block(function, "while_body");
                let exit_bb = self.context.append_basic_block(function, "while_exit");

                self.builder.build_unconditional_branch(cond_bb).unwrap();
                self.builder.position_at_end(cond_bb);
                let cond_raw = self.codegen_expression(condition)?.unwrap().into_int_value();
                let cond = self.ensure_i1(cond_raw);
                self.builder
                    .build_conditional_branch(cond, body_bb, exit_bb)
                    .unwrap();

                self.builder.position_at_end(body_bb);

                // Auto-frame: poll input + begin frame at top of loop
                if auto_frame {
                    self.call_runtime("runtime_frame_auto", &[], LType::Void, &[]);
                }

                self.push_scope();
                self.loop_exit_stack.push((cond_bb, exit_bb));
                for s in &body.statements {
                    self.codegen_statement(s)?;
                }
                self.loop_exit_stack.pop();
                self.pop_scope();

                // Auto-frame: physics + draw + present + timing at end of loop
                if auto_frame {
                    if self.needs_terminator() {
                        self.call_runtime("runtime_frame_auto_end", &[], LType::Void, &[]);
                    }
                }

                if self.needs_terminator() {
                    self.builder.build_unconditional_branch(cond_bb).unwrap();
                }

                self.builder.position_at_end(exit_bb);

                if auto_frame {
                    self.in_auto_frame = false;
                }
            }
            Statement::For {
                variable,
                iterable,
                body,
                ..
            } => {
                self.codegen_for_loop(variable, iterable, body)?;
            }
            Statement::Match {
                subject, arms, ..
            } => {
                self.codegen_match(subject, arms)?;
            }
            Statement::Break { .. } => {
                if let Some(&(_, break_bb)) = self.loop_exit_stack.last() {
                    if self.needs_terminator() {
                        self.builder.build_unconditional_branch(break_bb).unwrap();
                    }
                }
            }
            Statement::Continue { .. } => {
                if let Some(&(continue_bb, _)) = self.loop_exit_stack.last() {
                    if self.needs_terminator() {
                        self.builder.build_unconditional_branch(continue_bb).unwrap();
                    }
                }
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
        }
        Ok(())
    }

    /// Create the 4 basic blocks for a for-loop: cond, body, inc, exit.
    fn make_loop_blocks(&self) -> (BasicBlock<'ctx>, BasicBlock<'ctx>, BasicBlock<'ctx>, BasicBlock<'ctx>) {
        let function = self.current_function.unwrap();
        (
            self.context.append_basic_block(function, "for_cond"),
            self.context.append_basic_block(function, "for_body"),
            self.context.append_basic_block(function, "for_inc"),
            self.context.append_basic_block(function, "for_exit"),
        )
    }

    /// Codegen a loop body with proper scope, loop_exit_stack, and fallthrough to inc_bb.
    fn codegen_loop_body(
        &mut self,
        var_name: &str,
        var_alloca: PointerValue<'ctx>,
        var_ty: Type,
        body: &Block,
        inc_bb: BasicBlock<'ctx>,
        exit_bb: BasicBlock<'ctx>,
    ) -> Result<(), GBasicError> {
        self.push_scope();
        self.insert_var(var_name.to_string(), VarInfo { ptr: var_alloca, ty: var_ty });
        self.loop_exit_stack.push((inc_bb, exit_bb));
        for s in &body.statements {
            self.codegen_statement(s)?;
        }
        self.loop_exit_stack.pop();
        self.pop_scope();
        if self.needs_terminator() {
            self.builder.build_unconditional_branch(inc_bb).unwrap();
        }
        Ok(())
    }

    fn codegen_for_loop(
        &mut self,
        variable: &Identifier,
        iterable: &Expression,
        body: &Block,
    ) -> Result<(), GBasicError> {
        let i64_type = self.context.i64_type();

        // Check if iterable is a Range expression → simple int counter loop
        if let Expression::Range { start, end, .. } = iterable {
            let start_val = self.codegen_expression(start)?.unwrap().into_int_value();
            let end_val = self.codegen_expression(end)?.unwrap().into_int_value();

            let var_alloca = self.builder.build_alloca(i64_type, &variable.name).unwrap();
            self.builder.build_store(var_alloca, start_val).unwrap();

            let (cond_bb, body_bb, inc_bb, exit_bb) = self.make_loop_blocks();

            self.builder.build_unconditional_branch(cond_bb).unwrap();
            self.builder.position_at_end(cond_bb);
            let current = self.builder.build_load(i64_type, var_alloca, "i").unwrap().into_int_value();
            let cond = self.builder.build_int_compare(
                inkwell::IntPredicate::SLT, current, end_val, "for_cond"
            ).unwrap();
            self.builder.build_conditional_branch(cond, body_bb, exit_bb).unwrap();

            self.builder.position_at_end(body_bb);
            self.codegen_loop_body(&variable.name, var_alloca, Type::Int, body, inc_bb, exit_bb)?;

            self.builder.position_at_end(inc_bb);
            let next = self.builder.build_int_add(
                self.builder.build_load(i64_type, var_alloca, "i").unwrap().into_int_value(),
                i64_type.const_int(1, false),
                "inc"
            ).unwrap();
            self.builder.build_store(var_alloca, next).unwrap();
            self.builder.build_unconditional_branch(cond_bb).unwrap();

            self.builder.position_at_end(exit_bb);
            return Ok(());
        }

        // Array iteration: codegen array, iterate with index counter
        if let Expression::Array { elements, .. } = iterable {
            if elements.is_empty() {
                return Ok(());
            }

            let elem_ty = self.infer_expr_type(&elements[0]);
            let llvm_elem_ty = self.type_to_llvm_basic(&elem_ty);
            let len = elements.len() as u64;

            let array_ty = llvm_elem_ty.array_type(len as u32);
            let array_alloca = self.builder.build_alloca(array_ty, "arr").unwrap();

            for (i, elem) in elements.iter().enumerate() {
                let val = self.codegen_expression(elem)?.unwrap();
                let gep = unsafe {
                    self.builder.build_gep(
                        array_ty, array_alloca,
                        &[i64_type.const_int(0, false), i64_type.const_int(i as u64, false)],
                        "elem_ptr",
                    ).unwrap()
                };
                self.builder.build_store(gep, val).unwrap();
            }

            let idx_alloca = self.builder.build_alloca(i64_type, "idx").unwrap();
            self.builder.build_store(idx_alloca, i64_type.const_int(0, false)).unwrap();
            let var_alloca = self.builder.build_alloca(llvm_elem_ty, &variable.name).unwrap();

            let (cond_bb, body_bb, inc_bb, exit_bb) = self.make_loop_blocks();

            self.builder.build_unconditional_branch(cond_bb).unwrap();
            self.builder.position_at_end(cond_bb);
            let idx = self.builder.build_load(i64_type, idx_alloca, "idx").unwrap().into_int_value();
            let cond = self.builder.build_int_compare(
                inkwell::IntPredicate::SLT, idx, i64_type.const_int(len, false), "for_cond"
            ).unwrap();
            self.builder.build_conditional_branch(cond, body_bb, exit_bb).unwrap();

            self.builder.position_at_end(body_bb);
            let idx_val = self.builder.build_load(i64_type, idx_alloca, "idx").unwrap().into_int_value();
            let elem_ptr = unsafe {
                self.builder.build_gep(
                    array_ty, array_alloca,
                    &[i64_type.const_int(0, false), idx_val],
                    "elem_ptr",
                ).unwrap()
            };
            let elem_val = self.builder.build_load(llvm_elem_ty, elem_ptr, "elem").unwrap();
            self.builder.build_store(var_alloca, elem_val).unwrap();

            self.codegen_loop_body(&variable.name, var_alloca, elem_ty, body, inc_bb, exit_bb)?;

            self.builder.position_at_end(inc_bb);
            let next_idx = self.builder.build_int_add(
                self.builder.build_load(i64_type, idx_alloca, "idx").unwrap().into_int_value(),
                i64_type.const_int(1, false),
                "inc"
            ).unwrap();
            self.builder.build_store(idx_alloca, next_idx).unwrap();
            self.builder.build_unconditional_branch(cond_bb).unwrap();

            self.builder.position_at_end(exit_bb);
            return Ok(());
        }

        // Dynamic array iteration: codegen iterable as a handle, iterate with index counter
        let arr_handle = self.codegen_expression(iterable)?.unwrap();
        let len = self.call_runtime("runtime_array_length", &[LType::I64], LType::I64, &[arr_handle.into()]).unwrap().into_int_value();

        let idx_alloca = self.builder.build_alloca(i64_type, "idx").unwrap();
        self.builder.build_store(idx_alloca, i64_type.const_int(0, false)).unwrap();
        let var_alloca = self.builder.build_alloca(i64_type, &variable.name).unwrap();

        let (cond_bb, body_bb, inc_bb, exit_bb) = self.make_loop_blocks();

        self.builder.build_unconditional_branch(cond_bb).unwrap();
        self.builder.position_at_end(cond_bb);
        let idx = self.builder.build_load(i64_type, idx_alloca, "idx").unwrap().into_int_value();
        // Re-read length each iteration (array may change during loop)
        let cur_len = self.call_runtime("runtime_array_length", &[LType::I64], LType::I64, &[arr_handle.into()]).unwrap().into_int_value();
        let cond = self.builder.build_int_compare(
            inkwell::IntPredicate::SLT, idx, cur_len, "for_cond"
        ).unwrap();
        self.builder.build_conditional_branch(cond, body_bb, exit_bb).unwrap();

        self.builder.position_at_end(body_bb);
        let idx_val = self.builder.build_load(i64_type, idx_alloca, "idx").unwrap().into_int_value();
        let elem_val = self.call_runtime("runtime_array_get", &[LType::I64, LType::I64], LType::I64, &[arr_handle.into(), idx_val.into()]).unwrap();
        self.builder.build_store(var_alloca, elem_val).unwrap();

        self.codegen_loop_body(&variable.name, var_alloca, Type::Int, body, inc_bb, exit_bb)?;

        self.builder.position_at_end(inc_bb);
        let next_idx = self.builder.build_int_add(
            self.builder.build_load(i64_type, idx_alloca, "idx").unwrap().into_int_value(),
            i64_type.const_int(1, false),
            "inc"
        ).unwrap();
        self.builder.build_store(idx_alloca, next_idx).unwrap();
        self.builder.build_unconditional_branch(cond_bb).unwrap();

        self.builder.position_at_end(exit_bb);
        Ok(())
    }

    fn codegen_match(
        &mut self,
        subject: &Expression,
        arms: &[MatchArm],
    ) -> Result<(), GBasicError> {
        let subject_val = self.codegen_expression(subject)?.unwrap();
        let subject_ty = self.infer_expr_type(subject);
        let function = self.current_function.unwrap();
        let merge_bb = self.context.append_basic_block(function, "match_end");

        for (i, arm) in arms.iter().enumerate() {
            match &arm.pattern {
                Pattern::Wildcard(_) => {
                    // Unconditional — emit body and branch to merge
                    self.push_scope();
                    for s in &arm.body.statements {
                        self.codegen_statement(s)?;
                    }
                    self.pop_scope();
                    if self.needs_terminator() {
                        self.builder.build_unconditional_branch(merge_bb).unwrap();
                    }
                }
                Pattern::Literal(lit) => {
                    let pat_val = self.codegen_literal(lit)?;
                    let cond = self.build_equality_check(subject_val, pat_val, &subject_ty)?;

                    let arm_bb = self.context.append_basic_block(function, &format!("match_arm_{i}"));
                    let next_bb = self.context.append_basic_block(function, &format!("match_next_{i}"));

                    self.builder.build_conditional_branch(cond, arm_bb, next_bb).unwrap();

                    self.builder.position_at_end(arm_bb);
                    self.push_scope();
                    for s in &arm.body.statements {
                        self.codegen_statement(s)?;
                    }
                    self.pop_scope();
                    if self.needs_terminator() {
                        self.builder.build_unconditional_branch(merge_bb).unwrap();
                    }

                    self.builder.position_at_end(next_bb);
                }
                Pattern::Identifier(id) => {
                    // Bind the subject value to the identifier name, then execute body
                    let arm_bb = self.context.append_basic_block(function, &format!("match_arm_{i}"));
                    let next_bb = self.context.append_basic_block(function, &format!("match_next_{i}"));

                    // Identifier patterns always match (like a wildcard but with binding)
                    self.builder.build_unconditional_branch(arm_bb).unwrap();

                    self.builder.position_at_end(arm_bb);
                    self.push_scope();
                    let alloca = self.build_alloca_for_type(&subject_ty, &id.name);
                    self.builder.build_store(alloca, subject_val).unwrap();
                    self.insert_var(id.name.clone(), VarInfo { ptr: alloca, ty: subject_ty.clone() });
                    for s in &arm.body.statements {
                        self.codegen_statement(s)?;
                    }
                    self.pop_scope();
                    if self.needs_terminator() {
                        self.builder.build_unconditional_branch(merge_bb).unwrap();
                    }

                    // next_bb is unreachable after an identifier pattern (it catches all)
                    self.builder.position_at_end(next_bb);
                }
            }
        }

        // If we fall through all arms, branch to merge
        if self.needs_terminator() {
            self.builder.build_unconditional_branch(merge_bb).unwrap();
        }
        self.builder.position_at_end(merge_bb);
        Ok(())
    }

    fn codegen_literal(&mut self, lit: &Literal) -> Result<BasicValueEnum<'ctx>, GBasicError> {
        match &lit.kind {
            LiteralKind::Int(v) => Ok(self.context.i64_type().const_int(*v as u64, true).into()),
            LiteralKind::Float(v) => Ok(self.context.f64_type().const_float(*v).into()),
            LiteralKind::Bool(v) => Ok(self.context.bool_type().const_int(if *v { 1 } else { 0 }, false).into()),
            LiteralKind::String(s) => {
                let global = self.builder.build_global_string_ptr(s, "str").unwrap();
                Ok(global.as_pointer_value().into())
            }
        }
    }

    fn build_equality_check(
        &self,
        lv: BasicValueEnum<'ctx>,
        rv: BasicValueEnum<'ctx>,
        ty: &Type,
    ) -> Result<inkwell::values::IntValue<'ctx>, GBasicError> {
        match ty {
            Type::Int | Type::Bool => {
                Ok(self.builder.build_int_compare(
                    inkwell::IntPredicate::EQ, lv.into_int_value(), rv.into_int_value(), "eq"
                ).unwrap())
            }
            Type::Float => {
                Ok(self.builder.build_float_compare(
                    inkwell::FloatPredicate::OEQ, lv.into_float_value(), rv.into_float_value(), "eq"
                ).unwrap())
            }
            _ => {
                // For strings/unknown, compare as ints (pointer equality — MVP)
                Ok(self.builder.build_int_compare(
                    inkwell::IntPredicate::EQ, lv.into_int_value(), rv.into_int_value(), "eq"
                ).unwrap())
            }
        }
    }

    fn codegen_expression(
        &mut self,
        expr: &Expression,
    ) -> Result<Option<BasicValueEnum<'ctx>>, GBasicError> {
        match expr {
            Expression::Literal(lit) => Ok(Some(self.codegen_literal(lit)?)),
            Expression::Identifier(id) => {
                // Check for named colors first
                if let Some((r, g, b)) = named_color(&id.name) {
                    // Pack as i64: r << 16 | g << 8 | b
                    let packed = ((r as u64) << 16) | ((g as u64) << 8) | (b as u64);
                    return Ok(Some(self.context.i64_type().const_int(packed, false).into()));
                }

                let var = self.lookup_var(&id.name).ok_or_else(|| {
                    GBasicError::CodegenError {
                        span: Some(id.span), message: format!("undefined variable '{}'", id.name),
                    }
                })?;
                let llvm_type = self.type_to_llvm_basic(&var.ty);
                let ptr = var.ptr;
                let val = self.builder.build_load(llvm_type, ptr, &id.name).unwrap();
                Ok(Some(val))
            }
            Expression::BinaryOp {
                left, op, right, span,
            } => {
                // String concatenation via + operator
                let left_ty = self.infer_expr_type(left);
                if matches!(left_ty, Type::String) && matches!(op, BinaryOp::Add) {
                    let lv = self.codegen_expression(left)?.unwrap();
                    let rv = self.codegen_expression(right)?.unwrap();
                    let concat_fn = self.module.get_function("runtime_string_concat").unwrap();
                    let result = self.builder.build_call(
                        concat_fn,
                        &[lv.into(), rv.into()],
                        "concat"
                    ).unwrap();
                    return Ok(result.try_as_basic_value().left());
                }

                let right_ty = self.infer_expr_type(right);
                let lv = self.codegen_expression(left)?.unwrap();
                let rv = self.codegen_expression(right)?.unwrap();

                let result = match (&left_ty, &right_ty) {
                    // Mixed Int/Float: promote Int side to Float
                    (Type::Int, Type::Float) => {
                        let lf = self.builder.build_signed_int_to_float(
                            lv.into_int_value(), self.context.f64_type(), "itof"
                        ).unwrap();
                        self.codegen_float_binop(lf, op, rv.into_float_value())
                    }
                    (Type::Float, Type::Int) => {
                        let rf = self.builder.build_signed_int_to_float(
                            rv.into_int_value(), self.context.f64_type(), "itof"
                        ).unwrap();
                        self.codegen_float_binop(lv.into_float_value(), op, rf)
                    }
                    (Type::Int, _) | (Type::Bool, _) => {
                        self.codegen_int_binop(lv.into_int_value(), op, rv.into_int_value())
                    }
                    (Type::Float, _) => {
                        self.codegen_float_binop(lv.into_float_value(), op, rv.into_float_value())
                    }
                    _ => Err(GBasicError::CodegenError {
                        span: Some(*span), message: format!("unsupported binary op on {left_ty}"),
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
                            span: None, message: "cannot negate non-numeric".into(),
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
            Expression::Assignment { target, value, span } => {
                // Check if target is a field access (property setter)
                if let Some((var_name, prop_path)) = resolve_field_chain(target) {
                    let var = self.lookup_var(&var_name).ok_or_else(|| {
                        GBasicError::CodegenError {
                            span: Some(*span), message: format!("undefined variable '{var_name}'"),
                        }
                    })?;
                    let handle_ty = self.type_to_llvm_basic(&var.ty);
                    let handle = self.builder.build_load(handle_ty, var.ptr, "handle").unwrap();

                    return self.codegen_property_set(handle, &prop_path, value, *span);
                }

                let val = self.codegen_expression(value)?.unwrap();
                if let Expression::Identifier(id) = target.as_ref() {
                    let var = self.lookup_var(&id.name).ok_or_else(|| {
                        GBasicError::CodegenError {
                            span: Some(id.span), message: format!("undefined variable '{}'", id.name),
                        }
                    })?;
                    let ptr = var.ptr;
                    self.builder.build_store(ptr, val).unwrap();
                }
                Ok(Some(val))
            }
            Expression::StringInterp { parts, .. } => {
                self.emit_string_interp_parts(parts)?;
                let newline_fn = self.module.get_function("runtime_print_newline").unwrap();
                self.builder.build_call(newline_fn, &[], "").unwrap();
                let empty = self.builder.build_global_string_ptr("", "empty").unwrap();
                Ok(Some(empty.as_pointer_value().into()))
            }
            Expression::MethodChain { base, chain, .. } => {
                self.codegen_method_chain(*base, chain)
            }
            Expression::Array { elements, .. } => {
                self.codegen_array(elements)
            }
            Expression::Index { object, index, .. } => {
                self.codegen_index(object, index)
            }
            Expression::Range { .. } => {
                // Range expressions are only valid as for-loop iterables, not standalone
                Err(GBasicError::CodegenError {
                    span: None, message: "range expressions can only be used in for-loop iterables".into(),
                })
            }
            Expression::FieldAccess { .. } => {
                self.codegen_field_access_read(expr)
            }
        }
    }

    fn codegen_array(
        &mut self,
        elements: &[Expression],
    ) -> Result<Option<BasicValueEnum<'ctx>>, GBasicError> {
        if elements.is_empty() {
            // Empty array → dynamic array handle
            return Ok(self.call_runtime("runtime_array_new", &[], LType::I64, &[]));
        }

        let elem_ty = self.infer_expr_type(&elements[0]);
        let llvm_elem_ty = self.type_to_llvm_basic(&elem_ty);
        let len = elements.len() as u32;
        let array_ty = llvm_elem_ty.array_type(len);
        let alloca = self.builder.build_alloca(array_ty, "arr").unwrap();
        let i64_type = self.context.i64_type();

        for (i, elem) in elements.iter().enumerate() {
            let val = self.codegen_expression(elem)?.unwrap();
            let gep = unsafe {
                self.builder.build_gep(
                    array_ty,
                    alloca,
                    &[
                        i64_type.const_int(0, false),
                        i64_type.const_int(i as u64, false),
                    ],
                    "elem_ptr",
                ).unwrap()
            };
            self.builder.build_store(gep, val).unwrap();
        }

        // Return pointer to the array
        Ok(Some(alloca.into()))
    }

    fn codegen_index(
        &mut self,
        object: &Expression,
        index: &Expression,
    ) -> Result<Option<BasicValueEnum<'ctx>>, GBasicError> {
        let obj_val = self.codegen_expression(object)?.unwrap();
        let idx_val = self.codegen_expression(index)?.unwrap().into_int_value();

        // Infer element type from the array expression
        let elem_ty = match self.infer_expr_type(object) {
            Type::Array(inner) => *inner,
            _ => Type::Int, // fallback
        };
        let llvm_elem_ty = self.type_to_llvm_basic(&elem_ty);

        // Object should be a pointer to an array allocation
        let ptr = obj_val.into_pointer_value();

        let gep = unsafe {
            self.builder.build_gep(
                llvm_elem_ty,
                ptr,
                &[idx_val],
                "idx_ptr",
            ).unwrap()
        };
        let val = self.builder.build_load(llvm_elem_ty, gep, "idx_val").unwrap();
        Ok(Some(val))
    }

    fn ltype_to_meta(&self, t: LType) -> BasicMetadataTypeEnum<'ctx> {
        match t {
            LType::I64 => self.context.i64_type().into(),
            LType::F64 => self.context.f64_type().into(),
            LType::Bool => self.context.i64_type().into(), // bool passed as i64 in ABI
            LType::Ptr => self.context.ptr_type(inkwell::AddressSpace::default()).into(),
            LType::Void => unreachable!(),
        }
    }

    fn get_or_declare_runtime_fn(
        &self,
        namespace: NamespaceRef,
        method: &str,
    ) -> Result<(FunctionValue<'ctx>, Vec<LType>, LType), GBasicError> {
        let entry = get_namespace_method(namespace, method)
            .ok_or_else(|| GBasicError::CodegenError {
                span: None, message: format!("unknown namespace method: {namespace}.{method}"),
            })?;
        let param_types = entry.params;
        let ret_type = entry.ret;
        let fn_name = entry.runtime_name;

        let function = if let Some(f) = self.module.get_function(&fn_name) {
            f
        } else {
            let params: Vec<BasicMetadataTypeEnum> = param_types.iter().map(|t| self.ltype_to_meta(*t)).collect();
            let fn_type = match ret_type {
                LType::Void => self.context.void_type().fn_type(&params, false),
                LType::I64 => self.context.i64_type().fn_type(&params, false),
                LType::F64 => self.context.f64_type().fn_type(&params, false),
                LType::Bool => self.context.i64_type().fn_type(&params, false),
                LType::Ptr => self.context.ptr_type(inkwell::AddressSpace::default()).fn_type(&params, false),
            };
            self.module.add_function(&fn_name, fn_type, None)
        };

        Ok((function, param_types, ret_type))
    }

    fn codegen_method_chain(
        &mut self,
        namespace: NamespaceRef,
        chain: &[MethodCall],
    ) -> Result<Option<BasicValueEnum<'ctx>>, GBasicError> {
        let mut last_result: Option<BasicValueEnum<'ctx>> = None;
        let mut last_screen_pos: Option<String> = None;

        for call in chain {
            let method_name = &call.method.name; // already lowercased by lexer

            // Handle Screen properties that aren't in the namespace table
            if namespace == NamespaceRef::Screen {
                match method_name.as_str() {
                    "center" | "bottom_center" | "top_center" | "top_left" | "top_right"
                    | "bottom_left" | "bottom_right" => {
                        self.call_runtime("ensure_screen_init", &[], LType::Void, &[]);
                        last_screen_pos = Some(method_name.clone());
                        last_result = self.call_runtime("runtime_screen_center_x", &[], LType::F64, &[]);
                        continue;
                    }
                    // .x / .y after a Screen position property (e.g., Screen.center.y)
                    "x" | "y" if last_screen_pos.is_some() => {
                        let pos_name = last_screen_pos.as_deref().unwrap();
                        let f64_type = self.context.f64_type();
                        last_result = Some(match (pos_name, method_name.as_str()) {
                            ("center", "x") | ("top_center", "x") | ("bottom_center", "x") =>
                                self.call_runtime("runtime_screen_center_x", &[], LType::F64, &[]).unwrap(),
                            ("center", "y") =>
                                self.call_runtime("runtime_screen_center_y", &[], LType::F64, &[]).unwrap(),
                            ("bottom_center", "y") | ("bottom_left", "y") | ("bottom_right", "y") => {
                                let h = self.call_runtime("runtime_screen_height", &[], LType::I64, &[]).unwrap();
                                self.builder.build_signed_int_to_float(h.into_int_value(), f64_type, "hf").unwrap().into()
                            }
                            ("top_left", "x") | ("top_left", "y") | ("top_center", "y")
                            | ("bottom_left", "x") => f64_type.const_float(0.0).into(),
                            ("top_right", "x") | ("bottom_right", "x") => {
                                let w = self.call_runtime("runtime_screen_width", &[], LType::I64, &[]).unwrap();
                                self.builder.build_signed_int_to_float(w.into_int_value(), f64_type, "wf").unwrap().into()
                            }
                            ("top_right", "y") => f64_type.const_float(0.0).into(),
                            _ => self.call_runtime("runtime_screen_center_x", &[], LType::F64, &[]).unwrap(),
                        });
                        last_screen_pos = None;
                        continue;
                    }
                    _ => {}
                }
            }

            let (function, param_types, ret_type) = self.get_or_declare_runtime_fn(namespace, method_name)?;

            // Codegen args, casting as needed
            let mut compiled_args: Vec<BasicMetadataValueEnum> = Vec::new();
            for (i, arg) in call.args.iter().enumerate() {
                let val = self.codegen_expression(arg)?.ok_or_else(|| GBasicError::CodegenError {
                    span: None, message: format!("void expression as argument to {namespace}.{method_name}"),
                })?;

                let expected = param_types.get(i).copied().unwrap_or(LType::I64);
                let converted = self.coerce_to_ltype(val, &self.infer_expr_type(arg), expected)?;
                compiled_args.push(converted.into());
            }

            let call_result = self.builder
                .build_call(function, &compiled_args, if ret_type == LType::Void { "" } else { "ns_call" })
                .unwrap();

            last_result = match ret_type {
                LType::Void => None,
                _ => call_result.try_as_basic_value().left(),
            };
        }

        Ok(last_result)
    }

    fn coerce_to_ltype(
        &self,
        val: BasicValueEnum<'ctx>,
        from: &Type,
        to: LType,
    ) -> Result<BasicValueEnum<'ctx>, GBasicError> {
        match (from, to) {
            // Int → F64
            (Type::Int, LType::F64) => {
                Ok(self.builder.build_signed_int_to_float(
                    val.into_int_value(), self.context.f64_type(), "itof"
                ).unwrap().into())
            }
            // Float → I64
            (Type::Float, LType::I64) => {
                Ok(self.builder.build_float_to_signed_int(
                    val.into_float_value(), self.context.i64_type(), "ftoi"
                ).unwrap().into())
            }
            _ => Ok(val),
        }
    }

    fn codegen_call(
        &mut self,
        callee: &Expression,
        args: &[Expression],
    ) -> Result<Option<BasicValueEnum<'ctx>>, GBasicError> {
        // Handle method calls on objects: obj.method(args)
        if let Expression::FieldAccess { object, field, .. } = callee {
            // Special: print("text").at(x, y) → draw text on screen
            if field.name == "at" && args.len() == 2 {
                if let Expression::Call { callee: inner_callee, args: print_args, .. } = object.as_ref() {
                    if let Expression::Identifier(id) = inner_callee.as_ref() {
                        if id.name == "print" && print_args.len() == 1 {
                            return self.codegen_print_at(&print_args[0], &args[0], &args[1]);
                        }
                    }
                }
            }
            return self.codegen_object_method(object, &field.name, args);
        }

        if let Expression::Identifier(id) = callee {
            // Layer 1 builtin shortcuts
            match id.name.as_str() {
                "print" if args.len() == 1 => return self.codegen_print(&args[0]),
                "rect" if args.len() == 2 => {
                    let w = self.codegen_expression(&args[0])?.unwrap();
                    let h = self.codegen_expression(&args[1])?.unwrap();
                    let wf = self.coerce_to_ltype(w, &self.infer_expr_type(&args[0]), LType::F64)?;
                    let hf = self.coerce_to_ltype(h, &self.infer_expr_type(&args[1]), LType::F64)?;
                    return Ok(self.call_runtime("runtime_create_rect", &[LType::F64, LType::F64], LType::I64, &[wf.into(), hf.into()]));
                }
                "circle" if args.len() == 1 => {
                    let r = self.codegen_expression(&args[0])?.unwrap();
                    let rf = self.coerce_to_ltype(r, &self.infer_expr_type(&args[0]), LType::F64)?;
                    return Ok(self.call_runtime("runtime_create_circle", &[LType::F64], LType::I64, &[rf.into()]));
                }
                "key" if args.len() == 1 => {
                    // Ensure screen is init (for input polling)
                    self.call_runtime("ensure_screen_init", &[], LType::Void, &[]);
                    let key_val = self.codegen_expression(&args[0])?.unwrap();
                    return Ok(self.call_runtime("runtime_input_key_pressed", &[LType::Ptr], LType::Bool, &[key_val.into()]));
                }
                "play" if args.len() == 1 => {
                    let name = self.codegen_expression(&args[0])?.unwrap();
                    self.call_runtime("runtime_sound_effect_play", &[LType::Ptr], LType::Void, &[name.into()]);
                    return Ok(None);
                }
                "clear" => {
                    self.call_runtime("ensure_screen_init", &[], LType::Void, &[]);
                    if args.len() == 1 {
                        // clear(named_color) or clear(packed_color)
                        let val = self.codegen_expression(&args[0])?.unwrap();
                        // Unpack i64 color: r = (v >> 16) & 0xFF, g = (v >> 8) & 0xFF, b = v & 0xFF
                        let i64_type = self.context.i64_type();
                        let iv = val.into_int_value();
                        let r = self.builder.build_right_shift(iv, i64_type.const_int(16, false), false, "r").unwrap();
                        let g = self.builder.build_right_shift(iv, i64_type.const_int(8, false), false, "g").unwrap();
                        let b = self.builder.build_and(iv, i64_type.const_int(0xFF, false), "b").unwrap();
                        let r = self.builder.build_and(r, i64_type.const_int(0xFF, false), "r").unwrap();
                        let g = self.builder.build_and(g, i64_type.const_int(0xFF, false), "g").unwrap();
                        self.call_runtime("runtime_screen_clear", &[LType::I64, LType::I64, LType::I64], LType::Void, &[r.into(), g.into(), b.into()]);
                    } else if args.len() == 3 {
                        let r = self.codegen_expression(&args[0])?.unwrap();
                        let g = self.codegen_expression(&args[1])?.unwrap();
                        let b = self.codegen_expression(&args[2])?.unwrap();
                        self.call_runtime("runtime_screen_clear", &[LType::I64, LType::I64, LType::I64], LType::Void, &[r.into(), g.into(), b.into()]);
                    }
                    return Ok(None);
                }
                "random" if args.len() == 2 => {
                    let min = self.codegen_expression(&args[0])?.unwrap();
                    let max = self.codegen_expression(&args[1])?.unwrap();
                    return Ok(self.call_runtime("runtime_math_random_range", &[LType::I64, LType::I64], LType::I64, &[min.into(), max.into()]));
                }
                "point" if args.len() == 2 => {
                    // Point(x, y) constructor — pack as two f64 values
                    // For now, just return x as the primary value (used contextually in property setters)
                    // Point is handled specially in property assignment context
                    let x = self.codegen_expression(&args[0])?.unwrap();
                    let _y = self.codegen_expression(&args[1])?.unwrap();
                    // Store both in a temp struct... Actually for MVP, Point() is only meaningful
                    // in assignment context like `obj.position = Point(x, y)`.
                    // When used standalone, just return x (not great but workable).
                    return Ok(Some(x));
                }
                _ => {}
            }

            // Regular function call
            let function = self
                .module
                .get_function(&id.name)
                .ok_or_else(|| GBasicError::CodegenError {
                    span: None, message: format!("undefined function '{}'", id.name),
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
                span: None, message: "only direct function calls supported".into(),
            })
        }
    }

    /// Emit a typed print call. `suffix` is "" for newline-printing, "_part" for no-newline.
    fn emit_typed_print_call(
        &mut self,
        val: Option<BasicValueEnum<'ctx>>,
        ty: &Type,
        suffix: &str,
    ) {
        match ty {
            Type::String | Type::Unknown => {
                let fname = if suffix.is_empty() { "runtime_print" } else { "runtime_print_str_part" };
                let f = self.module.get_function(fname).unwrap();
                if let Some(v) = val {
                    self.builder.build_call(f, &[v.into()], "").unwrap();
                }
            }
            Type::Int => {
                let fname = format!("runtime_print_int{suffix}");
                let f = self.module.get_function(&fname).unwrap();
                self.builder.build_call(f, &[val.unwrap().into()], "").unwrap();
            }
            Type::Float => {
                let fname = format!("runtime_print_float{suffix}");
                let f = self.module.get_function(&fname).unwrap();
                self.builder.build_call(f, &[val.unwrap().into()], "").unwrap();
            }
            Type::Bool => {
                let bool_val = val.unwrap().into_int_value();
                let i64_val = self.builder.build_int_z_extend(bool_val, self.context.i64_type(), "bool_ext").unwrap();
                let fname = format!("runtime_print_int{suffix}");
                let f = self.module.get_function(&fname).unwrap();
                self.builder.build_call(f, &[i64_val.into()], "").unwrap();
            }
            _ => {
                let fname = if suffix.is_empty() { "runtime_print" } else { "runtime_print_str_part" };
                let f = self.module.get_function(fname).unwrap();
                if let Some(v) = val {
                    self.builder.build_call(f, &[v.into()], "").unwrap();
                }
            }
        }
    }

    /// Emit string interpolation parts (no trailing newline).
    fn emit_string_interp_parts(&mut self, parts: &[StringPart]) -> Result<(), GBasicError> {
        for part in parts {
            match part {
                StringPart::Lit(s) => {
                    let global = self.builder.build_global_string_ptr(s, "str_part").unwrap();
                    let f = self.module.get_function("runtime_print_str_part").unwrap();
                    self.builder.build_call(f, &[global.as_pointer_value().into()], "").unwrap();
                }
                StringPart::Expr(e) => {
                    let ty = self.infer_expr_type(e);
                    let val = self.codegen_expression(e)?;
                    self.emit_typed_print_call(val, &ty, "_part");
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
            self.emit_string_interp_parts(parts)?;
            let newline_fn = self.module.get_function("runtime_print_newline").unwrap();
            self.builder.build_call(newline_fn, &[], "").unwrap();
            return Ok(None);
        }

        let arg_ty = self.infer_expr_type(arg);
        let val = self.codegen_expression(arg)?;
        self.emit_typed_print_call(val, &arg_ty, "");
        Ok(None)
    }

    /// print("text").at(x, y) → render text on screen at position
    fn codegen_print_at(
        &mut self,
        text_arg: &Expression,
        x_arg: &Expression,
        y_arg: &Expression,
    ) -> Result<Option<BasicValueEnum<'ctx>>, GBasicError> {
        // Build the string to display
        let text_ptr = if let Expression::StringInterp { parts, .. } = text_arg {
            // Build concatenated string via runtime_string_concat
            self.build_interp_string(parts)?
        } else {
            let val = self.codegen_expression(text_arg)?;
            match self.infer_expr_type(text_arg) {
                Type::String | Type::Unknown => val.unwrap(),
                _ => {
                    // Convert to string representation — for MVP just print int/float to stdout
                    // and return empty string for screen
                    let global = self.builder.build_global_string_ptr("", "empty").unwrap();
                    global.as_pointer_value().into()
                }
            }
        };

        let x = self.codegen_expression(x_arg)?.unwrap();
        let y = self.codegen_expression(y_arg)?.unwrap();
        let xi = self.coerce_to_ltype(x, &self.infer_expr_type(x_arg), LType::I64)?;
        let yi = self.coerce_to_ltype(y, &self.infer_expr_type(y_arg), LType::I64)?;

        // Draw white text by default
        let i64_type = self.context.i64_type();
        let white = i64_type.const_int(255, false);
        self.call_runtime(
            "runtime_draw_text",
            &[LType::Ptr, LType::I64, LType::I64, LType::I64, LType::I64, LType::I64],
            LType::Void,
            &[text_ptr.into(), xi.into(), yi.into(), white.into(), white.into(), white.into()],
        );
        Ok(None)
    }

    /// Build an interpolated string into a single runtime string pointer.
    fn build_interp_string(
        &mut self,
        parts: &[StringPart],
    ) -> Result<BasicValueEnum<'ctx>, GBasicError> {
        let concat_fn = self.module.get_function("runtime_string_concat").unwrap();
        let mut result: Option<BasicValueEnum<'ctx>> = None;

        for part in parts {
            let part_str = match part {
                StringPart::Lit(s) => {
                    let global = self.builder.build_global_string_ptr(s, "str_part").unwrap();
                    global.as_pointer_value().into()
                }
                StringPart::Expr(e) => {
                    let ty = self.infer_expr_type(e);
                    let val = self.codegen_expression(e)?;
                    match ty {
                        Type::String | Type::Unknown => val.unwrap(),
                        Type::Int => {
                            // Convert int to string via snprintf-like approach
                            // For MVP, use a runtime helper or just format
                            // Actually, let's use runtime_int_to_str
                            let v = val.unwrap();
                            self.call_runtime("runtime_int_to_str", &[LType::I64], LType::Ptr, &[v.into()])
                                .unwrap_or_else(|| {
                                    self.builder.build_global_string_ptr("?", "fallback").unwrap().as_pointer_value().into()
                                })
                        }
                        Type::Float => {
                            let v = val.unwrap();
                            self.call_runtime("runtime_float_to_str", &[LType::F64], LType::Ptr, &[v.into()])
                                .unwrap_or_else(|| {
                                    self.builder.build_global_string_ptr("?", "fallback").unwrap().as_pointer_value().into()
                                })
                        }
                        _ => {
                            self.builder.build_global_string_ptr("?", "fallback").unwrap().as_pointer_value().into()
                        }
                    }
                }
            };

            result = Some(match result {
                None => part_str,
                Some(prev) => {
                    self.builder.build_call(concat_fn, &[prev.into(), part_str.into()], "concat")
                        .unwrap()
                        .try_as_basic_value()
                        .left()
                        .unwrap()
                }
            });
        }

        Ok(result.unwrap_or_else(|| {
            self.builder.build_global_string_ptr("", "empty").unwrap().as_pointer_value().into()
        }))
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
            BinaryOp::And => {
                let l1 = self.ensure_i1(lv);
                let r1 = self.ensure_i1(rv);
                self.builder.build_and(l1, r1, "and").unwrap().into()
            }
            BinaryOp::Or => {
                let l1 = self.ensure_i1(lv);
                let r1 = self.ensure_i1(rv);
                self.builder.build_or(l1, r1, "or").unwrap().into()
            }
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
                    span: None, message: format!("unsupported float op: {op}"),
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
                // Named colors are Int (packed RGB)
                if named_color(&id.name).is_some() {
                    return Type::Int;
                }
                self.lookup_var(&id.name)
                    .map(|v| v.ty.clone())
                    .unwrap_or(Type::Unknown)
            }
            Expression::BinaryOp { left, op, right, .. } => {
                match op {
                    BinaryOp::Eq | BinaryOp::Neq | BinaryOp::Lt | BinaryOp::Gt
                    | BinaryOp::Le | BinaryOp::Ge | BinaryOp::And | BinaryOp::Or => Type::Bool,
                    _ => {
                        let lt = self.infer_expr_type(left);
                        let rt = self.infer_expr_type(right);
                        if matches!(lt, Type::String) {
                            Type::String
                        } else if matches!((&lt, &rt), (Type::Int, Type::Float) | (Type::Float, Type::Int)) {
                            Type::Float
                        } else {
                            lt
                        }
                    }
                }
            }
            Expression::UnaryOp { op, operand, .. } => match op {
                UnaryOp::Not => Type::Bool,
                UnaryOp::Neg => self.infer_expr_type(operand),
            },
            Expression::Call { callee, .. } => {
                if let Expression::Identifier(id) = callee.as_ref() {
                    match id.name.as_str() {
                        "print" | "play" | "clear" => return Type::Void,
                        "rect" | "circle" => return Type::Int, // handle is i64
                        "key" => return Type::Bool,
                        "random" => return Type::Int,
                        "point" => return Type::Float, // MVP: Point returns float-ish
                        _ => {}
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
                // Method call on object: check known return types
                if let Expression::FieldAccess { field, .. } = callee.as_ref() {
                    match field.name.as_str() {
                        "collides" | "contains" => return Type::Bool,
                        "move" | "remove" | "add" | "at" => return Type::Void,
                        _ => {}
                    }
                }
                Type::Unknown
            }
            Expression::StringInterp { .. } => Type::String,
            Expression::Assignment { value, .. } => self.infer_expr_type(value),
            Expression::MethodChain { base, chain, .. } => {
                if let Some(last) = chain.last() {
                    // Screen properties
                    if *base == NamespaceRef::Screen {
                        match last.method.name.as_str() {
                            "center" | "bottom_center" | "top_center" => return Type::Float,
                            _ => {}
                        }
                    }
                    if let Some(entry) = get_namespace_method(*base, &last.method.name) {
                        return entry.ret.to_gbasic_type();
                    }
                }
                Type::Unknown
            }
            Expression::Array { elements, .. } => {
                if let Some(first) = elements.first() {
                    Type::Array(Box::new(self.infer_expr_type(first)))
                } else {
                    Type::Array(Box::new(Type::Unknown))
                }
            }
            Expression::Index { object, .. } => {
                match self.infer_expr_type(object) {
                    Type::Array(inner) => *inner,
                    _ => Type::Unknown,
                }
            }
            Expression::Range { .. } => Type::Unknown,
            Expression::FieldAccess { .. } => {
                if let Some((var_name, prop_path)) = resolve_field_chain(expr) {
                    if var_name == "screen" {
                        return match prop_path.as_str() {
                            "width" | "height" => Type::Int,
                            "center.x" | "center.y" => Type::Float,
                            _ => Type::Unknown,
                        };
                    }
                    return match prop_path.as_str() {
                        "position.x" | "position.y" | "velocity.x" | "velocity.y"
                        | "size.width" | "size.height" | "x" | "y" => Type::Float,
                        _ => Type::Unknown,
                    };
                }
                Type::Unknown
            }
        }
    }

    fn build_alloca_for_type(
        &self,
        ty: &Type,
        name: &str,
    ) -> PointerValue<'ctx> {
        self.builder.build_alloca(self.type_to_llvm_basic(ty), name).unwrap()
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
        self.type_to_llvm_basic(ty).into()
    }

    // ─── Object property setter ───

    fn codegen_property_set(
        &mut self,
        handle: BasicValueEnum<'ctx>,
        prop_path: &str,
        value: &Expression,
        span: Span,
    ) -> Result<Option<BasicValueEnum<'ctx>>, GBasicError> {
        let h: BasicMetadataValueEnum = handle.into();

        match prop_path {
            "position" => {
                // Value should be Point(x, y) call or Screen.center etc.
                if let Expression::Call { callee, args, .. } = value {
                    if let Expression::Identifier(id) = callee.as_ref() {
                        if id.name == "point" && args.len() == 2 {
                            let x = self.codegen_expression(&args[0])?.unwrap();
                            let y = self.codegen_expression(&args[1])?.unwrap();
                            let xf = self.coerce_to_ltype(x, &self.infer_expr_type(&args[0]), LType::F64)?;
                            let yf = self.coerce_to_ltype(y, &self.infer_expr_type(&args[1]), LType::F64)?;
                            self.call_runtime("runtime_set_position", &[LType::I64, LType::F64, LType::F64], LType::Void, &[h, xf.into(), yf.into()]);
                            return Ok(None);
                        }
                    }
                }
                // Handle Screen.center, Screen.bottom_center, etc. as position values
                if let Expression::MethodChain { base, chain, .. } = value {
                    if *base == NamespaceRef::Screen {
                        if let Some(last) = chain.last() {
                            let f64_type = self.context.f64_type();
                            let zero = f64_type.const_float(0.0);
                            let (px, py) = match last.method.name.as_str() {
                                "center" => {
                                    let cx = self.call_runtime("runtime_screen_center_x", &[], LType::F64, &[]).unwrap();
                                    let cy = self.call_runtime("runtime_screen_center_y", &[], LType::F64, &[]).unwrap();
                                    (cx, cy)
                                }
                                "bottom_center" => {
                                    let cx = self.call_runtime("runtime_screen_center_x", &[], LType::F64, &[]).unwrap();
                                    let sh = self.call_runtime("runtime_screen_height", &[], LType::I64, &[]).unwrap();
                                    let shy = self.builder.build_signed_int_to_float(sh.into_int_value(), f64_type, "sh").unwrap();
                                    (cx, shy.into())
                                }
                                "top_center" => {
                                    let cx = self.call_runtime("runtime_screen_center_x", &[], LType::F64, &[]).unwrap();
                                    (cx, zero.into())
                                }
                                "top_left" => (zero.into(), zero.into()),
                                "top_right" => {
                                    let sw = self.call_runtime("runtime_screen_width", &[], LType::I64, &[]).unwrap();
                                    let swf = self.builder.build_signed_int_to_float(sw.into_int_value(), f64_type, "sw").unwrap();
                                    (swf.into(), zero.into())
                                }
                                "bottom_left" => {
                                    let sh = self.call_runtime("runtime_screen_height", &[], LType::I64, &[]).unwrap();
                                    let shy = self.builder.build_signed_int_to_float(sh.into_int_value(), f64_type, "sh").unwrap();
                                    (zero.into(), shy.into())
                                }
                                "bottom_right" => {
                                    let sw = self.call_runtime("runtime_screen_width", &[], LType::I64, &[]).unwrap();
                                    let sh = self.call_runtime("runtime_screen_height", &[], LType::I64, &[]).unwrap();
                                    let swf = self.builder.build_signed_int_to_float(sw.into_int_value(), f64_type, "sw").unwrap();
                                    let shy = self.builder.build_signed_int_to_float(sh.into_int_value(), f64_type, "sh").unwrap();
                                    (swf.into(), shy.into())
                                }
                                _ => {
                                    return Err(GBasicError::CodegenError {
                                        span: Some(span), message: format!("unknown Screen property '{}'", last.method.name),
                                    });
                                }
                            };
                            self.call_runtime("runtime_set_position", &[LType::I64, LType::F64, LType::F64], LType::Void, &[h, px.into(), py.into()]);
                            return Ok(None);
                        }
                    }
                }
                Err(GBasicError::CodegenError {
                    span: Some(span), message: "unsupported value for .position assignment; use Point(x, y) or Screen.center".into(),
                })
            }
            "position.x" => {
                let val = self.codegen_expression(value)?.unwrap();
                let vf = self.coerce_to_ltype(val, &self.infer_expr_type(value), LType::F64)?;
                self.call_runtime("runtime_set_position_x", &[LType::I64, LType::F64], LType::Void, &[h, vf.into()]);
                Ok(None)
            }
            "position.y" => {
                let val = self.codegen_expression(value)?.unwrap();
                let vf = self.coerce_to_ltype(val, &self.infer_expr_type(value), LType::F64)?;
                self.call_runtime("runtime_set_position_y", &[LType::I64, LType::F64], LType::Void, &[h, vf.into()]);
                Ok(None)
            }
            "color" => {
                // Handle named color identifier or Color(r, g, b)
                if let Expression::Identifier(id) = value {
                    if let Some((r, g, b)) = named_color(&id.name) {
                        let ri = self.context.i64_type().const_int(r as u64, false);
                        let gi = self.context.i64_type().const_int(g as u64, false);
                        let bi = self.context.i64_type().const_int(b as u64, false);
                        self.call_runtime("runtime_set_color", &[LType::I64, LType::I64, LType::I64, LType::I64], LType::Void, &[h, ri.into(), gi.into(), bi.into()]);
                        return Ok(None);
                    }
                }
                if let Expression::Call { callee, args, .. } = value {
                    if let Expression::Identifier(id) = callee.as_ref() {
                        if id.name == "color" && args.len() == 3 {
                            let r = self.codegen_expression(&args[0])?.unwrap();
                            let g = self.codegen_expression(&args[1])?.unwrap();
                            let b = self.codegen_expression(&args[2])?.unwrap();
                            self.call_runtime("runtime_set_color", &[LType::I64, LType::I64, LType::I64, LType::I64], LType::Void, &[h, r.into(), g.into(), b.into()]);
                            return Ok(None);
                        }
                    }
                }
                // Packed color (from named color used as expression)
                let val = self.codegen_expression(value)?.unwrap();
                let i64_type = self.context.i64_type();
                let iv = val.into_int_value();
                let r = self.builder.build_right_shift(iv, i64_type.const_int(16, false), false, "r").unwrap();
                let g = self.builder.build_right_shift(iv, i64_type.const_int(8, false), false, "g").unwrap();
                let b = self.builder.build_and(iv, i64_type.const_int(0xFF, false), "b").unwrap();
                let r = self.builder.build_and(r, i64_type.const_int(0xFF, false), "r").unwrap();
                let g = self.builder.build_and(g, i64_type.const_int(0xFF, false), "g").unwrap();
                self.call_runtime("runtime_set_color", &[LType::I64, LType::I64, LType::I64, LType::I64], LType::Void, &[h, r.into(), g.into(), b.into()]);
                Ok(None)
            }
            "velocity" => {
                // velocity = Point(vx, vy)
                if let Expression::Call { callee, args, .. } = value {
                    if let Expression::Identifier(id) = callee.as_ref() {
                        if id.name == "point" && args.len() == 2 {
                            let vx = self.codegen_expression(&args[0])?.unwrap();
                            let vy = self.codegen_expression(&args[1])?.unwrap();
                            let vxf = self.coerce_to_ltype(vx, &self.infer_expr_type(&args[0]), LType::F64)?;
                            let vyf = self.coerce_to_ltype(vy, &self.infer_expr_type(&args[1]), LType::F64)?;
                            self.call_runtime("runtime_set_velocity", &[LType::I64, LType::F64, LType::F64], LType::Void, &[h, vxf.into(), vyf.into()]);
                            return Ok(None);
                        }
                    }
                }
                Err(GBasicError::CodegenError {
                    span: Some(span), message: "unsupported value for .velocity assignment; use Point(vx, vy)".into(),
                })
            }
            "velocity.x" => {
                let val = self.codegen_expression(value)?.unwrap();
                let vf = self.coerce_to_ltype(val, &self.infer_expr_type(value), LType::F64)?;
                self.call_runtime("runtime_set_velocity_x", &[LType::I64, LType::F64], LType::Void, &[h, vf.into()]);
                Ok(None)
            }
            "velocity.y" => {
                let val = self.codegen_expression(value)?.unwrap();
                let vf = self.coerce_to_ltype(val, &self.infer_expr_type(value), LType::F64)?;
                self.call_runtime("runtime_set_velocity_y", &[LType::I64, LType::F64], LType::Void, &[h, vf.into()]);
                Ok(None)
            }
            "gravity" => {
                let val = self.codegen_expression(value)?.unwrap();
                let vf = self.coerce_to_ltype(val, &self.infer_expr_type(value), LType::F64)?;
                self.call_runtime("runtime_set_gravity", &[LType::I64, LType::F64], LType::Void, &[h, vf.into()]);
                Ok(None)
            }
            "solid" => {
                let val = self.codegen_expression(value)?.unwrap();
                // Convert bool (i1) to i64
                let iv = val.into_int_value();
                let i64_val = self.builder.build_int_z_extend(iv, self.context.i64_type(), "bool_ext").unwrap();
                self.call_runtime("runtime_set_solid", &[LType::I64, LType::I64], LType::Void, &[h, i64_val.into()]);
                Ok(None)
            }
            "bounces" => {
                let val = self.codegen_expression(value)?.unwrap();
                let iv = val.into_int_value();
                let i64_val = self.builder.build_int_z_extend(iv, self.context.i64_type(), "bool_ext").unwrap();
                self.call_runtime("runtime_set_bounces", &[LType::I64, LType::I64], LType::Void, &[h, i64_val.into()]);
                Ok(None)
            }
            "visible" => {
                let val = self.codegen_expression(value)?.unwrap();
                let iv = val.into_int_value();
                let i64_val = self.builder.build_int_z_extend(iv, self.context.i64_type(), "bool_ext").unwrap();
                self.call_runtime("runtime_set_visible", &[LType::I64, LType::I64], LType::Void, &[h, i64_val.into()]);
                Ok(None)
            }
            "layer" => {
                let val = self.codegen_expression(value)?.unwrap();
                self.call_runtime("runtime_set_layer", &[LType::I64, LType::I64], LType::Void, &[h, val.into()]);
                Ok(None)
            }
            _ => Err(GBasicError::CodegenError {
                span: Some(span), message: format!("unknown property '{prop_path}' for assignment"),
            }),
        }
    }

    // ─── Object property getter ───

    fn codegen_field_access_read(
        &mut self,
        expr: &Expression,
    ) -> Result<Option<BasicValueEnum<'ctx>>, GBasicError> {
        // Handle MethodChain.field (e.g. Screen.center.y)
        if let Expression::FieldAccess { object, field, .. } = expr {
            if let Expression::MethodChain { base, chain, .. } = object.as_ref() {
                if *base == NamespaceRef::Screen {
                    if let Some(last) = chain.last() {
                        self.call_runtime("ensure_screen_init", &[], LType::Void, &[]);
                        match (last.method.name.as_str(), field.name.as_str()) {
                            ("center", "x") => return Ok(self.call_runtime("runtime_screen_center_x", &[], LType::F64, &[])),
                            ("center", "y") => return Ok(self.call_runtime("runtime_screen_center_y", &[], LType::F64, &[])),
                            ("bottom_center", "x") | ("top_center", "x") => return Ok(self.call_runtime("runtime_screen_center_x", &[], LType::F64, &[])),
                            ("bottom_center", "y") | ("bottom_left", "y") | ("bottom_right", "y") => {
                                let h = self.call_runtime("runtime_screen_height", &[], LType::I64, &[]).unwrap();
                                let hf = self.builder.build_signed_int_to_float(h.into_int_value(), self.context.f64_type(), "hf").unwrap();
                                return Ok(Some(hf.into()));
                            }
                            ("top_left", "x") | ("top_center", "y") | ("top_left", "y") | ("bottom_left", "x") => {
                                return Ok(Some(self.context.f64_type().const_float(0.0).into()));
                            }
                            ("top_right", "x") | ("bottom_right", "x") | ("top_right", "y") => {
                                if field.name.as_str() == "x" {
                                    let w = self.call_runtime("runtime_screen_width", &[], LType::I64, &[]).unwrap();
                                    let wf = self.builder.build_signed_int_to_float(w.into_int_value(), self.context.f64_type(), "wf").unwrap();
                                    return Ok(Some(wf.into()));
                                }
                                return Ok(Some(self.context.f64_type().const_float(0.0).into()));
                            }
                            _ => {}
                        }
                    }
                }
            }
        }

        if let Some((var_name, prop_path)) = resolve_field_chain(expr) {
            // Handle Screen.width, Screen.height as special cases
            if var_name == "screen" {
                return match prop_path.as_str() {
                    "width" => {
                        self.call_runtime("ensure_screen_init", &[], LType::Void, &[]);
                        let w = self.call_runtime("runtime_screen_width", &[], LType::I64, &[]);
                        Ok(w)
                    }
                    "height" => {
                        self.call_runtime("ensure_screen_init", &[], LType::Void, &[]);
                        let h = self.call_runtime("runtime_screen_height", &[], LType::I64, &[]);
                        Ok(h)
                    }
                    "center.x" => {
                        self.call_runtime("ensure_screen_init", &[], LType::Void, &[]);
                        Ok(self.call_runtime("runtime_screen_center_x", &[], LType::F64, &[]))
                    }
                    "center.y" => {
                        self.call_runtime("ensure_screen_init", &[], LType::Void, &[]);
                        Ok(self.call_runtime("runtime_screen_center_y", &[], LType::F64, &[]))
                    }
                    _ => {
                        let null = self.context.ptr_type(inkwell::AddressSpace::default()).const_null();
                        Ok(Some(null.into()))
                    }
                };
            }

            // Regular object property read
            if let Some(var) = self.lookup_var(&var_name) {
                let handle_ty = self.type_to_llvm_basic(&var.ty);
                let ptr = var.ptr;
                let handle = self.builder.build_load(handle_ty, ptr, "handle").unwrap();
                let h: BasicMetadataValueEnum = handle.into();

                return match prop_path.as_str() {
                    "position.x" | "x" => Ok(self.call_runtime("runtime_get_position_x", &[LType::I64], LType::F64, &[h])),
                    "position.y" | "y" => Ok(self.call_runtime("runtime_get_position_y", &[LType::I64], LType::F64, &[h])),
                    "velocity.x" => Ok(self.call_runtime("runtime_get_velocity_x", &[LType::I64], LType::F64, &[h])),
                    "velocity.y" => Ok(self.call_runtime("runtime_get_velocity_y", &[LType::I64], LType::F64, &[h])),
                    "size.width" => Ok(self.call_runtime("runtime_get_size_width", &[LType::I64], LType::F64, &[h])),
                    "size.height" => Ok(self.call_runtime("runtime_get_size_height", &[LType::I64], LType::F64, &[h])),
                    "length" => Ok(self.call_runtime("runtime_array_length", &[LType::I64], LType::I64, &[h])),
                    _ => {
                        let null = self.context.ptr_type(inkwell::AddressSpace::default()).const_null();
                        Ok(Some(null.into()))
                    }
                };
            }
        }

        // Unresolvable — return null pointer as fallback
        let null = self.context.ptr_type(inkwell::AddressSpace::default()).const_null();
        Ok(Some(null.into()))
    }

    // ─── Object method call ───

    fn codegen_object_method(
        &mut self,
        object: &Expression,
        method: &str,
        args: &[Expression],
    ) -> Result<Option<BasicValueEnum<'ctx>>, GBasicError> {
        let obj_val = self.codegen_expression(object)?.unwrap();
        let h: BasicMetadataValueEnum = obj_val.into();

        match method {
            "move" if args.len() == 2 => {
                let dx = self.codegen_expression(&args[0])?.unwrap();
                let dy = self.codegen_expression(&args[1])?.unwrap();
                let dxf = self.coerce_to_ltype(dx, &self.infer_expr_type(&args[0]), LType::F64)?;
                let dyf = self.coerce_to_ltype(dy, &self.infer_expr_type(&args[1]), LType::F64)?;
                self.call_runtime("runtime_object_move", &[LType::I64, LType::F64, LType::F64], LType::Void, &[h, dxf.into(), dyf.into()]);
                Ok(None)
            }
            "collides" if args.len() == 1 => {
                let other = self.codegen_expression(&args[0])?.unwrap();
                let result = self.call_runtime("runtime_object_collides", &[LType::I64, LType::I64], LType::Bool, &[h, other.into()]);
                Ok(result)
            }
            "contains" if args.len() == 2 => {
                let x = self.codegen_expression(&args[0])?.unwrap();
                let y = self.codegen_expression(&args[1])?.unwrap();
                let xf = self.coerce_to_ltype(x, &self.infer_expr_type(&args[0]), LType::F64)?;
                let yf = self.coerce_to_ltype(y, &self.infer_expr_type(&args[1]), LType::F64)?;
                let result = self.call_runtime("runtime_object_contains", &[LType::I64, LType::F64, LType::F64], LType::Bool, &[h, xf.into(), yf.into()]);
                Ok(result)
            }
            "remove" => {
                self.call_runtime("runtime_object_remove", &[LType::I64], LType::Void, &[h]);
                Ok(None)
            }
            "add" if args.len() == 1 => {
                let val = self.codegen_expression(&args[0])?.unwrap();
                self.call_runtime("runtime_array_add", &[LType::I64, LType::I64], LType::Void, &[h, val.into()]);
                Ok(None)
            }
            "remove_from" if args.len() == 1 => {
                let val = self.codegen_expression(&args[0])?.unwrap();
                self.call_runtime("runtime_array_remove_value", &[LType::I64, LType::I64], LType::Void, &[h, val.into()]);
                Ok(None)
            }
            "at" if args.len() == 2 => {
                // print("...").at(x, y) — positioned text on screen
                // `object` here is from print(), we need to intercept
                // and emit runtime_draw_text instead. We handle this by
                // storing the last print string and drawing it.
                let x = self.codegen_expression(&args[0])?.unwrap();
                let y = self.codegen_expression(&args[1])?.unwrap();
                // The print call already emitted text to stdout. For .at(),
                // we need the string. Re-evaluate if object was a print call.
                // For now, draw the last-printed string at position.
                // This is handled via a special path in codegen_call for print().at()
                let _ = x;
                let _ = y;
                Ok(None)
            }
            _ => {
                Err(GBasicError::CodegenError {
                    span: None, message: format!("unknown object method '.{method}()'"),
                })
            }
        }
    }

    fn emit_and_link(&self, output_path: &str) -> Result<(), GBasicError> {
        Target::initialize_native(&InitializationConfig::default()).map_err(|e| {
            GBasicError::CodegenError {
                span: None, message: format!("failed to init native target: {e}"),
            }
        })?;

        let triple = TargetMachine::get_default_triple();
        let target = Target::from_triple(&triple).map_err(|e| GBasicError::CodegenError {
            span: None, message: format!("failed to get target: {e}"),
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
                span: None, message: "failed to create target machine".into(),
            })?;

        let obj_path_str = format!("{output_path}.o");
        let obj_path = Path::new(&obj_path_str);
        machine
            .write_to_file(&self.module, FileType::Object, obj_path)
            .map_err(|e| GBasicError::CodegenError {
                span: None, message: format!("failed to write object file: {e}"),
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
            span: None, message: format!("failed to run linker: {e}"),
        })?;

        if !status.success() {
            return Err(GBasicError::CodegenError {
                span: None, message: format!("linking failed with status: {status}"),
            });
        }

        // Clean up object file
        let _ = std::fs::remove_file(&obj_path_str);

        Ok(())
    }
}
