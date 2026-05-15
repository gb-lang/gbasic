# G-Basic Progress Tracker

> Last updated: 2026-02-18 — Deep review: CLI, runtime/desktop, runtime/web, irgen, web_glue all audited. Accurate status for all items.

## Kids-Launch Sprint State

> Loop-driven sprint per `docs/kids-launch-timeline.md`. Updated every tick by the `/loop` skill following the control flow in `~/.claude/plans/i-want-a-loop-fuzzy-bentley.md`.

| Field | Value |
|-------|-------|
| current_day | 6 |
| current_phase | day_done |
| gate_status | awaiting_review |
| last_pr | pending (Day 6 — polish + deploy readiness) |
| last_tick | 2026-05-15 — Day 6: welcome band, errors/loading, telemetry, rate limit |
| blocker | — |

**Day 6 — done:**
- Added compact welcome band with Start Learning link and example preview chips
- Added inline compiler error panel above the editor
- Added visible Run button loading state while compiling
- Refined mobile fallback behavior around the new welcome band/header
- Added anonymous `POST /telemetry` endpoint to the compile service
- Added client telemetry for compile success/failure and lesson completion
- Added compile-service rate limit: 10 compile requests/minute/IP
- Updated README and compile-service docs with playground/deploy-readiness notes

**Day 7 — next:**
- Launch checklist, ops handoff, rollback/hotfix notes
- Browser/Chromebook/manual QA checklist and v0.3.0-kids tag instructions

**Day 5 — done:**
- Share button now encodes the editor program into the URL hash as `#code=...&title=...`
- Shared URLs load back into the editor on page open
- Share flow supports non-ASCII source via `TextEncoder`/`TextDecoder`
- Optional title prompt included in the share URL
- Added `compiler/cli/tests/canonical_games.rs` to type-check `pong.gb`, `flappy.gb`, and `angrybirds.gb`
- Added `compiler/cli/tests/lesson_fixtures.rs` to type-check all six starter/solution lesson programs

**Day 4 — done:**
- Added six lesson markdown files under `playground/lessons/`
- Added starter and solution `.gb` fixtures for each lesson
- Added `playground/lessons/manifest.json`
- Playground now has a lesson panel with title, goal, progress dots, Prev/Next, Load starter, Show solution, and Done actions
- Supports `#/learn/N` hash routes and also recognizes `/learn/N` when hosted with a fallback
- Lesson completion is stored locally in `localStorage`
- Lesson code loads directly into the Monaco editor

**Day 3 — done:**
- Added a CC0 starter asset pack with 15 generated sprites and 10 generated sound effects
- Root runtime assets live in `assets/{sprites,sounds}/`; web-playground copies live in `playground/assets/{sprites,sounds}/`
- Added `assets/manifest.json` and `playground/assets/manifest.json` for asset discovery
- Added `assets/CREDITS.md` and `playground/assets/CREDITS.md`
- Added `sprite("name")` Layer 1 shortcut, desugaring to `Asset.Sprite("name")`
- Added `Asset.Sprite(name)` and `Asset.Sound(name)` compiler/runtime support
- Web runtime now resolves simple asset names, explicit paths, and extensionless custom paths
- Desktop runtime resolves simple sprite/sound names against bundled `assets/` fallbacks
- Playground asset picker loads the manifest and inserts `sprite("hero")` / `play("jump")` snippets
- `examples/sprite_demo.gb` and `examples/sound_demo.gb` now use bundled name-based assets

**Day 2 — done:**
- `runtime_screen_sprite_load` (web): async fetch with `.png` / `.jpg` / `.jpeg` candidates from `assets/sprites/<name>`; cached by handle index in `state.sprites[]`; drawing before load completes is a silent no-op
- `runtime_screen_sprite_at` / `_scale`: sprite property mutation, returns handle for chaining
- `runtime_screen_sprite_draw`: `ctx.drawImage(image, x, y, w*scale, h*scale)`; respects `ready` flag
- `runtime_sound_effect_load` (web): pre-warms `state.soundCache` (Map<name, AudioBuffer>) via fetch + `decodeAudioData`; tries `.wav` / `.mp3` / `.ogg`
- `runtime_sound_effect_play` (web): plays buffered sound through a per-call `GainNode`; lazy `AudioContext.resume()` for autoplay-policy safety
- `runtime_sound_effect_volume` (web): name-keyed `state.soundVolumes` map applied at play time
- `state.assetSpriteRoot` / `assetSoundRoot` are host-overridable so the playground can point to its own asset dir
- `loadAndRunBytes(wasmBytes)` runtime entrypoint added so the playground can execute compile-service responses without writing a temporary `.wasm` URL
- Playground Run now mounts each compiled program in a fresh sandboxed iframe and calls the returned runtime + WASM bundle immediately
- Playground Stop removes the iframe, which tears down the running WASM instance, animation loop, and audio context
- The stale "runtime wiring lands Day 4" placeholder was removed; Day 4 can build lessons on top of the same sandbox runner

