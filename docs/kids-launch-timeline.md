# G-Basic: Kids-Launch Timeline (1-Week AI Sprint)

**Date:** 2026-05-08
**Goal:** Take G-Basic from "compiler is done" to **"a 7-12 year old can open a URL, type code, and see something happen"** — without installing anything.
**Owner:** Claude (executing). Chibueze (approving scope).

---

## 1. Why this doc

The compiler hit the bar set in `docs/gap-analysis-and-timeline.md` — desktop and web targets compile, 95+ tests pass, 12 examples build, CI is green. What's missing is everything between the compiler and a kid: no playground, no tutorials, no bundled art, no shareable links.

The 24-month plan in `roadmap.md` schedules this work across Months 9-12 (Phase 2 — Tooling). At Claude pace this is ~1 week of focused work, not a quarter. This doc commits to a 7-day sprint that ends with a hosted, kid-usable playground — and explicitly defers everything that doesn't move that single goal.

---

## 2. Scope

### In scope (must ship)
- **Web playground** — Monaco editor + Run button + canvas, hosted on a public URL
- **Compile-on-demand service** — small Rust HTTP endpoint that wraps `gbasic --target web` and returns the bundle
- **Web runtime polish** — finish sprite (PNG) and sound (Web Audio) paths; both are stubs today
- **Bundled asset starter pack** — 15 CC0 sprites + 10 CC0 sounds, accessible by name (`Sprite("hero")`)
- **Tutorial track** — 6 in-playground lessons from "hello" to "first game"
- **Shareable URLs** — encode program in URL so a kid can send their game to a friend
- **Verified canonical games** — `pong.gb`, `flappy.gb`, `angrybirds.gb` confirmed running end-to-end (desktop) and added to CI as smoke tests

### Out of scope (deferred — see `roadmap.md` Phase 2/3)
- VS Code extension, LSP, syntax highlighting in external editors
- Standalone Tauri IDE
- Debugger / DAP / hot reload
- `import` / multi-file projects
- GC replacement (`Memory.Pool/Stats`)
- Native installer / signed binaries
- Account system, code persistence beyond URL
- Mobile / tablet input handling

---

## 3. Daily Plan

### Day 1 (Mon): Playground scaffold + compile service

**Morning (4h)**
- New crate `playground/` (static site: HTML + Monaco + minimal JS)
- Editor pane (Monaco), output canvas, Run/Stop/Share buttons
- Wire keyboard focus so canvas captures keys when running
- Stubbed compile call — hardcoded WASM bundle from `examples/hello.gb` to verify glue

**Afternoon (4h)**
- New crate `services/compile/` — axum-based HTTP service exposing `POST /compile` (body: `.gb` source; response: `{wasm: base64, js: string, errors?: [...]}`)
- Wraps existing `gbasic --target web` (no recompile-in-browser; runs on the server)
- Sandbox: tmpfs working dir, 5s timeout, 1MB source limit, 5MB output limit
- Dockerfile so it deploys to Fly.io / Cloudflare Containers
- Wire playground → service; replace stub with real compile

**Deliverable:** Type `print("hi")` in the editor, hit Run, see `hi` rendered. Hosted locally.

---

### Day 2 (Tue): Web runtime — finish sprite + sound

The Feb 18 progress tracker flagged these as 🟡 stubs in `runtime/web`. They block any program that isn't pure shapes.

**Morning (4h)**
- `runtime_screen_sprite_load` (web): fetch image URL, `Image.decode()`, cache in JS map keyed by handle
- `runtime_screen_sprite_draw` (web): `ctx.drawImage` with position + scale
- Map `Sprite("hero")` → fetch `/assets/hero.png` from playground origin
- PNG + JPG support (BMP-only restriction is a desktop quirk — drop it for web)

**Afternoon (4h)**
- `runtime_sound_effect_load` (web): `fetch().then(decodeAudioData)` into AudioBuffer cache
- `runtime_sound_effect_play` (web): `BufferSource` connected to destination
- `runtime_sound_effect_volume` (web): GainNode in chain
- Update `web_parity.rs` test to no longer treat these as missing
- `IO.read_file` — keep stubbed, return empty (browser FS access is out of scope)

