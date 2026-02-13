mod symbol_table;

use gbasic_common::ast::*;
use gbasic_common::error::GBasicError;
use gbasic_common::span::Span;
use gbasic_common::types::Type;
use symbol_table::{Symbol, SymbolTable};

pub fn check(program: &Program) -> Result<(), GBasicError> {
    let mut checker = TypeChecker::new();
    checker.register_builtins();
    for stmt in &program.statements {
        checker.check_statement(stmt)?;
    }
    Ok(())
}

struct TypeChecker {
    symbols: SymbolTable,
}

impl TypeChecker {
    fn new() -> Self {
        Self {
            symbols: SymbolTable::new(),
        }
    }

    fn register_builtins(&mut self) {
        // print accepts any single argument (lenient for week 1)
        self.symbols.insert(
            "print".into(),
            Symbol {
                ty: Type::Function {
                    params: vec![Type::Unknown],
                    ret: Box::new(Type::Void),
                },
                mutable: false,
            },
        );
        // Layer 1 shortcuts
        let builtins: &[(&str, Vec<Type>, Type)] = &[
            ("rect", vec![Type::Unknown, Type::Unknown], Type::Int),
            ("circle", vec![Type::Unknown], Type::Int),
            ("key", vec![Type::String], Type::Bool),
            ("play", vec![Type::String], Type::Void),
            ("clear", vec![Type::Unknown], Type::Void),
            ("random", vec![Type::Int, Type::Int], Type::Int),
            ("point", vec![Type::Unknown, Type::Unknown], Type::Unknown),
            ("color", vec![Type::Int, Type::Int, Type::Int], Type::Unknown),
        ];
        for (name, params, ret) in builtins {
            self.symbols.insert(
                (*name).into(),
                Symbol {
                    ty: Type::Function {
                        params: params.clone(),
                        ret: Box::new(ret.clone()),
                    },
                    mutable: false,
                },
            );
        }
        // Named colors as global constants
        for color in &[
            "black", "white", "red", "green", "blue", "yellow",
            "orange", "purple", "pink", "cyan", "gray", "grey", "brown",
        ] {
            self.symbols.insert(
                (*color).into(),
                Symbol { ty: Type::Int, mutable: false },
            );
        }
    }

    fn check_statement(&mut self, stmt: &Statement) -> Result<(), GBasicError> {
        match stmt {
            Statement::Let {
                name,
                type_ann,
                value,
                span,
            } => {
                let val_ty = self.check_expression(value)?;
                let ty = if let Some(ann) = type_ann {
                    if !Self::types_compatible(ann, &val_ty) {
                        return Err(GBasicError::TypeError {
                            message: format!(
                                "type mismatch: expected {ann}, found {val_ty}"
                            ),
                            span: *span,
                        });
                    }
                    ann.clone()
                } else {
                    val_ty
                };
                self.symbols.insert(
                    name.name.clone(),
                    Symbol { ty, mutable: true },
                );
            }
            Statement::Function(func) => {
                let param_types: Vec<Type> = func
                    .params
                    .iter()
                    .map(|p| p.type_ann.clone().unwrap_or(Type::Unknown))
                    .collect();
                let ret_type = func.return_type.clone().unwrap_or(Type::Void);

                self.symbols.insert(
                    func.name.name.clone(),
                    Symbol {
                        ty: Type::Function {
                            params: param_types.clone(),
                            ret: Box::new(ret_type.clone()),
                        },
                        mutable: false,
                    },
                );

                self.symbols.push_scope();
                for (param, ty) in func.params.iter().zip(param_types.iter()) {
                    self.symbols.insert(
                        param.name.name.clone(),
                        Symbol {
                            ty: ty.clone(),
                            mutable: true,
                        },
                    );
                }
                for s in &func.body.statements {
                    self.check_statement(s)?;
                }
                self.symbols.pop_scope();
            }
            Statement::If {
                condition,
                then_block,
                else_block,
                span,
            } => {
                let cond_ty = self.check_expression(condition)?;
                if !Self::types_compatible(&Type::Bool, &cond_ty) {
                    return Err(GBasicError::TypeError {
                        message: format!(
                            "if condition must be Bool, found {cond_ty}"
                        ),
                        span: *span,
                    });
                }
                self.check_block(then_block)?;
                if let Some(else_b) = else_block {
                    self.check_block(else_b)?;
                }
            }
            Statement::While {
                condition,
                body,
                span,
            } => {
                let cond_ty = self.check_expression(condition)?;
                if !Self::types_compatible(&Type::Bool, &cond_ty) {
                    return Err(GBasicError::TypeError {
                        message: format!(
                            "while condition must be Bool, found {cond_ty}"
                        ),
                        span: *span,
                    });
                }
                self.check_block(body)?;
            }
            Statement::For {
                variable,
                iterable,
                body,
                ..
            } => {
                let iter_ty = self.check_expression(iterable)?;
                let var_ty = match &iter_ty {
                    Type::Array(inner) => *inner.clone(),
                    _ => Type::Int, // Range produces Int
                };
                self.symbols.push_scope();
                self.symbols.insert(
                    variable.name.clone(),
                    Symbol {
                        ty: var_ty,
                        mutable: false,
                    },
                );
                for s in &body.statements {
                    self.check_statement(s)?;
                }
                self.symbols.pop_scope();
            }
            Statement::Return { value, .. } => {
                if let Some(val) = value {
                    self.check_expression(val)?;
                }
            }
            Statement::Expression { expr, .. } => {
                self.check_expression(expr)?;
            }
            Statement::Block(block) => {
                self.check_block(block)?;
            }
            Statement::Match {
                subject, arms, ..
            } => {
                self.check_expression(subject)?;
                for arm in arms {
                    self.check_block(&arm.body)?;
                }
            }
            Statement::Break { .. } | Statement::Continue { .. } => {}
        }
        Ok(())
    }

