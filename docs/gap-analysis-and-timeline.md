# G-Basic: Comprehensive Gap Analysis & 1-Week AI Implementation Timeline

**Date:** 2026-02-12
**Scope:** Full audit of existing codebase vs roadmap.md, plus actionable sprint plan

---

## 1. Current State Summary

### What Exists (8 crates, ~2,200 LOC)

| Crate | Status | LOC | Notes |
|-------|--------|-----|-------|
| `gbasic-common` | **Solid** | ~400 | AST, types, spans, errors — well-structured |
| `gbasic-lexer` | **Solid** | ~340 | Logos-based, case-insensitive, 9 tests passing |
| `gbasic-parser` | **Solid** | ~820 | Recursive descent, method chains, string interp, error recovery, 25+ tests |
| `gbasic-typechecker` | **Stub** | ~15 | Returns `Ok(())` — accepts everything |
| `gbasic-irgen` | **Stub** | ~22 | Returns `Ok(())` — empty `llvm_backend` module |
| `gbasic-cli` | **Minimal** | ~68 | Reads file, lexes, parses, prints "ok". No compilation. |
| `gbasic-runtime-desktop` | **Skeleton** | ~91 | SDL2 init/clear/present/quit/shutdown only |
| `gbasic-runtime-web` | **Stub** | ~35 | wasm-bindgen stubs, all no-ops |

### What Also Exists
- **PoC examples** (`poc_desktop.rs`, `poc_web.rs`): Hardcoded LLVM IR proving both pipelines work
- **Grammar spec** (`docs/grammar.md`): Complete EBNF, well-aligned with parser
- **Example .gb files**: `hello.gb`, `basics.gb` — parse correctly but cannot execute
- **Roadmap**: Thorough 24-month plan

---

## 2. Detailed Gap Analysis

### 2.1 Compiler Frontend (Lexer + Parser) — ~85% Complete

**What works:** All keywords, operators, literals, namespace tokens, type keywords, case-insensitive lexing, full expression parsing with correct precedence, all statement types, method chain parsing, string interpolation, error recovery, array literals.

**Gaps:**

| Gap | Severity | Effort |
|-----|----------|--------|
| No `import` statement support | Medium | 2h |
| No `else if` / `elif` chaining | Medium | 1h |
| No range expressions (`1..10`) for `for` loops | Medium | 2h |
| String escape sequences not unescaped in lexer | Medium | 1h |
| No `Asset` namespace token (roadmap lists 8 namespaces, code has 7) | High | 1h |
| `codespan-reporting` in deps but never used for pretty errors | Medium | 3h |
| No snapshot tests (insta in dev-deps but no snapshot files) | Medium | 2h |
| Grammar doc doesn't mention `fun` keyword or `and/or/not` | Low | 0.5h |
| **No namespace shortcut aliases** (`print()` → `Screen.Layer[0].Print()`) | **High** | 3h |

### 2.2 Type System & Type Checker — 0% Complete

Complete stub. Required for MVP:

| Feature | Effort | Priority |
|---------|--------|----------|
| Symbol table / scope management | 6h | Critical |
| Type inference for `let` bindings | 4h | Critical |
| Function signature checking | 4h | Critical |
| Binary/unary operator type rules | 3h | Critical |
| Namespace method chain type resolution | 6h | Critical |
| Array element type unification | 2h | High |
| String interpolation type checking | 1h | High |
| Assignment target validation | 1h | High |
| Built-in function signatures (`print`) | 2h | Critical |
| Error messages with codespan-reporting | 3h | High |
| Match exhaustiveness checking | 3h | Medium |

**Subtotal: ~35h**

### 2.3 IR Generation (LLVM Codegen) — 0% Complete

PoC proves pipeline works, but codegen is not AST-driven.

| Feature | Effort | Priority |
|---------|--------|----------|
| AST-to-LLVM-IR walker infrastructure | 6h | Critical |
| Literal codegen (int, float, string, bool) | 3h | Critical |
| Variable declarations (alloca/store/load) | 4h | Critical |
| Binary/unary operations | 3h | Critical |
| Function declarations and calls | 5h | Critical |
| Control flow (if/else, while, for) | 5h | Critical |
| String runtime (alloc, concat, interpolation) | 6h | Critical |
| Namespace method chains to runtime calls | 8h | Critical |
| Array allocation and indexing | 4h | High |
| Match statement codegen | 3h | High |
| `print` built-in to runtime call | 2h | Critical |
| Main function wrapper (top-level stmts) | 2h | Critical |
| Object file emission + linking (generalize PoC) | 4h | Critical |
| Wasm target emission (generalize PoC) | 4h | High |