`cargo check -p gbasic-irgen --no-default-features` clean.

**Day 2 afternoon (morning+afternoon folded — natural single change):**
- Sound (load/play/volume) was the afternoon's main work — landed alongside sprite in the same edit because both share the same `state` extensions and helper-fn pattern
- `web_parity.rs` already passing (the names were on the declared list pre-existing; only the bodies changed)
- BMP-only restriction was desktop-side only; web now supports PNG/JPG/JPEG with extension probing
- `IO.read_file` web stub intentionally left returning empty

**Playground execution wiring (Day 1 carryover):**
- Completed in Day 2 correction. The playground now executes compile-service bundles in a per-run iframe sandbox.

**Day 1 — done:**
- `playground/` static site (Monaco + canvas + Run/Stop/Share)
- `services/compile/` axum service: `POST /compile` → `{wasm,js,errors?}`
  - 1MB source cap, 5s timeout, 5MB output cap, per-request `TempDir`
  - Picks `game_async.wasm` if present, else `game.wasm`
  - `/healthz` for liveness
- `services/compile/Dockerfile` (LLVM 18 + lld + binaryen + Rust build → slim Debian runtime, non-root user)
- Playground `Run` button now calls `POST /compile`, base64-decodes the wasm, logs round-trip stats — actual WASM instantiation deferred to Day 2 (where the runtime sprite/sound work lives)
- `cargo check -p gbasic-compile-service` passes locally

**Carryover into Day 2:**
- Replace the placeholder "compile OK — runtime wiring lands Day 2" canvas message with `eval(result.js)` + `WebAssembly.instantiate(wasmBytes)` once the web runtime sprite + sound paths are real

**Defaults locked in (per plan §Open Questions):**
- Cadence: self-paced
- Day-end gate: strict (loop holds until merge)
- Asset license: Kenney CC0 only
- Hosting: deferred — will request Chibueze decision on the Day 5 PR

## Legend
- ✅ Done — fully implemented, tested
- 🟡 Partial — implemented but has known gaps
- ❌ Not started — stub or missing
- 🔴 Broken — exists but known incorrect/non-functional

---

## 1. Compiler Pipeline

### 1.1 `compiler/common` — Shared AST & Types
| Item | Status | Notes |
|------|--------|-------|
| All AST node types (Program, Statement, Expression) | ✅ | Let, If, While, For, Match, Return, Break, Continue, Function, Block |
| MethodChain / NamespaceRef (8 namespaces) | ✅ | Screen, Sound, Input, Math, System, Memory, IO, Asset |
| StringInterp, Range, Array, FieldAccess, Assignment, Index | ✅ | |
| Type enum (Int, Float, String, Bool, Void, Array, Function, Unknown, Point, Color) | ✅ | `Point` and `Color` are proper variants with Display impl |
| GBasicError (Syntax, Type, Name, Codegen, Internal) | ✅ | |
| Span | ✅ | |
| `shortcuts.rs` Layer 1 alias table (14 entries) | ✅ | Static lookup used by parser for desugaring. Covers print, clear, random, abs, sqrt, sin, cos, key, play, log, wait, clamp, line |

### 1.2 `compiler/lexer` — Tokenizer
| Item | Status | Notes |
|------|--------|-------|
| All keywords, 8 namespace tokens, type keywords | ✅ | |
| All operators, delimiters, `..`, `->` | ✅ | |
| Int / Float / String literals | ✅ | |
| Case-insensitive keywords (lowercased via logos) | ✅ | |
| String escape sequences (`\n`, `\t`, `\\`, `\"`, `\{`, `\}`) | ✅ | |
| Single-line `//` and block `/* */` comments | ✅ | |
| Error recovery (bad char → `Token::Error`, continues) | ✅ | |
| `+=`, `-=`, `*=`, `/=` compound assignment tokens | ✅ | |
| **Tests** | ✅ | 13 unit tests + 5 snapshot tests |