**Deliverable:** `sprite_demo.gb` and `sound_demo.gb` run in browser identically to desktop.

---

### Day 3 (Wed): Asset library + Asset namespace

The roadmap budgets $2k-5k and 1-2 months for professional asset creation. For an MVP launch we curate from existing CC0 packs (Kenney.nl, OpenGameArt).

**Morning (4h)**
- Pick 15 sprites from Kenney's "Platformer Art Deluxe" + "Animal Pack" (all CC0):
  - 4 characters (hero, robot, cat, dog), 4 objects (coin, heart, star, key), 4 tiles (grass, water, stone, brick), 3 effects (explosion, sparkle, smoke)
- Pick 10 sounds from Kenney's "Interface Sounds" + "Impact Sounds":
  - jump, coin, hit, explosion, powerup, click, win, lose, beep, swoosh
- Drop into `assets/sprites/` and `assets/sounds/` with manifest `assets/manifest.json`
- Add `assets/CREDITS.md` with attribution per asset

**Afternoon (4h)**
- Implement `Asset.Sprite("name")` and `Asset.Sound("name")` in both runtimes — currently both return 0 stubs
- Resolve names against `manifest.json` (web) / bundled path (desktop)
- Replace example-program asset paths with name-based references
- Add asset picker panel to playground (clicking a sprite inserts `Sprite("hero")` at cursor — roadmap line 444)

**Deliverable:** `play("jump")` works zero-config. Asset panel makes sprites discoverable.

---

### Day 4 (Thu): Tutorial track

Six lessons, ~10 minutes each, embedded in playground side panel.

**Morning (4h)** — Lesson content (markdown in `playground/lessons/`):
1. **Hello, screen** — `print("Hi!")`, `clear(blue)`
2. **Make it move** — `circle()`, `.position`, `.velocity`, the implicit game loop
3. **Listen for keys** — `key("left")`, branching, paddle controls
4. **Bouncing** — `.bounces`, `.collides()`, screen edges
5. **Score and text** — variables, `print("Score: {s}").at(10,10)`
6. **Your first game** — Pong from scratch, step by step

Each lesson: text + "Load starter code" button + "Show solution" button + a single concept goal stated up top.

**Afternoon (4h)**
- Lesson runner UI: prev/next, progress dots, "Try it" button that swaps editor contents
- Routing: `/learn/1`, `/learn/2`, ... so a kid can bookmark
- Track completion in `localStorage` (no account — a checkmark per finished lesson)
- Smoke-test every lesson's starter code and solution code through the compile service in CI

**Deliverable:** A 7-year-old who has never coded can hit lesson 1 and reach lesson 6 in ~60 minutes.

---

### Day 5 (Fri): Sharing + canonical-game verification

**Morning (4h)**
- URL encoding: program goes in URL hash (LZ-string compressed) so no backend storage required
- "Share" button copies link to clipboard
- "Fork" semantics: opening a shared URL puts the program in editor; saving creates a new URL
- Optional title field (`?title=My+Pong`)
- Open Graph preview image generated from a screenshot of the canvas (Day 7 if time runs out)

**Afternoon (4h)** — Verify the games actually work
- On a real LLVM 18 + SDL2 machine, run `pong.gb`, `flappy.gb`, `angrybirds.gb`
- Fix anything that breaks (the Feb tracker lists these as "codegen complete; unverified")
- Add a `cargo test --test canonical_games` that compiles each, runs for N frames headless, and asserts no panic + non-empty framebuffer
- Wire into the existing CI workflow (`.github/workflows/ci.yml`)

**Deliverable:** A kid can build a game, hit Share, send the URL to a friend, and the friend opens it and plays. The three flagship games are now CI-protected.

---

### Day 6 (Sat): Polish + landing page

