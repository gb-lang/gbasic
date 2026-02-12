# G-Basic

A compiled programming language designed for kids aged 7-12 to learn programming through game creation. G-Basic compiles to native binaries via LLVM, with built-in support for 2D graphics, sound, and input handling.

## Quick Start

### Prerequisites

- Rust (edition 2024)
- LLVM 18 (`brew install llvm@18` on macOS)
- Set `LLVM_SYS_180_PREFIX` to your LLVM 18 install path

### Build

```bash
cargo build -p gbasic --features llvm
cargo build -p gbasic-runtime-desktop
```

### Hello World

```gbasic
print("Hello, World!")
```

```bash
echo 'print("Hello, World!")' > hello.gb
./target/debug/gbasic hello.gb -o hello --run
```

## Language Features

### Variables

```gbasic
let x = 42
let name: String = "Alice"
let pi = 3.14
let active = true
```

### Functions

```gbasic
fun add(a: Int, b: Int) -> Int {
    return a + b
}

print(add(3, 4))  // 7
```

### Control Flow

```gbasic
if score > 100 {
    print("You win!")
} else {
    print("Keep trying!")
}

for i in 0..10 {
    print(i)
}

while running {
    System.FrameBegin()
    // game logic
    System.FrameEnd()
}

match direction {
    1 { print("North") }
    2 { print("South") }
    _ { print("Unknown") }
}
```

### String Interpolation

```gbasic
let name = "Alice"
print("Hello, {name}!")

let score = 42
print("Score: {score}")
```

### Arrays

```gbasic
let numbers = [1, 2, 3, 4, 5]
print(numbers[0])

for n in [10, 20, 30] {
    print(n)
}
```

### Logical Operators

```gbasic
if x > 0 and x < 100 {
    print("In range")
}

if not done or force_quit {
    break
}
```

## Namespaces

G-Basic provides built-in namespaces for common game operations:

| Namespace | Purpose |
|-----------|---------|
| **Screen** | Window, drawing, sprites |
| **Sound** | Sound effects and audio |
| **Input** | Keyboard and mouse |
| **Math** | Math functions |
| **System** | Timing, frame control |
| **Memory** | Key-value store |
| **IO** | File read/write |

### Screen

```gbasic
Screen.Init(800, 600)
Screen.Clear(0, 0, 0)
Screen.DrawRect(10, 20, 100, 50, 255, 0, 0)
Screen.DrawCircle(400, 300, 50, 0, 255, 0)
Screen.DrawLine(0, 0, 800, 600, 255, 255, 255)
Screen.Present()

let sprite = Screen.SpriteLoad("hero.bmp")
Screen.SpriteAt(sprite, 100.0, 200.0)
Screen.SpriteScale(sprite, 2.0)
Screen.SpriteDraw(sprite)
```

### Sound

```gbasic
Sound.EffectLoad("explosion.wav")
Sound.EffectPlay("explosion.wav")
Sound.EffectVolume("explosion.wav", 0.5)
```

### Input

```gbasic
Input.Poll()
if Input.KeyPressed("space") {
    // jump!
}
let mx = Input.MouseX()
let my = Input.MouseY()
```

### Math

```gbasic
let angle = Math.Pi() / 4.0
let x = Math.Cos(angle) * 100.0
let y = Math.Sin(angle) * 100.0
let r = Math.Random()  // 0.0 to 1.0
```

### System

```gbasic
// Game loop with 60 FPS targeting
while true {
    System.FrameBegin()
    let dt = System.FrameTime()
    // update and render...
    System.FrameEnd()
}
```

## Architecture

```
Source (.gb)
    |
    v
  Lexer  (logos) ──> Tokens
    |
    v
  Parser ──> AST
    |
    v
  Typechecker ──> Validated AST
    |
    v
  LLVM Codegen (inkwell) ──> .o file
    |
    v
  Linker (cc) + Runtime (.a) ──> Native Binary
```

## Project Structure

```
compiler/
  common/     # Shared types: AST, Span, Type, Error
  lexer/      # Logos-based tokenizer
  parser/     # Recursive descent parser
  typechecker/# Type checking pass
  irgen/      # LLVM IR generation (inkwell)
  cli/        # gbasic binary (clap)

runtime/
  desktop/    # SDL2 runtime (staticlib linked into binaries)
  web/        # WebAssembly runtime (planned)

examples/     # Example G-Basic programs
docs/         # Language grammar and documentation
tests/        # End-to-end integration tests
```

## Testing

```bash
# Run all unit tests
cargo test --workspace

# Run end-to-end tests (requires LLVM)
cargo test --test e2e

# Type-check only
./target/debug/gbasic program.gb --check

# Dump tokens/AST/IR for debugging
./target/debug/gbasic program.gb --dump-tokens
./target/debug/gbasic program.gb --dump-ast
./target/debug/gbasic program.gb --dump-ir
```

## Examples

| Example | Description |
|---------|-------------|
| `hello.gb` | Hello world |
| `basics.gb` | Variables and expressions |
| `arithmetic.gb` | Math operations |
| `control_flow.gb` | If/else, loops, match |
| `namespaces.gb` | Using built-in namespaces |
| `pong.gb` | Pong game |
| `particles.gb` | Particle effects |
| `math_viz.gb` | Animated sine wave |
| `sprite_demo.gb` | Sprite loading and movement |
| `sound_demo.gb` | Sound effects |

## License

MIT