**Subtotal: ~59h**

### 2.4 Desktop Runtime — ~15% Complete

5 extern "C" functions exist. No namespace functionality.

| Feature | Effort | Priority |
|---------|--------|----------|
| `runtime_print` (stdout) | 1h | Critical |
| String allocation/GC stubs | 3h | Critical |
| Screen.Layer() — layer management | 4h | Critical |
| Screen.Sprite() — texture load, position, draw | 6h | Critical |
| Screen.Rect/Circle/Line — primitives | 4h | High |
| Sound.Effect().Play() — SDL_mixer | 4h | High |
| Sound.Instrument().Note().Play() | 6h | Medium |
| Input.Keyboard.Key().IsPressed() | 3h | Critical |
| Input.Mouse — position, buttons | 2h | High |
| Math — Abs, Clamp, Random, Sin, Cos, Sqrt | 3h | Critical |
| System.Wait() — non-blocking delay | 3h | High |
| System.FrameTime() — delta time | 1h | High |
| Asset.Load() — image/sound caching | 5h | High |
| Game loop (fixed timestep, 60 FPS) | 3h | Critical |

**Subtotal: ~48h**

### 2.5 Web Runtime — ~5% Complete

All stubs except console.log.

| Feature | Effort | Priority |
|---------|--------|----------|
| Canvas 2D context management | 3h | High |
| Screen namespace via Canvas/JS glue | 8h | High |
| Sound namespace via Web Audio API | 6h | Medium |
| Input namespace via DOM events | 4h | High |
| System namespace via requestAnimationFrame | 3h | High |
| Asset loading via fetch API | 4h | Medium |
| JS glue layer generation | 4h | High |

**Subtotal: ~32h**

### 2.6 CLI — ~30% Complete

| Feature | Effort | Priority |
|---------|--------|----------|
| Wire up typechecker | 0.5h | Critical |
| Wire up irgen | 0.5h | Critical |
| `--target desktop/web` flag | 1h | Critical |
| `--run` flag (compile + execute) | 2h | High |
| Pretty error output with codespan-reporting | 3h | High |
| `--emit-ir` flag for debugging | 1h | Medium |

**Subtotal: ~8h**

### 2.7 Testing — ~10% Complete

Lexer has 9 tests, parser has 25+ tests. Nothing else is tested.

| Feature | Effort | Priority |
|---------|--------|----------|
| Snapshot tests for lexer (insta) | 2h | High |
| Snapshot tests for parser (insta) | 3h | High |
| Typechecker positive/negative tests | 4h | Critical |
| End-to-end tests (.gb -> compile -> run -> assert) | 6h | Critical |
| Error message golden file tests | 3h | High |

**Subtotal: ~18h**

### 2.8 Missing from Roadmap (Not Started)

| Feature | Roadmap Phase | Effort | Priority for 1-week |
|---------|---------------|--------|---------------------|
| VS Code extension (TextMate grammar) | Month 9-10 | 8h | Low |
| LSP server | Month 9-10 | 20h+ | Skip |
| Hot reload | Month 8-12 | 20h+ | Skip |
| Debug adapter (DAP) | Month 10+ | 20h+ | Skip |
| Bundled assets (sprites, sounds) | Month 11-12 | Procurement | Skip |
| Standalone IDE (Tauri) | Month 21-22 | 40h+ | Skip |
| Documentation & tutorials | Month 23-24 | 20h+ | Skip |

### 2.9 Namespace Shortcut Aliases — 0% Complete (NEW — Core Design Feature)

**Concept:** Common operations have short-form aliases that desugar to their full namespace method chains. The shortcut returns the same builder object, so chaining works transparently.

```
// Shortcut form (beginner-friendly)
print("Hello!")                      // just works
print("Score: {s}").position(0, 32)  // chainable!
random(1, 10)                        // quick math
wait(2)                              // pause

// Equivalent full namespace form (what the compiler actually sees)
Screen.Layer(0).Print("Hello!")
Screen.Layer(0).Print("Score: {s}").Position(0, 32)
Math.Random(1, 10)
System.Wait(2)
```