**Morning (4h)**
- Landing page at `/` — one-sentence pitch, big "Start Learning" button, three example screenshots
- Error rendering — when compile fails, show the codespan-reporting output inline above the editor (red squiggle on the right line if Monaco supports it)
- Loading states — spinner during compile, "Compiling..." text
- Mobile fallback — show a "use a desktop or laptop" message under 800px width (we're not solving touch input this sprint)
- Favicon + OG image + meta tags

**Afternoon (4h)**
- Telemetry: anonymous compile-count + lesson-completion counters posted to a single endpoint (no PII, no cookies)
- Rate limit on compile service (10/min/IP) to avoid runaway bills
- Cost cap: compile service hosted on Fly.io free tier or Cloudflare Containers; budget alert at $10/mo
- README update: add "Try it in your browser →" link at the top
- DNS: point `play.gbasic.dev` (or whatever Chibueze owns) at the deploy

**Deliverable:** Public URL. README directs visitors to it. Cost is bounded.

---

### Day 7 (Sun): Bug bash + launch checklist

**Morning (4h)**
- Run all 6 lessons end-to-end; fix any rough edge
- Test on Chrome, Safari, Firefox (desktop)
- Test on a real Chromebook (most school computers are Chromebooks — non-negotiable for the target audience)
- Fix any browser-specific WASM/Audio quirks
- Verify share URLs round-trip correctly with non-ASCII titles

**Afternoon (4h)**
- Tag `v0.3.0-kids` milestone
- Write `LAUNCH.md`: how to redeploy, how to roll back, how to hot-fix a lesson, who has admin access
- Hand-off doc for Chibueze covering ops + costs + how to add new lessons
- Soft-launch: post to ~5 friends-with-kids; collect first 24h of feedback before broader announce

**Deliverable:** A real kid can use it. Chibueze can run it without Claude.

---

## 4. Success Criteria

| Criterion | Target |
|-----------|--------|
| Public URL reachable, < 2s first paint | Yes |
| `print("hi")` from cold editor → output in browser | < 4s end-to-end |
| All 6 lessons completable by a non-coder in < 75 min total | Yes |
| `pong.gb`, `flappy.gb`, `angrybirds.gb` run on desktop without crashing | Yes |
| Share URL round-trips a 200-line program | Yes |
| Asset starter pack: 15 sprites + 10 sounds, properly attributed | Yes |
| Works on a stock Chromebook with no extensions | Yes |
| Compile service stays under $10/mo at 1k compiles/day | Yes |
| CI smoke-tests all canonical games + all lesson starter/solution pairs | Yes |

---

## 5. Risks & Mitigations

| Risk | Mitigation |
|------|------------|
| Compile service abuse (mining, DoS) | 5s CPU timeout, 1MB source limit, 10/min/IP rate limit, sandboxed tmpfs |
| WASM size too large for slow connections | wasm-opt -Oz already in pipeline; lazy-load asset bundles per lesson |
| Web Audio autoplay policy blocks sound until user gesture | Defer `AudioContext.resume()` to first Run click — already standard |
| Chromebook compatibility surprise | Day 7 morning specifically tests one |
| Asset license confusion | Stick to Kenney CC0 only; no mixed sources for v1 |
| Latency to compile service from far regions | Cloudflare Containers (edge) or accept 200-500ms — acceptable for "Run" UX |
| Over-scoping | Strict daily deliverables; defer anything not on the list |

---

## 6. What This Sprint Does **Not** Solve

These are real and valuable, but explicitly punted to keep the sprint to a week:

- **Saved projects without an account** — URL-only sharing only; Phase 2 needs accounts
- **Multiplayer / collaborative editing** — out of scope
- **Touch input on tablets** — desktop/laptop only for v1
- **Offline mode** — requires WASM-compiled compiler, deferred to Phase 3
- **Curriculum for teachers** — examples only; lesson-plan packaging is a follow-up
- **Internationalization** — English-only for v1
- **Accessibility audit** — Monaco has reasonable defaults; full WCAG pass is a follow-up

---

## 7. Definition of "Approved to Start"

Chibueze signs off on:
1. The 7 in-scope items in §2
2. The 7 deferred items in §6 being deferred (not surprise-cut)
3. The hosting cost ceiling ($10/mo) and where the deploy lives
4. Domain to use (`play.gbasic.dev` or alternative)
5. Asset license policy (CC0-only for v1)

Once those are agreed, Day 1 starts.