### 1.3 `compiler/parser` — Recursive Descent
| Item | Status | Notes |
|------|--------|-------|
| `let`, `fun`/`fn`, `if`/`else`, `while`, `for`, `match`, `return`, `break`, `continue` | ✅ | |
| Namespace method chains (`Screen.Init(800,600)`) | ✅ | |
| Field access, assignment, array literals, index, string interpolation | ✅ | |
| Range expressions (`0..10`, `0 to 10`) | ✅ | `to` desugars to `0..(n+1)` |
| All binary operators with correct precedence, unary `-`/`not` | ✅ | |
| Newline-as-statement-terminator, error recovery | ✅ | |
| Layer 1 shortcut desugaring in parser (14 shortcuts) | ✅ | All shortcuts desugared in `parse_postfix` via `SHORTCUTS` table → canonical `MethodChain` AST nodes |
| `+=`, `-=`, `*=`, `/=` compound assignment | ✅ | Desugars to `Assignment(target, BinaryOp(target, op, rhs))` |
| **Tests** | ✅ | 42 unit tests + 5 snapshot tests |

### 1.4 `compiler/typechecker` — Type Checker
| Item | Status | Notes |
|------|--------|-------|
| Scoped symbol table (push/pop scope, IndexMap stack) | ✅ | |
| Literal inference, `let` + type annotation check | ✅ | |
| Function decl + arity + argument type checks | ✅ | |
| `if`/`while` condition must be Bool | ✅ | |
| `for` range → Int, `for` array → element type | ✅ | |
| Assignment type compatibility | ✅ | |
| Binary ops (arithmetic, comparison, logical), Int/Float promotion | ✅ | |
| String concat via `+`, unary ops, `match`, `break`/`continue` | ✅ | |
| String interpolation (all parts checked) | ✅ | |
| Layer 1 builtins registered (all 14 shortcuts + `rect`, `circle`, `point`, `color`) | ✅ | All shortcuts + value constructors registered as `Type::Unknown` return |
| Named color constants (12 colors as `Type::Color`) | ✅ | Fixed: was `Type::Int`, now correctly `Type::Color` |
| `mouse` / `screen` pseudo-objects registered | ✅ | Registered as `Type::Unknown` for lenient field access |
| `MethodChain` return type inference | 🟡 | Returns `Type::Unknown` for most chains; only a few special-cased (key→Bool, collides→Bool) |
| `FieldAccess` return type inference | 🟡 | Returns `Type::Unknown` for object properties; no per-property type tracking |
| Return type validation (body vs declared `-> Type`) | ✅ | `current_return_type` context; `return` validates against declared type |
| Mutability enforcement | ✅ | Named colors and for-loop vars are immutable; assignment is a type error |
| `break`/`continue` outside loop detection | ✅ | `loop_depth` counter; error at depth 0 |
| **Tests** | ✅ | 35 unit tests |