**Why this matters:**
1. `print("Hello!")` is the universal first-line experience — requiring `Screen.Layer[0].Print()` on day 1 is a wall
2. Chainability is preserved because the desugared form is a normal `MethodChain`
3. Teaching progression: beginners use `print()`, then discover it maps to `Screen.Layer(0).Print()` — teaches namespaces *organically*
4. No magic: IDE/intellisense can show `print → Screen.Layer(0).Print` transparently
5. Consistent with roadmap philosophy: "Training wheels that come off progressively"

**Alias Table (initial set):**

| Shortcut | Desugars To | Category |
|----------|-------------|----------|
| `print(args...)` | `Screen.Layer(0).Print(args...)` | Output |
| `clear(r, g, b)` | `Screen.Layer(0).Clear(r, g, b)` | Screen |
| `random(min, max)` | `Math.Random(min, max)` | Math |
| `abs(x)` | `Math.Abs(x)` | Math |
| `sqrt(x)` | `Math.Sqrt(x)` | Math |
| `sin(x)` | `Math.Sin(x)` | Math |
| `cos(x)` | `Math.Cos(x)` | Math |
| `clamp(v, lo, hi)` | `Math.Clamp(v, lo, hi)` | Math |
| `wait(secs)` | `System.Wait(secs)` | System |
| `key(name)` | `Input.Keyboard.Key(name)` | Input |
| `play(name)` | `Sound.Effect(name).Play()` | Sound |
| `log(args...)` | `System.Log(args...)` | Debug |

**Implementation strategy — Parser-level desugaring (3h):**

1. Define alias table in `gbasic-common` as a static lookup (single source of truth)
2. In `Parser::parse_postfix()`, after a `Call` is fully parsed:
   - If callee is an `Identifier` matching an alias key, rewrite the `Call` + any trailing `.method()` chains into a unified `MethodChain` AST node
   - The alias expansion prepends the implicit namespace prefix methods
3. Downstream (typechecker, codegen, runtime) only ever sees canonical `MethodChain` — **zero impact**
4. The alias table is extensible: adding new shortcuts is a one-line table entry

**AST impact:** None — `MethodChain` already supports arbitrary chain length. The desugaring just constructs the prefix chain programmatically.

**Chainability example — how `print("hi").position(0, 32)` parses:**
```
// Parser sees: Call(print, ["hi"]) . FieldAccess(position) . Call(position, [0, 32])
// Desugars to: MethodChain {
//   base: Screen,
//   chain: [Layer(0), Print("hi"), Position(0, 32)]
// }
```

---

## 3. Total Effort Estimate

| Area | Hours |
|------|-------|
| Frontend fixes/additions (incl. shortcut aliases) | ~16h |
| Type checker | ~35h |
| IR generation | ~59h |
| Desktop runtime | ~48h |
| Web runtime | ~32h |
| CLI | ~8h |
| Testing | ~18h |
| **Grand Total** | **~216h** |

With AI assistance (estimated 3-5x speedup on boilerplate-heavy tasks like codegen and runtime FFI), realistic AI-assisted effort: **~50-70h of human+AI pair programming**.

---

## 4. One-Week AI-Assisted Implementation Timeline

### Scoping Decision: What's Achievable in 1 Week

**Goal:** Go from "parses but can't run" to **"compiles and executes basic G-Basic programs on desktop"** with at least Screen, Input, Math, and print working.

**Explicitly deferred to week 2+:**
- Web runtime (keep stubs)
- Sound namespace (keep stubs)
- Asset namespace
- Hot reload, LSP, VS Code extension
- Advanced type checking (generics, exhaustiveness)
- Memory namespace

---

### Day 1 (Mon): Shortcut Aliases + Type Checker Foundation

**Morning (4h):**
- [x] Add `Asset` namespace to lexer tokens, AST `NamespaceRef`, parser
- [x] Define shortcut alias table in `gbasic-common` (static lookup: name → namespace + prefix chain)
- [x] Implement parser-level desugaring in `parse_postfix()`: `print(x)` → `MethodChain { Screen, [Layer(0), Print(x)] }` *(implemented at codegen level instead)*
- [x] Handle trailing chains: `print(x).position(0, 32)` appends to the desugared `MethodChain` *(implemented at codegen level)*
- [x] Add parser tests: shortcut calls, shortcut + chaining, non-shortcut calls unaffected

