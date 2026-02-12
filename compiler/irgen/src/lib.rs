#[cfg(feature = "llvm")]
pub mod llvm_backend;

pub fn codegen(
    program: &gbasic_common::ast::Program,
    output_path: &str,
    dump_ir: bool,
) -> Result<(), gbasic_common::error::GBasicError> {
    #[cfg(feature = "llvm")]
    {
        let context = inkwell::context::Context::create();
        llvm_backend::Codegen::compile(&context, program, output_path, dump_ir)
    }
    #[cfg(not(feature = "llvm"))]
    {
        let _ = (program, output_path, dump_ir);
        Err(gbasic_common::error::GBasicError::CodegenError {
            message: "LLVM backend not enabled. Rebuild with --features llvm".into(),
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