    fn check_block(&mut self, block: &Block) -> Result<(), GBasicError> {
        self.symbols.push_scope();
        for stmt in &block.statements {
            self.check_statement(stmt)?;
        }
        self.symbols.pop_scope();
        Ok(())
    }

    fn check_expression(&mut self, expr: &Expression) -> Result<Type, GBasicError> {
        match expr {
            Expression::Literal(lit) => Ok(match &lit.kind {
                LiteralKind::Int(_) => Type::Int,
                LiteralKind::Float(_) => Type::Float,
                LiteralKind::String(_) => Type::String,
                LiteralKind::Bool(_) => Type::Bool,
            }),
            Expression::Identifier(id) => {
                self.symbols.lookup(&id.name).map(|s| s.ty.clone()).ok_or(
                    GBasicError::NameError {
                        message: format!("undefined variable '{}'", id.name),
                        span: id.span,
                    },
                )
            }
            Expression::BinaryOp {
                left,
                op,
                right,
                span,
            } => {
                let lt = self.check_expression(left)?;
                let rt = self.check_expression(right)?;
                self.check_binary_op(&lt, op, &rt, *span)
            }
            Expression::UnaryOp {
                op,
                operand,
                span,
            } => {
                let t = self.check_expression(operand)?;
                match op {
                    UnaryOp::Neg => {
                        if matches!(t, Type::Int | Type::Float | Type::Unknown) {
                            Ok(t)
                        } else {
                            Err(GBasicError::TypeError {
                                message: format!("cannot negate {t}"),
                                span: *span,
                            })
                        }
                    }
                    UnaryOp::Not => {
                        if matches!(t, Type::Bool | Type::Unknown) {
                            Ok(Type::Bool)
                        } else {
                            Err(GBasicError::TypeError {
                                message: format!("'not' requires Bool, found {t}"),
                                span: *span,
                            })
                        }
                    }
                }
            }
            Expression::Call {
                callee,
                args,
                span,
            } => {
                let callee_ty = self.check_expression(callee)?;
                match callee_ty {
                    Type::Function { params, ret } => {
                        if params.len() != args.len() {
                            return Err(GBasicError::TypeError {
                                message: format!(
                                    "expected {} argument(s), found {}",
                                    params.len(),
                                    args.len()
                                ),
                                span: *span,
                            });
                        }
                        for (arg, param_ty) in args.iter().zip(params.iter()) {
                            let arg_ty = self.check_expression(arg)?;
                            if !Self::types_compatible(param_ty, &arg_ty) {
                                return Err(GBasicError::TypeError {
                                    message: format!(
                                        "argument type mismatch: expected {param_ty}, found {arg_ty}"
                                    ),
                                    span: arg.span(),
                                });
                            }
                        }
                        Ok(*ret)
                    }
                    Type::Unknown => {
                        for arg in args {
                            self.check_expression(arg)?;
                        }
                        Ok(Type::Unknown)
                    }
                    _ => Err(GBasicError::TypeError {
                        message: format!("'{callee_ty}' is not callable"),
                        span: *span,
                    }),
                }
            }
            Expression::Assignment {
                target,
                value,
                span,
            } => {
                let val_ty = self.check_expression(value)?;
                if let Expression::Identifier(id) = target.as_ref() {
                    let target_ty = self
                        .symbols
                        .lookup(&id.name)
                        .map(|s| s.ty.clone())
                        .ok_or(GBasicError::NameError {
                            message: format!("undefined variable '{}'", id.name),
                            span: id.span,
                        })?;
                    if !Self::types_compatible(&target_ty, &val_ty) {
                        return Err(GBasicError::TypeError {
                            message: format!(
                                "cannot assign {val_ty} to {target_ty}"
                            ),
                            span: *span,
                        });
                    }
                    Ok(target_ty)
                } else {
                    Ok(val_ty)
                }
            }
            Expression::StringInterp { parts, .. } => {
                for part in parts {
                    if let StringPart::Expr(e) = part {
                        self.check_expression(e)?;
                    }
                }
                Ok(Type::String)
            }
            Expression::MethodChain { chain, .. } => {
                for call in chain {
                    for arg in &call.args {
                        self.check_expression(arg)?;
                    }
                }
                Ok(Type::Unknown)
            }
            Expression::Array { elements, .. } => {
                let mut elem_ty = Type::Unknown;
                for el in elements {
                    let t = self.check_expression(el)?;
                    if elem_ty == Type::Unknown {
                        elem_ty = t;
                    }
                }
                Ok(Type::Array(Box::new(elem_ty)))
            }
            Expression::Index { object, index, .. } => {
                self.check_expression(object)?;
                self.check_expression(index)?;
                Ok(Type::Unknown)
            }
            Expression::FieldAccess { object, .. } => {
                self.check_expression(object)?;
                Ok(Type::Unknown)
            }
            Expression::Range { start, end, .. } => {
                self.check_expression(start)?;
                self.check_expression(end)?;
                Ok(Type::Unknown)
            }
        }
    }