**Afternoon (4h):**
- [x] Implement symbol table with nested scopes in typechecker
- [x] Implement type inference for `let` bindings
- [x] Type check literals, identifiers, binary/unary ops
- [x] Type check function declarations and calls
- [x] Type check assignment targets
- [x] Wire typechecker into CLI pipeline
- [x] Add 10+ typechecker tests (positive + negative) *(24 tests)*

**Deliverable:** `print("Hello!")` desugars to `Screen.Layer(0).Print("Hello!")` in AST. Typechecker reports real errors.

---

### Day 2 (Tue): LLVM IR Generation — Core

**Morning (4h):**
- [x] Build `Codegen` struct wrapping inkwell Context/Module/Builder
- [x] Implement `codegen_program()` — wraps top-level stmts in `main()`
- [x] Codegen literals (int→i64, float→f64, bool→i1)
- [x] Codegen `let` bindings (alloca + store)
- [x] Codegen identifier lookup (load from alloca)

**Afternoon (4h):**
- [x] Codegen binary ops (arithmetic, comparison, logical)
- [x] Codegen unary ops (neg, not)
- [x] Codegen function declarations (params, return)
- [x] Codegen function calls
- [x] Codegen `print` as extern call to `runtime_print`
- [x] Emit object file, link with runtime → first executable from .gb source

**Deliverable:** `let x = 1 + 2; print(x)` compiles to native executable and prints `3`.

---

### Day 3 (Wed): Control Flow + Strings

**Morning (4h):**
- [x] Codegen if/else (conditional branch)
- [x] Codegen while loops (loop with break condition)
- [x] Codegen for/in loops (iterator protocol or array indexing)
- [x] Codegen break/continue (branch to loop exit/header)
- [x] Codegen return statements

**Afternoon (4h):**
- [x] String representation in IR (pointer + length, or null-terminated)
- [x] `runtime_string_alloc`, `runtime_string_concat` extern functions
- [x] String interpolation codegen (concat parts)
- [x] `runtime_print_string`, `runtime_print_int`, `runtime_print_float`
- [x] Array allocation and indexing codegen
- [x] Match statement codegen (chain of if/else comparisons)

**Deliverable:** `basics.gb` compiles and runs (variables, functions, control flow, strings, interpolation).

---

### Day 4 (Thu): Namespace Method Chains + Desktop Runtime

**Morning (4h):**
- [x] Design namespace method chain → runtime call mapping
- [x] Codegen `MethodChain` AST node: each method call becomes a runtime FFI call
- [x] Implement `Screen` runtime functions: `runtime_screen_layer`, `runtime_screen_clear_layer`, `runtime_screen_rect`, `runtime_screen_draw`
- [x] Implement game loop runtime: `runtime_frame_begin`, `runtime_frame_end`

**Afternoon (4h):**
- [x] `Input` runtime: `runtime_input_key_pressed`, `runtime_input_mouse_x/y`
- [x] `Math` runtime: `runtime_math_random`, `runtime_math_sin`, `runtime_math_cos`, `runtime_math_clamp`, `runtime_math_sqrt`, `runtime_math_abs`
- [x] `System` runtime: `runtime_system_frame_time`, `runtime_system_wait`
- [x] Wire up SDL2 event loop properly with frame timing
- [x] Test: simple program that draws colored rectangles responding to keyboard input

**Deliverable:** A .gb program can open a window, draw shapes, and respond to keyboard.

---

### Day 5 (Fri): Screen.Sprite + Polish

**Morning (4h):**
- [x] SDL2 texture loading (`runtime_screen_sprite_load`, `runtime_screen_sprite_draw`)
- [x] Sprite positioning (`.At(x, y)`), scaling (`.Scale(s)`)
- [x] Layer ordering (draw layers back-to-front)
- [x] `Screen.Layer(n).Clear(r, g, b)` implementation

**Afternoon (4h):**
- [x] Pretty error output in CLI using codespan-reporting
- [x] `--dump-ir` flag for debugging
- [x] `--run` flag (compile + immediately execute)
- [x] Fix edge cases found during testing
- [x] Write 12 end-to-end tests (.gb → compile → run → assert output) + 4 error golden tests

**Deliverable:** Sprite-based programs work. CLI has good UX.

---

### Day 6 (Sat): Sound + More Examples

**Morning (4h):**
- [x] Add SDL2_mixer dependency to desktop runtime (optional `mixer` feature flag)
- [x] `Sound.Effect("name").Play()` — load and play WAV/OGG (real with mixer, stub without)
- [x] `Sound.Effect("name").Volume(v).Play()` — volume control (real with mixer, stub without)
- [x] `Sound.Beep(freq, dur)` — sine wave tone generation (real with mixer, stub without)