### 1.5 `compiler/irgen` — LLVM Code Generation (~2781 LOC)
| Item | Status | Notes |
|------|--------|-------|
| LLVM context/module/builder (inkwell 0.5 / LLVM 18) | ✅ | |
| `let` → alloca + store, variable load | ✅ | |
| All binary ops (int + float + mixed Int/Float promotion) | ✅ | |
| Unary ops, `if`/`else`, `while`, `for` range, `for` static array, `for` dynamic array | ✅ | |
| `match` (literal, identifier, wildcard patterns) | ✅ | |
| `break`/`continue` (loop exit stack) | ✅ | |
| Function declaration + call (two-pass: declare then body) | ✅ | |
| Implicit return (last expression), explicit `return` | ✅ | |
| String literals, string interpolation (print + `.at()`), string concat | ✅ | |
| Named colors → packed i64 (12 colors) | ✅ | |
| Object constructors: `rect(w,h)`, `circle(r)` | ✅ | `Call` nodes handled in `codegen_call` |
| Object property setters: `.position`, `.position.x/y`, `.color`, `.velocity`, `.velocity.x/y`, `.gravity`, `.solid`, `.bounces`, `.visible`, `.layer` | ✅ | |
| Object property getters: `.position.x/y`, `.velocity.x/y`, `.size.width/height`, `.length` | ✅ | |
| Object methods: `.move()`, `.collides()`, `.contains()`, `.remove()`, `.add()` | ✅ | |
| `Screen.center`, `.bottom_center`, `.top_center`, `.top_left`, `.top_right`, `.bottom_left`, `.bottom_right` | ✅ | All 7 position constants wired in `codegen_method_chain` and `codegen_field_access_read` |
| `Screen.width` / `Screen.height` | ✅ | Via `codegen_field_access_read` for `screen.width/height` |
| Namespace method chains (Math, Screen, Input, Sound, System, Memory, IO, Asset) | ✅ | `get_namespace_method` dispatch table + special-case branches in `codegen_method_chain` |
| Auto-frame loop (`while true` → `runtime_frame_auto` + `runtime_frame_auto_end`) | ✅ | Only outermost `while true` triggers auto-frame; nested loops unaffected |
| `ensure_screen_init` (lazy SDL2 init) | ✅ | Called at start of auto-frame and in all Screen/Input shortcuts |
| Desktop target: emit object + link via `cc`/`gcc` | ✅ | Workspace root auto-detected; tries release then debug lib; OS-aware SDL2 flags |
| Web target: WASM + wasm-ld + wasm-opt asyncify + JS/HTML glue | ✅ | `emit_wasm` + `generate_web_output`; asyncify non-fatal if wasm-opt missing |
| LLVM IR dump (`--dump-ir`) | ✅ | |
| `point(x,y)` constructor | � | **Bug:** `codegen_call` for `point` discards `y` and returns only `x`. Only works correctly when used as arg to `codegen_property_set` (position/velocity). Standalone `let p = point(1,2)` is broken. |
| `print("...").at(x, y)` → `runtime_draw_text` | ✅ | Lookahead in `codegen_method_chain`; `build_interp_string` builds concatenated string ptr |
| `print(non-string).at(x, y)` | 🔴 | **Bug:** `codegen_print_at` for non-string types (Int/Float) silently returns empty string instead of converting to string. Score display `print("Score: {score}").at(10,10)` works (StringInterp), but `print(score).at(10,10)` does not. |
| `at()` on object method call path | 🔴 | **Bug:** `codegen_object_method` for `"at"` is a dead stub — discards x/y and returns None. This path is only hit if `.at()` is called on a non-print object; the print+at path is handled correctly in `codegen_method_chain`. |
| `remove_from` array method | 🟡 | Codegen handles `obj.remove_from(arr)` but guardrails spec uses `arr.remove(item)` — API mismatch |
| `Screen.Layer(n)` — layer index | 🟡 | `layer` step is skipped (calls `ensure_screen_init` and continues); layer index `n` is silently ignored |
| `line(from, to)` with Point args | 🔴 | **Bug:** Takes 4 scalar args `(x1,y1,x2,y2)` hardcoded; color always white. Spec requires `line(from, to)` with Point objects. |
| `infer_expr_type` for named colors | 🟡 | Returns `Type::Int` (packed RGB) in irgen, but typechecker returns `Type::Color`. Inconsistency is harmless for now but could cause issues if type-gated codegen is added. |
| String match equality | 🟡 | `build_equality_check` for strings uses pointer equality (not string content comparison). `match s { "hello" -> ... }` will always fail at runtime. |
| Windows/Linux/macOS linker flags | ✅ | `emit_and_link` detects OS; MinGW gcc preferred on Windows; full SDL2 framework flags on macOS |
| **Tests** | � | Only 1 placeholder test (`test_stub_passes`). No real irgen tests — all codegen changes are untested without LLVM 18. |
| **`compiler/irgen/src/lib.rs`** | 🟡 | `codegen()` silently returns error if `llvm` feature not enabled — CLI will fail with unhelpful message if built without feature |

---

## 2. Runtime

