//! Layer 1 shortcut alias definitions.
//!
//! These are the "beginner-friendly" function names that desugar to namespace method chains.
//! Desugaring happens in the parser (`parse_postfix`) — this table is the single source of truth.
//! Typechecker and codegen only see canonical `MethodChain` AST nodes.

/// A shortcut alias definition.
pub struct ShortcutDef {
    /// The shortcut function name (e.g. "print")
    pub name: &'static str,
    /// The full namespace (e.g. "Screen")
    pub namespace: &'static str,
    /// The prefix chain that gets prepended (e.g. "Layer(0).Print")
    pub prefix_chain: &'static str,
    /// Human-readable description
    pub description: &'static str,
}

/// All Layer 1 shortcuts. Parser desugars calls matching these names into `MethodChain` nodes.
pub static SHORTCUTS: &[ShortcutDef] = &[
    ShortcutDef {
        name: "print",
        namespace: "Screen",
        prefix_chain: "Layer(0).Print",
        description: "Print text to stdout or screen",
    },
    ShortcutDef {
        name: "clear",
        namespace: "Screen",
        prefix_chain: "Layer(0).Clear",
        description: "Clear the screen with a color",
    },
    ShortcutDef {
        name: "random",
        namespace: "Math",
        prefix_chain: "Random",
        description: "Generate a random number",
    },
    ShortcutDef {
        name: "abs",
        namespace: "Math",
        prefix_chain: "Abs",
        description: "Absolute value",
    },
    ShortcutDef {
        name: "sqrt",
        namespace: "Math",
        prefix_chain: "Sqrt",
        description: "Square root",
    },
    ShortcutDef {
        name: "sin",
        namespace: "Math",
        prefix_chain: "Sin",
        description: "Sine function",
    },
    ShortcutDef {
        name: "cos",
        namespace: "Math",
        prefix_chain: "Cos",
        description: "Cosine function",
    },
    ShortcutDef {
        name: "key",
        namespace: "Input",
        prefix_chain: "Keyboard.Key",
        description: "Check if a key is pressed",
    },
    ShortcutDef {
        name: "sprite",
        namespace: "Asset",
        prefix_chain: "Sprite",
        description: "Load a bundled or project sprite by name",
    },
    ShortcutDef {
        name: "play",
        namespace: "Sound",
        prefix_chain: "Effect.Play",
        description: "Play a sound effect",
    },
    ShortcutDef {
        name: "log",
        namespace: "System",
        prefix_chain: "Log",
        description: "Log a debug message",
    },
    ShortcutDef {
        name: "wait",
        namespace: "System",
        prefix_chain: "Wait",
        description: "Sleep for a number of seconds",
    },
    ShortcutDef {
        name: "clamp",
        namespace: "Math",
        prefix_chain: "Clamp",
        description: "Clamp a value between min and max",
    },
    ShortcutDef {
        name: "line",
        namespace: "Screen",
        prefix_chain: "Layer(0).Line",
        description: "Draw a line between two points",
    },
];

/// Look up a shortcut by name.
pub fn lookup_shortcut(name: &str) -> Option<&'static ShortcutDef> {
    SHORTCUTS.iter().find(|s| s.name == name)
}
