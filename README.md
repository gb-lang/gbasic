# G-Basic

A compiled programming language designed for kids aged 7-12 to learn programming through game creation. G-Basic compiles to native binaries via LLVM, with built-in support for 2D graphics, sound, and input handling.

> Browser playground: https://gb-lang.github.io/gbasic/

## Project Status

G-Basic is approximately 85% complete for the desktop target:

- Compiler: lexer, parser, typechecker, LLVM codegen all functional
- Runtime: SDL2-based desktop runtime with graphics, input, physics, and sound
- Object model: handle-based game objects (rect, circle) with properties and physics
- 95+ tests passing (unit, snapshot, e2e, error golden)
- 12 example programs including Pong and particle effects

## Quick Start

### Prerequisites

- Rust (edition 2024, stable toolchain)
- LLVM 18
- A C linker (`cc`)

**macOS:**
```bash
brew install llvm@18
export LLVM_SYS_180_PREFIX="$(brew --prefix llvm@18)"
```

**Linux:**
```bash
# Ubuntu/Debian
sudo apt install llvm-18-dev libpolly-18-dev
export LLVM_SYS_180_PREFIX=/usr/lib/llvm-18
```

**Windows:**

See [Building on Windows](#building-on-windows) below for full instructions.

### Build

```bash
# Build the compiler (requires LLVM)
cargo build -p gbasic --features llvm

# Build the desktop runtime (SDL2 builds from source via bundled feature)
cargo build -p gbasic-runtime-desktop

# Optional: enable SDL2_mixer for real audio (requires libsdl2-mixer-dev)
cargo build -p gbasic-runtime-desktop --features mixer
```

### Hello World

```gbasic
print("Hello, World!")
```

```bash
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
} else if score > 50 {
    print("Getting close!")
} else {
    print("Keep trying!")
}

for i in 0..10 {
    print(i)
}

// Inclusive range
for i in 1 to 5 {
    print(i)  // 1, 2, 3, 4, 5
}

while running {
    // game logic
}

match direction {
    1 -> { print("North") }
    2 -> { print("South") }
    _ -> { print("Unknown") }
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

// Dynamic arrays
let items = []
items.add(10)
items.add(20)
print(items.length)  // 2

for n in items {
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

### Object Model (Guardrails API)

```gbasic
// Create game objects
let ball = circle(10)
ball.position = Screen.center
ball.velocity = (3, -2)
ball.color = red
ball.bounces = true

let paddle = rect(100, 20)
paddle.position = Screen.bottom_center
paddle.color = white
paddle.solid = true

// Implicit game loop with physics
while true {
    if key("left") {
        paddle.move(-5, 0)
    }
    if key("right") {
        paddle.move(5, 0)
    }
    if ball.collides(paddle) {
        play("bounce")
    }
    print("Score: {score}").at(10, 10)
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
| **Asset** | Asset loading and caching |

### Layer 1 Shortcuts

Common operations have beginner-friendly shortcuts:

| Shortcut | Full Form |
|----------|-----------|
| `print(text)` | `IO.Print(text)` |
| `rect(w, h)` | Create rectangle object |
| `circle(r)` | Create circle object |
| `key(name)` | `Input.KeyPressed(name)` |
| `random(min, max)` | `Math.Random(min, max)` |
| `play(sound)` | `Sound.EffectPlay(sound)` |
| `clear(r, g, b)` | `Screen.Clear(r, g, b)` |

## CLI Usage

```bash
# Compile to binary
./target/debug/gbasic program.gb -o program

# Compile and run
./target/debug/gbasic program.gb -o program --run

# Type-check only
./target/debug/gbasic program.gb --check

# Debug output
./target/debug/gbasic program.gb --dump-tokens
./target/debug/gbasic program.gb --dump-ast
./target/debug/gbasic program.gb --dump-ir
```

## Building on Windows

### Prerequisites

1. **Visual Studio Build Tools** -- Install [Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) with the "Desktop development with C++" workload. This provides the MSVC compiler and linker.

2. **LLVM 18** -- Download the pre-built Windows installer from [releases.llvm.org](https://github.com/llvm/llvm-project/releases) or install via Chocolatey:
   ```powershell
   choco install llvm --version=18.1.8
   ```

3. **SDL2 development libraries** -- Download the SDL2 development libraries for Visual C++ from [libsdl.org](https://github.com/libsdl-org/SDL/releases), or install via vcpkg:
   ```powershell
   vcpkg install sdl2:x64-windows
   ```

4. **CMake** -- Required for building native dependencies. Install from [cmake.org](https://cmake.org/download/) or:
   ```powershell
   choco install cmake
   ```

### Environment Variables

Set these in your system environment or in the x64 Native Tools Command Prompt:

```powershell
set LLVM_SYS_180_PREFIX=C:\Program Files\LLVM
```

If you installed SDL2 manually (not via vcpkg), add the SDL2 `lib` directory to your `LIB` path:

```powershell
set LIB=%LIB%;C:\path\to\SDL2\lib\x64
```

### Build

Open the **x64 Native Tools Command Prompt for VS** (not a regular terminal) and run:

```powershell
cargo build --release
```

> **Note:** You must use the x64 Native Tools Command Prompt so that the MSVC linker and Windows SDK paths are available. Building from a regular PowerShell or CMD window will fail to find the C linker.

## Architecture

```
Source (.gb)
    |
    v
  Lexer  (logos) --> Tokens
    |
    v
  Parser --> AST
    |
    v
  Typechecker --> Validated AST
    |
    v
  LLVM Codegen (inkwell) --> .o file
    |
    v
  Linker (cc) + Runtime (.a) --> Native Binary
```

## Project Structure

```
compiler/
  common/     # Shared types: AST, Span, Type, Error, Shortcuts
  lexer/      # Logos-based tokenizer (case-insensitive)
  parser/     # Recursive descent parser
  typechecker/# Type checking pass
  irgen/      # LLVM IR generation (inkwell)
  cli/        # gbasic binary (clap) + e2e tests

runtime/
  desktop/    # SDL2 runtime (staticlib linked into binaries)
  web/        # WebAssembly runtime (planned)

examples/     # Example G-Basic programs
docs/         # Language grammar and documentation
```

## Testing

```bash
# Run all unit tests (lexer, parser, typechecker, codegen)
cargo test --workspace

# Run end-to-end tests (requires LLVM + runtime)
cargo test -p gbasic --test e2e

# Run error message golden tests
cargo test -p gbasic --test error_golden
```

## Examples

| Example | Description |
|---------|-------------|
| `hello.gb` | Hello world |
| `basics.gb` | Variables and expressions |
| `arithmetic.gb` | Math operations |
| `control_flow.gb` | If/else, loops, match |
| `namespaces.gb` | Using built-in namespaces |
| `pong.gb` | Pong game (keyboard + physics) |
| `particles.gb` | Particle effects |
| `math_viz.gb` | Animated sine wave |
| `bouncing_balls.gb` | Physics with bouncing objects |
| `color_mixer.gb` | Color manipulation |
| `sprite_demo.gb` | Sprite loading and movement |
| `sound_demo.gb` | Sound effects |

## Assets

The `assets/` directory contains minimal placeholder files that G-Basic example programs can reference out of the box:

| File | Description |
|------|-------------|
| `assets/beep.wav` | Minimal WAV file (short silence) -- default sound effect |
| `assets/default_sprite.bmp` | 8x8 white BMP -- default sprite placeholder |

Replace these with your own assets as needed. See `assets/README.md` for details.

## License

MIT
