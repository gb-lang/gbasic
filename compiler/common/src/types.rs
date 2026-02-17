use serde::{Deserialize, Serialize};

/// The type system for G-Basic.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Type {
    Int,
    Float,
    String,
    Bool,
    Void,
    /// Point(x, y) — 2D vector value type
    Point,
    /// Color(r, g, b) — packed RGB color value type
    Color,
    Array(Box<Type>),
    /// A function type: (param_types) -> return_type
    Function {
        params: Vec<Type>,
        ret: Box<Type>,
    },
    /// Type not yet resolved (used during type checking)
    Unknown,
}

impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::Int => write!(f, "Int"),
            Type::Float => write!(f, "Float"),
            Type::String => write!(f, "String"),
            Type::Bool => write!(f, "Bool"),
            Type::Void => write!(f, "Void"),
            Type::Point => write!(f, "Point"),
            Type::Color => write!(f, "Color"),
            Type::Array(inner) => write!(f, "[{inner}]"),
            Type::Function { params, ret } => {
                write!(f, "(")?;
                for (i, p) in params.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{p}")?;
                }
                write!(f, ") -> {ret}")
            }
            Type::Unknown => write!(f, "?"),
        }
    }
}