    fn check_binary_op(
        &self,
        lt: &Type,
        op: &BinaryOp,
        rt: &Type,
        span: Span,
    ) -> Result<Type, GBasicError> {
        // Unknown unifies with anything
        if matches!(lt, Type::Unknown) || matches!(rt, Type::Unknown) {
            return match op {
                BinaryOp::Eq
                | BinaryOp::Neq
                | BinaryOp::Lt
                | BinaryOp::Gt
                | BinaryOp::Le
                | BinaryOp::Ge
                | BinaryOp::And
                | BinaryOp::Or => Ok(Type::Bool),
                _ => {
                    if *lt == Type::Unknown && *rt == Type::Unknown {
                        Ok(Type::Unknown)
                    } else if *lt == Type::Unknown {
                        Ok(rt.clone())
                    } else {
                        Ok(lt.clone())
                    }
                }
            };
        }

        // Helper: check if one side is Int and the other Float (implicit promotion)
        let is_int_float_mix = |a: &Type, b: &Type| {
            matches!((a, b), (Type::Int, Type::Float) | (Type::Float, Type::Int))
        };

        match op {
            BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul | BinaryOp::Div | BinaryOp::Mod => {
                if lt == rt && matches!(lt, Type::Int | Type::Float) {
                    Ok(lt.clone())
                } else if is_int_float_mix(lt, rt) {
                    Ok(Type::Float)
                } else if matches!(op, BinaryOp::Add)
                    && matches!(lt, Type::String)
                    && matches!(rt, Type::String)
                {
                    Ok(Type::String)
                } else {
                    Err(GBasicError::TypeError {
                        message: format!("cannot apply '{op}' to {lt} and {rt}"),
                        span,
                    })
                }
            }
            BinaryOp::Eq | BinaryOp::Neq | BinaryOp::Lt | BinaryOp::Gt | BinaryOp::Le | BinaryOp::Ge => {
                if lt == rt || is_int_float_mix(lt, rt) {
                    Ok(Type::Bool)
                } else {
                    Err(GBasicError::TypeError {
                        message: format!("cannot compare {lt} and {rt}"),
                        span,
                    })
                }
            }
            BinaryOp::And | BinaryOp::Or => {
                if matches!(lt, Type::Bool) && matches!(rt, Type::Bool) {
                    Ok(Type::Bool)
                } else {
                    Err(GBasicError::TypeError {
                        message: format!("logical '{op}' requires Bool operands, found {lt} and {rt}"),
                        span,
                    })
                }
            }
        }
    }