### 2.1 `runtime/desktop` — SDL2 Runtime (~1366 LOC)
| Item | Status | Notes |
|------|--------|-------|
| SDL2 window init (`runtime_screen_init`) | ✅ | Uses `present_vsync`; canvas + event_pump stored in `SDL_STATE` thread-local |
| Screen clear, draw rect, draw line, draw circle, present, set pixel | ✅ | All implemented; circle uses midpoint algorithm |
| Screen width/height, center_x/center_y | ✅ | |
| Sprite load/at/scale/draw | 🟡 | BMP only via `Surface::load_bmp`; no PNG/JPG; texture recreated every draw call (perf issue) |
| Input poll (keyboard + mouse + quit) | ✅ | `KEY_STATE` HashMap; `MOUSE_STATE` = `(x, y, clicked)` |
| `runtime_input_key_pressed`, `runtime_input_mouse_x/y/clicked` | ✅ | |
| Math: sin, cos, sqrt, abs, floor, ceil, pow, max, min, random, pi, random_range, clamp | ✅ | xorshift64 RNG; lazy-seeded from `SystemTime::now()` |
| System: time, sleep, wait, exit, frame_begin, frame_end, frame_time, log | ✅ | 60 FPS cap in `frame_end` and `frame_auto_end` |
| Sound: beep (synthesized), effect_load/play/volume | ✅ (feature-gated) | `#[cfg(feature = "mixer")]`; stubs print to stderr otherwise |
| Asset: load | � | **Stub** — logs `"(stub — asset caching not yet implemented)"` and returns 0 |
| Memory: set/get (HashMap key-value store) | ✅ | |
| IO: read_file, write_file | ✅ | `read_file` returns null on error (caller must handle) |
| Print (string, int, float, parts, newline), string concat, int/float to string | ✅ | |
| `ensure_screen_init` (lazy auto-init 800×600) | ✅ | `SCREEN_AUTO_INIT` Cell prevents double-init |
| Object system (Rect/Circle, handle = Vec index as i64) | ✅ | `OBJECTS` thread-local Vec; `alive` flag for soft-delete |
| Object property setters: position, color, velocity, gravity, solid, bounces, visible, layer | ✅ | |
| Object property getters: position.x/y, velocity.x/y, size.width/height | ✅ | |
| Object methods: move, collides (AABB), contains, remove | ✅ | |
| Physics step: gravity, velocity, screen-edge bounce, solid-object bounce | ✅ | Solid-object bounce uses overlap-axis heuristic |
| Auto-draw (sorted by layer, alive+visible objects) | ✅ | |
| Frame auto (`runtime_frame_auto` + `runtime_frame_auto_end`) | ✅ | auto: poll+quit-check; auto_end: physics+draw+present+60fps |
| Dynamic arrays (new, add, get, length, remove_value) | ✅ | `DYN_ARRAYS` Vec of Vec<i64>; handle = index |
| Text drawing (`runtime_draw_text`) | ✅ | 5×7 bitmap font at 2× scale; A-Z, 0-9, common punctuation |
| Legacy compat functions (`runtime_init`, `runtime_clear_screen`, `runtime_present`, `runtime_should_quit`, `runtime_shutdown`) | ✅ | Thin wrappers; kept for backward compat |
| **Bundled sound assets** | ❌ | No `.wav` files ship with runtime; `play("bounce")` calls `effect_play` which tries to load from CWD and fails silently |
| **Memory leak** in string concat/int_to_str/float_to_str | 🟡 | `CString::into_raw()` intentionally leaks — no GC in MVP |
| **`runtime_screen_clear_color`** | 🟡 | Exists but not called by codegen — `codegen_method_chain` calls `runtime_screen_clear` directly |
| **`runtime_math_random_range` off-by-one** | 🟡 | Uses `% (max - min + 1)` which is inclusive of max — matches spec but differs from `random(min, max)` exclusive convention |
| **Sprite texture recreated per draw** | 🟡 | `runtime_screen_sprite_draw` clones surface data and creates a new texture every frame — O(n) allocations per sprite per frame |

### 2.2 `runtime/web` — WASM Runtime
| Item | Status | Notes |
|------|--------|-------|
| Crate exists as workspace member | ✅ | Empty lib.rs — actual JS glue lives in `irgen/src/web_glue.rs` |
| JS glue generation (`runtime.js` + `index.html`) | ✅ | `generate_web_output()` writes both files to output dir |
| JS implementations of all runtime functions | ✅ | `RUNTIME_JS` static string embedded in binary; ~480 LOC of JS |
| WASM asyncify support (via `wasm-opt`) | ✅ | `runtime_frame_auto_end` triggers asyncify unwind; `requestAnimationFrame` drives rewind |
| Asyncify fallback if wasm-opt missing | ✅ | Non-fatal warning; falls back to synchronous (blocking) execution |
| Parity test (`web_parity.rs`) | ✅ | `test_runtime_function_parity` checks JS glue covers all declared names; `test_all_codegen_runtime_calls_covered` scans llvm_backend.rs |
| **`runtime_input_mouse_clicked`** | ❌ | **Missing from JS glue** — not in `js_runtime_function_names()` list and not implemented in `RUNTIME_JS`. Parity test will pass because it's not in the declared list, but codegen calls it. |
| **`runtime_math_clamp`** | ❌ | **Missing from JS glue** — not in `js_runtime_function_names()` and not in `RUNTIME_JS`. Same gap. |
| **`runtime_system_wait`** | ❌ | **Missing from JS glue** — not in `js_runtime_function_names()` and not in `RUNTIME_JS`. |
| **`runtime_system_log`** | ❌ | **Missing from JS glue** — not in `js_runtime_function_names()` and not in `RUNTIME_JS`. |
| **`runtime_screen_clear_color`** | ❌ | Missing from JS glue (not called by codegen either, so low priority) |
| Sound effect load/play/volume | 🟡 | JS stubs return 0 / no-op — no Web Audio implementation for file-based sounds |
| Sprite load/at/scale/draw | 🟡 | JS stubs return 0 / no-op — no canvas image rendering |
| Asset load | 🟡 | JS stub returns 0 |
| IO read_file | 🟡 | JS stub returns empty string — no file access in browser |
| `wasm_alloc` bump allocator | ✅ | Emitted by `emit_wasm_alloc`; used by JS `writeCStr` for string returns |

