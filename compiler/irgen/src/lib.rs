#[cfg(feature = "llvm")]
pub mod llvm_backend;

#[cfg(feature = "llvm")]
pub mod web_glue;

/// Compilation target platform.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompileTarget {
    Desktop,
    Web,
}

pub fn codegen(
    program: &gbasic_common::ast::Program,
    output_path: &str,
    dump_ir: bool,
    target: CompileTarget,
) -> Result<(), gbasic_common::error::GBasicError> {
    #[cfg(feature = "llvm")]
    {
        let context = inkwell::context::Context::create();
        llvm_backend::Codegen::compile(&context, program, output_path, dump_ir, target)
    }
    #[cfg(not(feature = "llvm"))]
    {
        let _ = (program, output_path, dump_ir, target);
        Err(gbasic_common::error::GBasicError::CodegenError {
            message: "LLVM backend not enabled. Rebuild with --features llvm".into(),
            span: None,
        })
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_stub_passes() {
        // Placeholder
    }
}
