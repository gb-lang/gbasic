use crate::span::Span;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum GBasicError {
    #[error("Syntax error: {message}")]
    SyntaxError { message: String, span: Span },

    #[error("Type error: {message}")]
    TypeError { message: String, span: Span },

    #[error("Name error: {message}")]
    NameError { message: String, span: Span },

    #[error("Codegen error: {message}")]
    CodegenError { message: String, span: Option<Span> },

    #[error("Internal compiler error: {message}")]
    InternalError { message: String },
}

impl GBasicError {
    pub fn span(&self) -> Option<Span> {
        match self {
            GBasicError::SyntaxError { span, .. }
            | GBasicError::TypeError { span, .. }
            | GBasicError::NameError { span, .. } => Some(*span),
            GBasicError::CodegenError { span, .. } => *span,
            _ => None,
        }
    }
}