---

## 3. CLI (`compiler/cli`)
| Item | Status | Notes |
|------|--------|-------|
| Full pipeline: lex → parse → typecheck → codegen | ✅ | |
| `--dump-tokens`, `--dump-ast`, `--dump-ir`, `--check`, `--skip-typecheck` | ✅ | |
| `--output`, `--run`, `--target desktop\|web` | ✅ | |
| Pretty error reporting with source spans (codespan-reporting) | ✅ | |
| **No-file invocation** | 🟡 | `gbasic` with no args silently returns (no help text shown) |
| **E2E tests** (`compiler/cli/tests/e2e.rs`) | 🟡 | 9 tests cover basic language features; require LLVM 18 + SDL2 to run; no game-loop or object tests |
| **Web parity tests** (`compiler/cli/tests/web_parity.rs`) | 🔴 | `test_all_codegen_runtime_calls_covered` will fail: `runtime_input_mouse_clicked`, `runtime_math_clamp`, `runtime_system_wait`, `runtime_system_log` are called in codegen but absent from `js_runtime_function_names()` |
| **Error golden tests** (`compiler/cli/tests/error_golden.rs`) | 🟡 | Exists; coverage unknown without running |

---

## 4. Examples Status

| Example | Parses | Type-checks | Compiles | Notes |
|---------|--------|-------------|----------|---------|
| `hello.gb` | ✅ | ✅ | ✅ | Verified by e2e tests |
| `arithmetic.gb` | ✅ | ✅ | ✅ | Verified by e2e tests |
| `basics.gb` | ✅ | ✅ | ✅ | |
| `control_flow.gb` | ✅ | ✅ | ✅ | |
| `namespaces.gb` | ✅ | ✅ | 🟡 | Namespace chains work; not verified on LLVM 18 |
| `math_viz.gb` | ✅ | ✅ | 🟡 | Not verified on LLVM 18 |
| `sound_demo.gb` | ✅ | ✅ | 🟡 | `play()` fails silently (no bundled assets) |
| `sprite_demo.gb` | ✅ | ✅ | 🟡 | BMP only, no bundled sprites |
| `particles.gb` | ✅ | ✅ | 🟡 | Not verified on LLVM 18 |
| `pong.gb` | ✅ | ✅ | 🟡 | Codegen complete; not verified on LLVM 18 + SDL2 |
| `flappy.gb` | ✅ | ✅ | 🟡 | Codegen complete; `pipe.remove()` during `for pipe in pipes` is unsafe (corrupts index) |
| `angrybirds.gb` | ✅ | ✅ | 🟡 | Codegen complete; not verified on LLVM 18 + SDL2 |

---

## 5. Design Conformance vs. `docs/guardrails.md`

| Design Rule | Status | Notes |
|-------------|--------|---------|
| Layer 1 shortcuts (print, rect, circle, key, play, clear, random, abs, sqrt, sin, cos, wait, log, clamp, line) | ✅ | All desugared by parser; handled in codegen. `rect`/`circle` remain `Call` nodes. |
| Layer 2 OO (`let paddle = rect(100,20)`, `.position`, `.color`) | ✅ | |
| Layer 3 full namespace chains | 🟡 | `Screen.Layer(0)` handled; layer index silently ignored; `Screen.Layer(1).Rect(...)` not supported |
| Physics-as-properties (`.velocity`, `.gravity`, `.solid`, `.bounces`) | ✅ | |
| `ball.collides(paddle)` built-in collision | ✅ | AABB; circle uses bounding box (not true circle collision) |
| Named colors (12) | ✅ | black, white, red, green, blue, yellow, orange, purple, pink, cyan, gray/grey, brown |
| `print("Score: {score}").at(10, 10)` | ✅ | StringInterp path works; plain `print(score).at(10,10)` broken (non-string silently becomes empty) |
| `Screen.center` / all 7 position constants | ✅ | |
| Auto-draw, `clear()` only clears background | ✅ | |
| Arrays: `[]`, `.add()`, `.remove()`, `.length`, `for item in array` | 🟡 | `arr.remove(item)` maps to `remove_value` (by value); `obj.remove()` maps to `runtime_object_remove` (by handle) — different semantics, both work |
| `Point(x,y)`, `Color(r,g,b)` value constructors | � | Work only in `.position`/`.velocity`/`.color` assignment context; `point()` standalone discards y; no real value type in codegen |
| `velocity = (vx, vy)` tuple syntax | ✅ | Parser desugars `(x,y)` → `Call{"point"}` which `codegen_property_set` handles |
| `mouse.x` / `mouse.y` / `mouse.clicked` | ✅ | Wired in desktop runtime; missing from web JS glue |
| `line(from, to)` with Point args | � | Takes 4 scalars, not Point objects; color hardcoded to white |
| `wait()`, `log()`, `abs()`, `sqrt()`, `sin()`, `cos()`, `clamp()` shortcuts | ✅ | Desktop runtime: all wired. Web JS glue: `wait`/`log`/`clamp` missing. |
| Bundled sound/sprite assets | ❌ | No bundled files; `play()` fails silently |
| `Asset.Sound()` / `Asset.Sprite()` | ❌ | `Asset.load()` is a stub in both desktop and web |
| No boilerplate (no `Screen.Init()` required) | ✅ | `ensure_screen_init()` handles lazy init |
| `print("Score: {score}")` without `.at()` | ✅ | Prints to stdout (terminal); does not appear on game screen |

