# G-Basic Roadmap

## Executive Summary

**Mission:** Create a fundamental programming language that teaches efficient architecture and machine reality through immediate, visual feedback - where a beginner writes real code that maps to actual hardware behavior within their first session.

**Non-negotiable constraint:** The language must teach genuine computer science concepts (not abstract them away) while being immediately productive.

**Strategic approach:** Reuse proven foundations (LLVM, existing parsers, battle-tested GC) and focus innovation only on what matters: the namespace design, teaching model, and integration layer.

---

## Part 1: What We're Actually Building

### The Core Product

A **teaching-oriented systems language** with:
1. Hardware-reflective namespace design (Screen, Sound, Input, Math, System, Memory, IO)
2. Visual/audio-first standard library (bundled with quality assets)
3. Method chaining as primary API pattern for discoverability
4. Performance transparency (what you write maps predictably to machine operations)
5. Single-file execution model (zero ceremony)
6. **Multi-target output:** same source compiles to desktop (native executable) or web (HTML + JS + Wasm)
7. **AI-friendly by design:** The architecture (small namespace set, consistent chaining API, predictable semantics, explicit hardware mapping) inherently makes the language easier for AI to parse, reason about, and predict than general-purpose languages that tometimes have large, inconsistent APIs. Also part of the goal is to use AI in generating most of the language.

**NOT building:**
- A new runtime from scratch (reuse V8/LLVM/similar)
- A new GC algorithm (reuse proven generational GC)
- A new UI framework (reuse ImGui or web technologies)
- A teaching platform separate from the language (the language IS the teaching)

### What "Fundamental" Actually Means

The language exposes and teaches:
- **Memory:** Stack vs heap, ownership, lifetime, allocation cost
- **CPU:** Instructions map to operations, performance characteristics visible
- **Display pipeline:** Framebuffer → layers → blit → vsync
- **I/O boundaries:** File system, peripherals, network as explicit foreign operations
- **Concurrency model:** Explicit async/await or structured concurrency (no hidden threads)

This is NOT Scratch (hides everything) and NOT C (shows too much too early).
It's a **guided path through the machine** with training wheels that come off progressively.

---

## Part 2: Strategic Architecture Decisions

### Decision 0: Compiler Implementation Language

**The compiler itself must be written in something.** This decision affects hiring, iteration speed, LLVM integration ergonomics, and long-term maintainability.

**Options:**

