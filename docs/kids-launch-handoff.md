# Kids Launch Handoff

Audience: Chibueze or whoever owns the G-Basic soft launch after the agent sprint.

## What Exists

- Compiler and web target return runnable WASM bundles.
- Playground can compile, run, stop, and share programs.
- Runtime runs each program in a fresh iframe sandbox.
- Starter assets are included and discoverable in the playground.
- Six lessons exist with starter and solution code.
- Compile service has size/time/rate guardrails and anonymous event counters.

## How to Add a Lesson

1. Add `playground/lessons/lesson-N.md`.
2. Add `lesson-N.starter.gb` and `lesson-N.solution.gb`.
3. Add an entry to `playground/lessons/manifest.json`.
4. Run the lesson fixture smoke check in CI.

## How to Add an Asset

1. Add web PNG to `playground/assets/sprites/` or WAV to `playground/assets/sounds/`.
2. Add desktop BMP/WAV mirror to `assets/`.
3. Update both `manifest.json` files.
4. Update both `CREDITS.md` files.

## Open Decisions

- Hosted domain for the playground
- Compile-service host: Fly.io, Cloudflare Containers, or another container host
- Budget alert owner for the compile service
- Who receives first 24h feedback
- Whether broad launch waits for real Kenney/professional art

## Known Follow-Ups

- True screenshot-based Open Graph preview
- Account-based saved projects
- Touch controls for tablets
- Full accessibility pass
- Teacher curriculum packaging
- In-browser compiler for offline mode