---

## 6. Infrastructure

| Item | Status | Notes |
|------|--------|---------|
| Cargo workspace (8 crates) | ✅ | common, lexer, parser, typechecker, irgen, cli, runtime/desktop, runtime/web |
| `rust-toolchain.toml` | ✅ | |
| LLVM 18 dependency (inkwell 0.5) | ✅ (declared) | Requires `LLVM_SYS_180_PREFIX` env var on Windows; `llvm` feature flag on irgen |
| SDL2 bundled feature | ✅ (declared) | `sdl2` with `bundled` feature; requires CMake on Windows |
| `docs/guardrails.md`, `docs/grammar.md`, `docs/gap-analysis-and-timeline.md` | ✅ | |
| `roadmap.md` | ✅ | |
| Snapshot tests (insta) | ✅ | 5 snapshot files in `compiler/parser/tests/snapshots/` |
| E2E integration tests | 🟡 | `compiler/cli/tests/e2e.rs` (9 tests), `e2e_web.rs`, `error_golden.rs`; require LLVM 18 to run |
| Web parity tests | 🔴 | `web_parity.rs` will fail: 4 runtime fns called by codegen missing from JS glue list |
| CI / GitHub Actions | ❌ | No `.github/` directory |
| Windows build instructions | ❌ | README doesn't cover LLVM 18 + SDL2 + CMake setup on Windows |

---

## 7. Bugs Found in Deep Review (Priority Order)

### 🔴 P1 — Newly identified, blocking correctness

1. 🔴 **Web JS glue missing 4 runtime functions** — `runtime_input_mouse_clicked`, `runtime_math_clamp`, `runtime_system_wait`, `runtime_system_log` are called by codegen but absent from `js_runtime_function_names()` and `RUNTIME_JS`. Web target will crash at runtime when these are called. **Fix:** add to both `js_runtime_function_names()` and `RUNTIME_JS`.

2. 🔴 **`point(x,y)` standalone discards y** — `codegen_call` for `"point"` evaluates `y` but returns only `x`. `let p = point(1, 2)` silently gives `p = 1`. Only works correctly as arg to `codegen_property_set`. **Fix:** pack x+y into a struct or i128, or restrict `point()` to property-set context only.

3. 🔴 **`print(non-string).at(x,y)` silently renders nothing** — `codegen_print_at` for Int/Float args returns an empty string ptr instead of converting to string. `print(score).at(10,10)` draws blank. StringInterp form `print("Score: {score}").at(10,10)` works correctly. **Fix:** call `runtime_int_to_str`/`runtime_float_to_str` in `codegen_print_at`.

4. 🔴 **String `match` always fails** — `build_equality_check` for `Type::Unknown`/`Type::String` uses pointer equality (`EQ` on i64 ptrs). `match s { "hello" -> ... }` will never match at runtime since string literals are different allocations. **Fix:** call a `runtime_string_eq` function.

5. 🔴 **`web_parity` test will fail** — `test_all_codegen_runtime_calls_covered` scans `llvm_backend.rs` for `call_runtime("name"` patterns and checks against `js_runtime_function_names()`. The 4 missing functions above will cause this test to fail.

### 🟡 P2 — Known gaps, non-blocking for basic games

6. 🟡 **`pipe.remove()` during `for pipe in pipes` corrupts iteration** — `flappy.gb` removes elements from a dynamic array while iterating; index counter skips elements. **Fix:** iterate backwards, or snapshot length, or deferred-removal queue.