    fn types_compatible(expected: &Type, actual: &Type) -> bool {
        if matches!(expected, Type::Unknown) || matches!(actual, Type::Unknown) {
            return true;
        }
        expected == actual
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn check_src(src: &str) -> Result<(), GBasicError> {
        let program = gbasic_parser::parse(src).map_err(|e| e.into_iter().next().unwrap())?;
        check(&program)
    }

    #[test]
    fn literal_int() {
        assert!(check_src("let x = 42").is_ok());
    }

    #[test]
    fn literal_float() {
        assert!(check_src("let x = 3.14").is_ok());
    }

    #[test]
    fn literal_string() {
        assert!(check_src("let x = \"hello\"").is_ok());
    }

    #[test]
    fn literal_bool() {
        assert!(check_src("let x = true").is_ok());
    }

    #[test]
    fn arithmetic_same_type() {
        assert!(check_src("let x = 1 + 2").is_ok());
        assert!(check_src("let x = 1.0 * 2.0").is_ok());
    }

    #[test]
    fn arithmetic_int_float_promotion() {
        // Int + Float promotes to Float
        assert!(check_src("let x = 1 + 2.0").is_ok());
        // String + Int is still an error
        assert!(check_src(r#"let x = "a" + 1"#).is_err());
    }

    #[test]
    fn comparison_returns_bool() {
        assert!(check_src("let x = 1 < 2\nif x { let y = 1 }").is_ok());
    }

    #[test]
    fn logical_ops() {
        assert!(check_src("let x = true and false").is_ok());
        assert!(check_src("let x = true or false").is_ok());
    }

    #[test]
    fn undefined_variable() {
        let r = check_src("let x = y");
        assert!(r.is_err());
        let msg = r.unwrap_err().to_string();
        assert!(msg.contains("undefined"));
    }

    #[test]
    fn function_decl_and_call() {
        assert!(check_src("fun add(a: Int, b: Int) -> Int { return a + b }\nlet x = add(1, 2)").is_ok());
    }

    #[test]
    fn print_builtin() {
        assert!(check_src("print(\"hello\")").is_ok());
        assert!(check_src("print(42)").is_ok());
    }

    #[test]
    fn wrong_arg_count() {
        let r = check_src("fun f(a: Int) -> Int { return a }\nf(1, 2)");
        assert!(r.is_err());
        let msg = r.unwrap_err().to_string();
        assert!(msg.contains("argument"));
    }

    #[test]
    fn scoping() {
        // Inner scope variable not visible outside
        let r = check_src("if true { let x = 1 }\nlet y = x");
        assert!(r.is_err());
    }

    #[test]
    fn assignment_type_check() {
        assert!(check_src("let x = 1\nx = 2").is_ok());
    }

    #[test]
    fn for_range_variable_is_int() {
        assert!(check_src("for i in 0..10 { print(i) }").is_ok());
    }

    #[test]
    fn for_array_variable_inferred() {
        assert!(check_src("for x in [1, 2, 3] { print(x) }").is_ok());
    }

    #[test]
    fn string_concat_types() {
        assert!(check_src(r#"let x = "a" + "b""#).is_ok());
        assert!(check_src(r#"let x = "a" + 1"#).is_err());
    }

    #[test]
    fn nested_scopes() {
        // Variable shadowing in nested blocks
        assert!(check_src("let x = 1\nif true { let x = \"hello\" }").is_ok());
    }

    #[test]
    fn match_subject_type() {
        assert!(check_src("match 1 { 1 -> { print(\"one\") } _ -> { print(\"other\") } }").is_ok());
    }

    #[test]
    fn assignment_to_undeclared() {
        let r = check_src("x = 42");
        assert!(r.is_err());
        let msg = r.unwrap_err().to_string();
        assert!(msg.contains("undefined"));
    }

    #[test]
    fn break_continue_in_loop() {
        assert!(check_src("while true { break }").is_ok());
        assert!(check_src("for i in 0..10 { continue }").is_ok());
    }

    #[test]
    fn if_else_if_chain() {
        assert!(check_src("if true { let x = 1 } else { let x = 2 }").is_ok());
    }

    #[test]
    fn method_chain_args_checked() {
        assert!(check_src("Screen.Init(800, 600)").is_ok());
    }

    #[test]
    fn logical_op_non_bool_error() {
        let r = check_src("let x = 1 and 2");
        assert!(r.is_err());
    }
}
