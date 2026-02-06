/// IR generation stub — to be implemented in a later milestone.

pub fn codegen(
    _program: &gbasic_common::ast::Program,
) -> Result<(), gbasic_common::error::GBasicError> {
    // Stub: no-op for now
    Ok(())
}

#[cfg(feature = "llvm")]
pub mod llvm_backend {
    // LLVM backend will be implemented here
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_stub_passes() {
        // Placeholder
    }
}