7. 🟡 **No bundled sound/sprite assets** — `play("bounce")` tries to load `"bounce"` as a file path from CWD; fails silently (prints to stderr). No `.wav` files ship with the runtime.

8. 🟡 **`line(from, to)` takes 4 scalars, not Point objects** — Spec requires `line(from, to)` with Point args. Currently `line(x1,y1,x2,y2)`. Color hardcoded to white.

9. 🟡 **`Screen.Layer(n)` index silently ignored** — `layer` step in `codegen_method_chain` skips the layer index; all objects render on layer 0 regardless of `Screen.Layer(1)` etc.

10. 🟡 **`remove_from` vs `remove` API mismatch** — Codegen has `"remove_from"` method on arrays but guardrails spec uses `arr.remove(item)`. The `flappy.gb` example uses `pipe.remove()` (object remove), not array remove.

11. 🟡 **`runtime_screen_clear_color` dead code** — Exists in runtime but never called by codegen; `codegen_method_chain` calls `runtime_screen_clear` directly.

12. 🟡 **Sprite texture recreated per frame** — `runtime_screen_sprite_draw` clones surface data and creates a new SDL2 texture every draw call — O(n) allocations per sprite per frame.

13. 🟡 **`infer_expr_type` named color inconsistency** — Returns `Type::Int` in irgen (packed RGB), but typechecker returns `Type::Color`. Harmless now but could cause type-gated codegen issues.

14. 🟡 **`gbasic` with no args silently exits** — Should print usage/help. Currently returns immediately.

### ✅ Previously Fixed (for reference)

- ✅ All 14 Layer 1 shortcuts end-to-end (parser desugar → codegen dispatch)
- ✅ `print("...").at(x, y)` → `runtime_draw_text` (StringInterp path)
- ✅ `Math.clamp`, `System.wait`, `System.log`, `Screen.line` wired
- ✅ `mouse.x/y/clicked` wired in desktop runtime
- ✅ RNG lazy-seeded from `SystemTime::now()`
- ✅ OS-aware linker flags (macOS/Linux/Windows MinGW)
- ✅ Return type validation, mutability enforcement, break/continue loop detection
- ✅ `Type::Point` and `Type::Color` as proper enum variants

---

## 8. What Works End-to-End (on a machine with LLVM 18 + SDL2)

- ✅ `gbasic hello.gb` — compiles and runs (verified by e2e tests)
- ✅ `gbasic arithmetic.gb` — compiles and runs (verified by e2e tests)
- ✅ `gbasic basics.gb` / `control_flow.gb` — compile and run
- ✅ `gbasic --check pong.gb` / `flappy.gb` / `angrybirds.gb` — type-check passes
- ✅ `gbasic --dump-ast flappy.gb` — parses correctly (MethodChain nodes)
- 🟡 `gbasic pong.gb` — codegen complete; unverified on LLVM 18 + SDL2
- 🟡 `gbasic flappy.gb` — codegen complete; `pipe.remove()` during iteration unsafe
- 🟡 `gbasic angrybirds.gb` — codegen complete; unverified on LLVM 18 + SDL2
- 🔴 Web target (`--target web`) — WASM codegen complete but JS glue missing 4 functions; will crash

---

## 9. Suggested Next Steps (Ordered by Impact)

1. **Fix web JS glue** — Add `runtime_input_mouse_clicked`, `runtime_math_clamp`, `runtime_system_wait`, `runtime_system_log` to both `js_runtime_function_names()` and `RUNTIME_JS` in `web_glue.rs`. Fixes web parity test and web target correctness.
2. **Fix `print(non-string).at(x,y)`** — In `codegen_print_at`, call `runtime_int_to_str`/`runtime_float_to_str` for non-string types. Needed for `print(score).at(10,10)` pattern.
3. **Fix string match equality** — Add `runtime_string_eq(a, b) -> bool` to runtime and use it in `build_equality_check` for string types.
4. **Verify canonical games on LLVM 18 + SDL2** — All codegen fixes are in place; need binary verification of `pong.gb`, `flappy.gb`, `angrybirds.gb`.
5. **Fix `pipe.remove()` during iteration** — Snapshot array length before loop, or implement deferred-removal queue in runtime.
6. **Bundle minimal sound assets** — Silent stub `.wav` files so `play()` doesn't fail silently.
7. **Add CI workflow** — GitHub Actions: `cargo test --workspace` (lexer/parser/typechecker tests; skip irgen which needs LLVM).
8. **Windows build instructions** — README: LLVM 18 + SDL2 + CMake setup on Windows.