| Language | LLVM Integration | Pros | Cons |
|----------|-----------------|------|------|
| **C++** | Native (LLVM is C++) | Zero-friction LLVM API access, direct use of libclang/TableGen, largest pool of compiler engineers | Slow iteration, memory bugs, complex build systems |
| **Rust** | Via `inkwell` or `llvm-sys` crate | Memory safety, strong type system, good Wasm tooling (Cranelift fallback is native Rust), growing compiler community | LLVM bindings lag behind upstream, steeper learning curve for contributors |
| **Zig** | Via C interop (LLVM is bundled in Zig's toolchain) | Simple language, fast compilation, C interop is trivial, small binary output | Smaller ecosystem, fewer compiler engineers available, language still maturing |
| **Go** | Via `tinygo/llvm` or CGo bindings | Fast iteration, simple concurrency for LSP server, easy onboarding | CGo overhead, poor LLVM ergonomics, GC adds latency to compiler itself |

**Recommended: Rust**
- `inkwell` provides a safe, idiomatic wrapper over the LLVM C API
- The LSP server, CLI tooling, and compiler can share a single codebase
- Cranelift (Rust-native) serves as a realistic fallback backend
- Cargo ecosystem provides testing, benchmarking, and fuzzing tooling out of the box
- Tauri (Phase 2 IDE) is also Rust, enabling code sharing between compiler and IDE shell

**If Rust proves too slow to iterate on:** Fall back to C++ with sanitizers enabled, using LLVM's native APIs directly.

### Decision 1: Language Core (Reuse Foundation)

**Don't reinvent:** Parser generators, type systems, intermediate representations

**Reuse options:**
- **LLVM** - Industry standard, proven optimization, multiple backends
- **Cranelift** - Faster compilation, Rust ecosystem, wasm-first
- **JVM** - Mature, excellent GC, huge ecosystem (but heavy)

**Recommended: Hybrid strategy — LLVM + Multi-Target**
- Single frontend → LLVM IR (shared); backends emit desktop or web output
- **Desktop:** LLVM → x86-64/ARM native code, link with SDL2 runtime → native executable
- **Web:** LLVM → WebAssembly, JavaScript glue, PixiJS/Web Audio API → HTML + JS + Wasm bundle
- Custom parser/frontend for G-Basic syntax; namespace API maps to IR, backends implement platform runtimes

**Parser strategy: Hand-written recursive descent (not ANTLR)**
- A teaching language lives or dies by its error messages. Hand-written parsers allow full control over error recovery, context-aware suggestions ("Did you forget `.Draw()` at the end of your chain?"), and span tracking for precise diagnostics.
- Languages that prioritize developer experience (Go, Rust, Swift, V) all use hand-written parsers for this reason.
- Trade-off: More upfront work (~2-4 weeks vs ~1 week for ANTLR), but pays for itself immediately in error quality.

**Timeline impact:** 6-9 months saved vs building IR + optimizer from scratch

### Multi-Backend Architecture (Desktop + Web)

**Hybrid strategy: LLVM + Multi-Target**

```
┌─────────────────────────────────────────────────────────┐
│              G-Basic Frontend                            │
│  (Parser, Type Checker, Semantic Analysis)               │
└─────────────────┬───────────────────────────────────────┘
                  │
                  ↓
┌─────────────────────────────────────────────────────────┐
│              LLVM IR Generation                         │
│  (Shared code — namespaces map to IR)                   │
└────┬────────────────────────────────────────────────┬───┘
     │                                                 │
     ↓                                                 ↓
┌──────────────────────┐                  ┌──────────────────────┐
│  Desktop Backend     │                  │   Web Backend        │
│                      │                  │                      │
│  LLVM → x86-64/ARM   │                  │  LLVM → WebAssembly  │
│  Link with SDL2      │                  │  + JavaScript glue   │
│  Native executable   │                  │  + PixiJS for Screen │
│                      │                  │  + Web Audio API     │
└──────────────────────┘                  └──────────────────────┘
```

**Compiler pipeline**

```
┌─────────────────────────────────────────────┐
│ Source Code (.gb file)                      │
└─────────────┬───────────────────────────────┘
              │
              ↓
┌─────────────────────────────────────────────┐
│ Frontend (Shared)                            │
│  — Lexer/Parser (ANTLR or hand-written)      │
│  — AST construction                          │
│  — Type checking                             │
│  — Semantic analysis                         │
│  — Namespace resolution                      │
└─────────────┬───────────────────────────────┘
              │
              ↓
┌─────────────────────────────────────────────┐
│ Middle-End (Shared)                          │
│  — LLVM IR generation                        │
│  — Namespace → IR mapping                    │
│  — Optimization passes (optional)            │
└─────────────┬───────────────────────────────┘
              │
        ┌─────┴─────┐
        ↓           ↓
┌──────────────┐  ┌──────────────────────────┐
│ Desktop      │  │ Web                      │
│ Backend      │  │ Backend                  │
│              │  │                          │
│ LLVM → x64   │  │ LLVM → Wasm              │
│ Link SDL2    │  │ Generate JS glue         │
│ Executable   │  │ Bundle assets            │
│              │  │ HTML + JS + Wasm         │
└──────────────┘  └──────────────────────────┘
```

**Runtime libraries**

| Layer | Desktop | Web |
|-------|---------|-----|
| **Graphics** | SDL2 (window, render, input) | PixiJS (canvas/WebGL) |
| **Audio** | SDL2_mixer | Web Audio API |
| **System / async** | Minimal C/C++ runtime, GC (bdwgc or similar) | JavaScript async/await (e.g. System.Wait), JS GC |
| **Shared (compiled)** | Math, asset caching, namespace method implementations | Same logic in Wasm + JS glue |

**Target outputs:** Native executable (desktop); HTML + JS + Wasm bundle (web). Same G-Basic source, one build flag or target option.

### Decision 2: Runtime & Memory Management

**Don't reinvent:** Garbage collection algorithms

**Reuse options:**
- **bdwgc (Boehm)** - Conservative GC, C/C++ compatible, proven
- **mimalloc + reference counting** - Deterministic, predictable, teachable
- **Rust's ownership model** - Compile-time, but complex for beginners
- **Generational GC (reuse existing)** - Industry standard, well understood

**Recommended: Two-tier approach**
1. **Default: Generational GC** (reuse proven implementation like bdwgc or similar)
   - Automatic, beginner-friendly
   - Predictable pause times with tuning
   - Clear teaching moments: "This is when cleanup happens"

2. **Advanced: Manual memory hints** (opt-in via Memory namespace)
   - `Memory.Pool()` for hot paths
   - `Memory.Ref()` for explicit sharing
   - `Memory.Stats()` for inspection

**Teaching progression:**
- Weeks 1-4: Just use GC, don't think about it
- Weeks 5-8: Learn what GC does, when it runs
- Weeks 9+: Optimize hot paths with Memory namespace

**Timeline impact:** 12-18 months saved vs building custom GC

### Decision 3: Graphics Backend (Desktop + Web)

**Don't reinvent:** Rendering pipelines, windowing, input handling

**Desktop backend:** SDL2 + custom layer abstraction
- SDL2 for windowing, input, basic drawing
- Screen namespace maps to SDL texture/surface; hardware acceleration via SDL_Renderer
- Battle-tested, cross-platform, minimal dependencies

**Web backend:** PixiJS
- Canvas/WebGL, same Screen namespace API surface via JS glue
- Layer/sprite/drawing primitives map to PixiJS display objects
- Single codebase: namespace → IR; desktop backend uses SDL2, web backend uses PixiJS at runtime

**Timeline impact:** 6-12 months saved vs building window/input/render from scratch

### Decision 4: Audio Backend (Desktop + Web)

**Don't reinvent:** Audio synthesis, mixing, streaming

**Desktop backend:** SDL2_mixer + custom Sound namespace
- SDL_mixer for playback, mixing; Sound namespace for note/instrument abstraction

**Web backend:** Web Audio API
- Same Sound namespace API via JavaScript glue; instruments/effects map to Web Audio nodes

**Bundled assets strategy (shared):**
- 5-10 instruments (multi-sampled), 20-30 sound effects; CC0 or similar; total <50MB

**Timeline impact:** 3-6 months saved vs building audio engine

### Decision 5: Development Environment

**Don't reinvent:** Code editors, debuggers, UI frameworks

**Reuse options:**
- **Web-based IDE** (Monaco editor + Node.js backend)
- **Desktop IDE** (Electron or Tauri)
- **VS Code extension** (leverage existing infrastructure)
- **Custom native** (Qt or ImGui)

**Recommended: Phase approach**

**Phase 1: VS Code extension**
- Leverage existing editor
- Syntax highlighting via TextMate grammar
- LSP for intellisense/errors
- Custom panel for live preview
- Fastest path to usable IDE

**Phase 2: Standalone desktop (Tauri)**
- Package VS Code extension features
- Add custom preview/asset panels
- Distribute as single executable
- Keep web tech for UI (faster iteration)


---

## Part 3: Namespace Architecture (The Innovation Layer)

This is where we DON'T reuse - this is the core teaching innovation.

### Core Design Principle: Hardware Reflection

Every namespace maps to actual hardware or system boundary:

```
Screen    → Framebuffer, GPU pipeline, display refresh
Sound     → Audio buffer, DAC, speakers
Input     → Device drivers, interrupt handlers, event queue
Math      → ALU, FPU, SIMD units
System    → CPU scheduler, timers, program lifecycle
Memory    → RAM, heap allocator, garbage collector
IO        → File system, network stack, peripheral buses
Asset     → File I/O + decoding + caching
```

### Namespace API Design (Consistency Contract)

**Every namespace follows the same pattern:**

1. **Entry point** - Top-level namespace (capitalized)
2. **Target selection** - What you're operating on
3. **Action chain** - What you're doing (chainable)
4. **Terminal action** - Final execution (not chainable)

**Examples across namespaces:**

```
# Screen namespace
Screen.Layer(1).Sprite("hero").At(100, 200).Scale(2).Draw()
#      ^^^^^^^^ ^^^^^^^^^^^^^^ ^^^^^^^^^^^^^ ^^^^^^^^ ^^^^^^
#      Target   What           Position      Size      Terminal

# Sound namespace
Sound.Instrument("piano").Note(60).Duration(0.5).Play()
#     ^^^^^^^^^^^^^^^^^^^^^ ^^^^^^^^ ^^^^^^^^^^^^^ ^^^^^^
#     Target                What      How long      Terminal

# Input namespace
Input.Keyboard.Key("Space").IsPressed()
#     ^^^^^^^^ ^^^^^^^^^^^^ ^^^^^^^^^^^
#     Target   What         Query (terminal)

# System namespace
System.Timer(5.0).OnComplete(callback).Start()
#      ^^^^^^^^^^ ^^^^^^^^^^^^^^^^^^^^ ^^^^^^^
#      Target     What happens         Terminal
```

### Namespace Shortcut Aliases (Beginner On-Ramp)

**Core principle:** Common operations have short-form aliases that are **parser-level sugar** desugaring to their full namespace method chains. The shortcut returns the same builder object, so chaining works transparently.

**This is NOT a separate feature — it IS the namespace system**, just with a beginner-friendly entry point.

```
# Shortcut form (what beginners write)
print("Hello!")                         # just works on day 1
print("Score: {s}").Position(0, 32)     # chainable — same builder object
random(1, 10)                           # quick math
wait(2)                                 # pause without blocking

# What the compiler actually sees (desugared in parser)
Screen.Layer(0).Print("Hello!")
Screen.Layer(0).Print("Score: {s}").Position(0, 32)
Math.Random(1, 10)
System.Wait(2)
```

**Alias table (initial set):**

| Shortcut | Desugars To | Category |
|----------|-------------|----------|
| `print(args...)` | `Screen.Layer(0).Print(args...)` | Output |
| `clear(r, g, b)` | `Screen.Layer(0).Clear(r, g, b)` | Screen |
| `random(min, max)` | `Math.Random(min, max)` | Math |
| `abs(x)` | `Math.Abs(x)` | Math |
| `sqrt(x)` | `Math.Sqrt(x)` | Math |
| `sin(x)` / `cos(x)` | `Math.Sin(x)` / `Math.Cos(x)` | Math |
| `clamp(v, lo, hi)` | `Math.Clamp(v, lo, hi)` | Math |
| `wait(secs)` | `System.Wait(secs)` | System |
| `key(name)` | `Input.Keyboard.Key(name)` | Input |
| `play(name)` | `Sound.Effect(name).Play()` | Sound |
| `log(args...)` | `System.Log(args...)` | Debug |

**Implementation rule:** Desugaring happens **once, in the parser**. The typechecker, codegen, and runtime never see shortcut names — only canonical `MethodChain` nodes. This prevents architectural drift: there is exactly one code path for namespace operations, whether the user wrote `print("hi")` or `Screen.Layer(0).Print("hi")`.

**Teaching progression:**
- Week 1: Use `print()`, `random()`, `wait()` — it just works
- Week 2: Discover `Screen.Layer(0).Print()` — "oh, print goes to a screen layer!"
- Week 3: Use `Screen.Layer(1).Print()` — "I can print to different layers!"
- Week 4+: Build custom abstractions on top of the full namespace API

### Critical Teaching Moments Built Into Design

**1. Memory visibility**
```
# Default: GC handles it
sprite = Screen.Sprite("hero")  # Allocated, GC will clean up

# Advanced: Manual control
pool = Memory.Pool(1024)  # Explicit allocation
sprite = pool.Allocate(Sprite, "hero")  # From pool
pool.Clear()  # Explicit cleanup
```
**Teaching:** "You can let the machine manage memory, or control it yourself"

**2. Performance transparency**
```
# This is expensive (creates new texture each frame)
function update():
    Screen.Layer(1).Image("background.png").Draw()  # Bad!

# This is fast (reuse texture)
bg = Asset.Load("background", "background.png")  # Once, upfront
function update():
    Screen.Layer(1).Texture(bg).Draw()  # Just blit
```
**Teaching:** "Loading is expensive, drawing is cheap - measure the difference"

**3. Concurrency model**
```
# This blocks (wrong)
function enemy_patrol():
    move_left()
    Sleep(2)  # Everything freezes!
    move_right()

# This doesn't block (right)
function enemy_patrol():
    move_left()
    System.Wait(2)  # Just this function pauses
    move_right()
```
**Teaching:** "Your code can pause without stopping the whole program"

---

## Part 4: Asset Integration Strategy


### Why assets matter

1. **Immediate visual feedback** - Code changes → visible results instantly
2. **Quality baseline** - Beginners judge themselves by output quality
3. **Reduced cognitive load** - Don't learn art tools AND programming simultaneously
4. **Professional examples** - Bundled demos show "good" looks like

### Bundled Asset Library (Non-Negotiable)

**Sprites (30-50 items):**
- Characters: 8-10 variations (different colors/styles of person, robot, animal)
- Objects: 10-15 items (platforms, coins, hearts, stars, boxes)
- UI elements: 5-10 (buttons, panels, icons)
- Tiles: 8-12 (grass, water, stone, wall variations)
- Effects: 5-8 (explosion, sparkle, smoke)

**Quality bar:**
- 32x32 or 64x64 base size
- Consistent art style (pixel art or vector)
- Multiple animation frames where relevant
- Transparent backgrounds (PNG)
- Professional quality (hire artist or curate from open assets)

**Sounds (20-30 items):**
- Instruments: Piano (multi-sampled), drums (kick/snare/hat), organ, synth
- Effects: Jump, coin, hit, explosion, powerup, death, win, lose
- Music: 2-3 short loops (upbeat, calm, tense)

**Quality bar:**
- 44.1kHz or 48kHz, 16-bit
- Professionally mixed/mastered
- Loopable where appropriate
- <10MB total for all sounds

**Budget:** $2,000-5,000 for professional asset creation/curation
**Timeline:** 1-2 months (can be parallel with language development)

### Asset Discovery & Teaching Integration

**Asset panel design:**
```
┌─────────────────────────────────────┐
│ Assets                   [+ Add]    │
├─────────────────────────────────────┤
│ Sprites                             │
│  📦 hero_blue     [Preview] [Code]  │
│  📦 hero_red      [Preview] [Code]  │
│  📦 coin          [Preview] [Code]  │
│  📦 platform      [Preview] [Code]  │
│                                     │
│ Sounds                              │
│  🔊 jump          [▶️ Play] [Code]  │
│  🔊 coin          [▶️ Play] [Code]  │
│  🎵 piano_c4      [▶️ Play] [Code]  │
└─────────────────────────────────────┘
```

**[Code] button shows example:**
```
# Click [Code] on "hero_blue" sprite
Screen.Layer(1).Sprite("hero_blue").At(100, 200).Draw()

# Click [Code] on "jump" sound
Sound.Effect("jump").Play()
```

**Teaching moment:** "Assets are just names - the code references them"

---

## Part 5: Engineering Roadmap

### Foundation Phase (Months 1-4): Language Core

**Month 1: Specification & Architecture**
- Complete language specification (syntax, semantics, type system)
- Namespace API contract documentation
- Memory model specification
- Multi-backend architecture: shared frontend/IR, desktop + web target outputs
- Tech stack validation (LLVM, SDL2 desktop; Wasm + PixiJS/Web Audio for web)
- Asset requirements and sourcing plan

**Month 2: Parser & Frontend**
- Lexer/parser (use ANTLR or similar)
- AST representation
- Type checker
- Basic semantic analysis
- Error message framework

**Month 3-4: Code Generation & Runtime (Multi-Backend)**
- Shared LLVM IR generation; namespace → IR mapping
- Desktop backend: LLVM → x86-64/ARM, link SDL2, GC (bdwgc or similar), native executable
- Web backend: LLVM → WebAssembly, JS glue, asset bundling → HTML + JS + Wasm
- Minimal runtime libs per target; basic standard library scaffolding

**Deliverable:** Compiler that emits desktop executable and/or web bundle from same G-Basic source
**Success:** "Hello World" compiles and runs on desktop and in browser

### Implementation Phase 1 (Months 5-8): Core Namespaces

**Month 5-6: Screen + Math (Both Backends)**
- Desktop: SDL2 integration; Screen namespace → SDL layer/sprite/drawing
- Web: PixiJS integration via JS glue; same Screen API → PixiJS display objects
- Math namespace (vectors, basic ops, random) — shared IR
- Window/canvas and event loop per target

**Month 7: Sound + Input (Both Backends)**
- Desktop: SDL_mixer; Sound namespace (instruments, effects)
- Web: Web Audio API glue; same Sound API
- Input namespace (keyboard, mouse); event handling and dispatch per target

**Month 8: System + Asset (Both Backends)**
- System namespace (Wait, FrameTime, timers); async/await or coroutines (desktop: runtime; web: JS)
- Asset namespace (load, cache, reference); image/sound loading per target

**Deliverable:** All core namespaces functional on desktop and web
**Success:** Example program with sprite, sound, input, animation runs at 60 FPS on desktop and in browser

### Implementation Phase 2 (Months 9-12): Tooling

**Month 9-10: VS Code Extension**
- TextMate grammar (syntax highlighting)
- LSP server (intellisense, errors)
- Build task integration (target: desktop and/or web)
- Preview panel (embedded SDL window for desktop, web view for web target)

**Month 11-12: Asset Integration & Examples**
- Asset library curation/creation
- Asset panel UI in extension
- 15-20 example programs
- Tutorial content (in-editor)

**Deliverable:** Full IDE experience in VS Code
**Success:** Beginner completes first program in 15-20 minutes

### Polish Phase (Months 19-24): Performance & Distribution

**Month 19-20: Optimization**
- Performance profiling
- Hot path optimization
- Memory tuning
- 60 FPS validation across examples

**Month 21-22: Standalone IDE (Optional)**
- Tauri-based desktop app
- Bundle compiler + runtime
- Asset panel and preview
- One-click install

**Month 23-24: Documentation & Launch**
- Complete API documentation
- Video tutorials
- Teacher resources
- Public beta release

**Deliverable:** Production-ready v1.0
**Success:** 100+ beta users, positive feedback, stable performance

---

## Part 6: Success Metrics (Measurable)

### Language Quality Metrics

| Metric | Target | Measurement |
|--------|--------|-------------|
| **Compile time** | <1s for <1000 LOC | Timer |
| **Runtime performance** | 60 FPS, 50 sprites, 10 sounds | Profiler |
| **Memory overhead** | <50MB for typical program | Process monitor |
| **Startup time** | <500ms to first frame | Timer |
| **Error quality** | 90% actionable without docs | User testing |

### Teaching Effectiveness Metrics

| Metric | Target | Measurement |
|--------|--------|-------------|
| **Time to first success** | 15 min (moving sprite + sound) | Observation |
| **Concept retention** | Explain 4/7 namespaces after session | Interview |
| **Code quality** | Writes efficient code naturally | Code review |
| **Debugging ability** | Fixes common errors independently | Task completion |
| **Skill transfer** | Can teach peer a concept | Peer session |

### Adoption Metrics

| Metric | 6 months | 12 months | 24 months |
|--------|----------|-----------|-----------|
| **Active users** | 50 | 500 | 2000 |
| **Community programs** | 20 | 100 | 500 |
| **Tutorial completions** | 30 | 300 | 1500 |
| **Retention (return users)** | 40% | 50% | 60% |

---

## Part 7: What Makes This Different

### Compared to Scratch
- **Scratch:** Hides all machine reality, visual programming only
- **G-Basic:** Exposes machine reality through namespaces, text-based, efficient

### Compared to LÖVE (Lua)
- **LÖVE:** Game framework, Lua's dynamic typing, callback-heavy
- **G-Basic:** Teaching language, static types optional, namespace-driven

### Unique Value Proposition

**G-Basic is the only language that:**
1. Maps syntax directly to hardware concepts (not abstractions)
2. Ships with production-quality assets (not placeholders)
3. Makes efficient code the natural path (not the expert path)
4. Teaches through documentation integrated intellisense and namespace exploration
5. Compiles to native performance (not interpreted overhead)

---

## Part 8: Risk Mitigation

### Technical Risks

**Risk: LLVM complexity**
- **Mitigation:** Start with simple IR generation, add optimizations incrementally
- **Fallback:** Cranelift or JVM if LLVM proves too complex

**Risk: GC pauses affect 60 FPS**
- **Mitigation:** Tune GC for low latency, provide Memory namespace escape hatch
- **Fallback:** Reference counting for v1.0, improve GC in v2.0

**Risk: Cross-platform issues**
- **Mitigation:** SDL2 handles most platform differences
- **Fallback:** Target one platform for v1.0, expand later

### Scope Risks

**Risk: Feature creep**
- **Mitigation:** Lock namespace API in Phase 1, no additions until v2.0
- **Enforcement:** API review board (even if one person)

**Risk: Asset quality insufficient**
- **Mitigation:** Budget for professional assets upfront
- **Fallback:** Curate high-quality CC0 assets, credit properly

### Adoption Risks

**Risk: Too hard for target age**
- **Mitigation:** Test with kids monthly, iterate on examples/tutorial
- **Pivot:** Adjust target age up if needed

**Risk: Not engaging enough**
- **Mitigation:** Focus on games/visuals (intrinsically motivating)
- **Pivot:** Add more example types (art, music, simulations)

---

## Part 9: Developer Experience Gaps (Previously Missing)

### Debugging Story

The roadmap must address how users debug programs. Without a debugger, beginners will hit walls and abandon the language.

**Strategy: Two-tier debugging**

**Tier 1: Built-in diagnostic tools (Day 1)**
- `System.Log(value)` — prints any value with type info to a debug console
- `System.Inspect(namespace)` — dumps namespace state (e.g., `System.Inspect(Screen)` shows all layers, sprites, positions)
- `System.Slow(0.5)` — runs program at half speed so beginners can see what's happening
- `Memory.Stats()` — already in the roadmap, shows allocations/GC activity
- Visual overlay: `System.Debug(true)` draws sprite bounding boxes, layer boundaries, FPS counter

**Tier 2: VS Code debug adapter (Month 10+)**
- Implement DAP (Debug Adapter Protocol) in the LSP server
- LLVM can emit DWARF debug info (desktop) — map source locations to IR via `DIBuilder`
- Wasm target: use Chrome DevTools protocol or source maps
- Breakpoints, step-through, variable inspection, call stack
- Custom debug visualizers for namespace objects (show a sprite's texture inline, play a Sound inline)

**Teaching moment:** "A debugger lets you freeze time and look inside your program"

### Hot Reload

For a language promising "immediate visual feedback," compile-edit-restart is too slow. Users should see changes reflected live.

**Implementation approach:**

```
┌──────────────┐     ┌────────────────┐     ┌──────────────┐
│ File watcher  │────▶│ Incremental    │────▶│ Runtime      │
│ (notify/      │     │ recompile      │     │ hot-swap     │
│  chokidar)    │     │ (changed fn    │     │ (patch fn    │
│               │     │  only)         │     │  pointers)   │
└──────────────┘     └────────────────┘     └──────────────┘
```

- **Desktop:** Recompile changed functions to shared library (.dylib/.so/.dll), `dlopen`/`dlsym` to swap function pointers at runtime. State (sprites, sounds, variables) survives the reload.
- **Web:** Recompile changed module to Wasm, use `WebAssembly.instantiate()` to swap. PixiJS scene graph persists.
- **Scope:** Only function bodies hot-reload. Struct/type changes require full restart (with clear message explaining why).
- **Fallback:** If hot reload is too complex for v1.0, implement fast-restart instead — full recompile but restore program state from a snapshot.

**Timeline:** Prototype in Month 8, stable by Month 12.

### Code Sharing & Imports

The roadmap specifies single-file execution, but users will eventually want to split code and share it.

**Phase 1 (v1.0): Local imports only**
```
import "enemies.gb"        # Relative path, merged into compilation
import "utils/helpers.gb"  # Subdirectory
```
- No package manager, no registry, no versioning
- The compiler resolves imports as additional source files in the same compilation unit
- Teaching moment: "Your program can be split across files — they're stitched together before compiling"

**Phase 2 (v2.0+): Package registry**
- Central registry (like crates.io or npm) for community libraries
- `gbasic.toml` manifest file for multi-file projects
- Semantic versioning, dependency resolution
- **Only pursue this if adoption metrics justify it** — premature package management adds complexity without value for beginners

### Compiler Testing & Quality

A teaching language compiler must be exceptionally reliable. Bugs in the compiler destroy trust.

**Testing strategy:**

| Layer | Approach | Tooling |
|-------|----------|---------|
| **Lexer/Parser** | Snapshot tests — input source → expected AST | `insta` (Rust) or custom harness |
| **Type checker** | Positive + negative cases, expected error messages | Same harness |
| **IR generation** | FileCheck-style tests — verify LLVM IR patterns | LLVM `FileCheck` or `lit` |
| **End-to-end** | `.gb` source → compile → run → assert stdout/exit code | Custom test runner |
| **Error messages** | Golden file tests — exact error output comparison | Snapshot testing |
| **Fuzzing** | Random/mutated source → compiler must not crash | `cargo-fuzz` / `libFuzzer` |
| **Performance** | Compile-time and runtime benchmarks, tracked over time | `criterion` + CI dashboard |

**Minimum bar for any release:**
- Zero compiler crashes on valid input
- Zero compiler crashes on invalid input (must always produce an error message)
- All example programs compile and run correctly on both targets
- Error messages tested against golden files to prevent regressions

**CI pipeline:** Every PR runs full test suite on Linux + macOS + Windows. Web target tested via headless Chromium.

---

## Part 10: Critical Path (Updated)

**Longest poles:**

1. **Language specification** → Parser → Code generation (Months 1-3)
   - Everything depends on this
   - Cannot rush without tech debt

2. **Core namespaces** → Examples → User testing (Months 3-10)
   - Must be stable before IDE work completes
   - Examples validate the API design

3. **IDE + Assets** → Beta testing → Launch (Months 10-12)
   - IDE makes or breaks user experience
   - Assets determine first impression quality

**Target beta release:**
1. All 7 core namespaces working
2. 20+ high-quality bundled assets
3. Working IDE (VS Code extension)
4. 10+ example programs
5. Getting started tutorial

---

## Part 11: Bottom Line

### What We're Building
A **teaching-first systems language** that makes machine behavior visible and efficient code natural, bundled with professional assets and an integrated IDE.

### How We're Building It
**Reuse proven infrastructure** (LLVM, SDL2, GC, VS Code) and **innovate only on namespace design and teaching integration**.

### Success Looks Like
- 2000+ active users at 24 months
- 50%+ can explain memory/performance concepts
- Programs run at 60 FPS without expert knowledge
- Users graduate to Unity/Rust/C++ with strong foundations
- Teachers adopt for CS education

### The Moat
**Not the language syntax** (can be copied).
**Not the IDE** (can be replicated).
**The moat is the integrated teaching experience:**
- Namespace design that maps to hardware
- Bundled professional assets
- Examples that teach implicitly
- Performance transparency
- Progression from beginner to efficient systems programmer

This is a **10-year product** if v1.0 succeeds. The roadmap gets us to year 2.