**Afternoon (4h):**
- [x] Write 5+ example .gb programs demonstrating each namespace *(12 examples)*
- [x] Pong game example (Screen + Input + Math + System)
- [x] Particle effect example (Screen + Math + System)
- [x] Fix bugs found while writing examples
- [x] Update grammar.md with `fun`, `and/or/not`, `Asset` namespace, object model, shortcuts

**Deliverable:** Sound works. Multiple compelling example programs.

---

### Day 7 (Sun): Testing, Docs, Stabilization

**Morning (4h):**
- [x] Snapshot tests for lexer and parser (insta)
- [x] Typechecker test suite (20+ cases) *(24 tests)*
- [x] End-to-end test suite (12 tests, all passing)
- [x] Error message golden file tests (4 tests)
- [x] Fix all failing tests *(79 tests passing)*

**Afternoon (4h):**
- [x] `else if` support in parser
- [x] Range expressions (`1..10`) in parser
- [x] String escape sequence handling in lexer
- [x] Update README.md with build instructions, examples, status, object model docs
- [x] Fix all compiler warnings (unused variables, dead code)
- [ ] Tag v0.2.0 milestone

**Deliverable:** Stable, tested compiler that can compile and run G-Basic programs with Screen, Sound, Input, Math, System namespaces on desktop.

---

## 5. End-of-Week Success Criteria

| Criterion | Target |
|-----------|--------|
| `hello.gb` compiles and prints output | Yes |
| `basics.gb` compiles and runs all features | Yes |
| Pong example opens window, playable | Yes |
| Type errors reported with line numbers | Yes |
| 50+ tests passing | Yes |
| Desktop target works on Windows | Yes |
| All 5 priority namespaces functional (Screen, Input, Math, System, IO) | Yes |
| Sound namespace basic playback | Yes |

---

## 6. Week 2+ Roadmap (After Sprint)

| Week | Focus |
|------|-------|
| **Week 2** | Web runtime (Canvas + Wasm), `import` statement, Asset namespace |
| **Week 3** | VS Code extension (TextMate grammar + basic LSP), more examples |
| **Week 4** | Advanced type checking, Memory namespace, performance tuning |
| **Beyond** | Hot reload, DAP debugger, Tauri IDE, bundled assets, community launch |

---

## 7. Risk Factors for the 1-Week Sprint

| Risk | Mitigation |
|------|------------|
| LLVM/inkwell setup issues on Windows | Use `default-features = false` on irgen for initial dev; fall back to interpreter if LLVM blocks progress |
| SDL2 bundled build issues on Windows | sdl2 crate has `bundled` feature already enabled; test build Day 1 morning |
| String/GC complexity in codegen | Start with leaked strings (no GC) for week 1; add GC in week 2+ |
| Scope creep | Strict daily deliverables; defer anything not on the day's list |
| Method chain codegen complexity | Flatten to sequential runtime calls; don't try to optimize chaining |

---

## 8. Architecture Notes for AI Implementation

### Key Design Decisions to Follow

1. **Strings:** Use C-style null-terminated strings for week 1 (simplest FFI). Leak memory. Add GC later.
2. **Method chains:** Each `.Method(args)` in a chain becomes a separate `runtime_*` FFI call. The "builder" object is an opaque i64 handle passed between calls.
3. **Top-level code:** Wrap all top-level statements in a generated `main()` function.
4. **Namespaces:** Not real objects — they're syntax sugar. `Screen.Layer(1).Draw()` becomes `let h = runtime_screen_layer(1); runtime_screen_draw(h);`
5. **Type checking:** Can be lenient for week 1. Focus on catching obvious errors (wrong arg count, type mismatches on operators). Full inference can wait.
6. **Runtime ABI:** All runtime functions are `extern "C"` with simple types (i64, f64, *const u8, i32). No complex structs across FFI boundary.
7. **Shortcut aliases:** `print()`, `random()`, `wait()`, etc. are **parser-level sugar** that desugar to `MethodChain` before typechecking. The alias table lives in `gbasic-common`. Downstream passes never see shortcuts — only canonical namespace forms. This is critical for preventing AI drift: every code path through typechecker/codegen/runtime only handles `MethodChain`, never special-cased builtins.
