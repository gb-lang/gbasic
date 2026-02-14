//! Layer 1 shortcut alias definitions.
//!
//! These are the "beginner-friendly" function names that desugar to namespace method chains.
//! Actual handling is done at the codegen level — this table serves as the single source of
//! truth for documentation, IDE support, and validation.

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

/// All Layer 1 shortcuts. Codegen recognizes these names and emits the appropriate runtime calls.
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
        name: "rect",
        namespace: "Screen",
        prefix_chain: "Layer(0).Rect",
        description: "Create a rectangle game object",
    },
    ShortcutDef {
        name: "circle",
        namespace: "Screen",
        prefix_chain: "Layer(0).Circle",
        description: "Create a circle game object",
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
];

/// Look up a shortcut by name.
pub fn lookup_shortcut(name: &str) -> Option<&'static ShortcutDef> {
    SHORTCUTS.iter().find(|s| s.name == name)
}
