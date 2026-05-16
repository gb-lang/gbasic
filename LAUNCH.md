# G-Basic Kids Launch Runbook

This runbook covers the `v0.3.0-kids` soft launch: playground, compile service,
starter assets, lessons, sharing, and the first-feedback loop.

## Release Scope

- Browser playground with Monaco editor, Run/Stop/Share, sandboxed WASM runner
- Compile-on-demand service with timeout, output cap, source cap, rate limiting
- CC0 starter asset pack
- Six in-playground lessons
- Share URLs
- Canonical game and lesson fixture smoke checks

## Pre-Launch Checklist

- [ ] PR #6 merged: Day 2 corrected runtime execution
- [ ] PR #8 merged: Day 3 assets
- [ ] PR #9 merged: Day 4 lessons
- [ ] PR #10 merged: Day 5 sharing and smoke checks
- [ ] PR #11 merged: Day 6 polish and service guardrails
- [ ] CI green on `main`
- [ ] `cargo test --workspace` green in CI
- [ ] Compile service deployed
- [ ] Playground static site deployed
- [ ] `window.__GBASIC_COMPILE_URL` points to the deployed compile service
- [ ] Domain chosen and DNS pointed
- [ ] First soft-launch testers selected

## Local Smoke Test

Terminal 1:

```sh
cargo build --release -p gbasic --features llvm
export GBASIC_BIN="$(pwd)/target/release/gbasic"
cargo run -p gbasic-compile-service
```

Terminal 2:

```sh
python3 -m http.server -d playground 8000
```

Open `http://localhost:8000`.

Checklist:

- [ ] Lesson 1 loads
- [ ] Load starter inserts code into editor
- [ ] Run compiles and starts the sandbox
- [ ] Share copies a URL
- [ ] Opening the share URL restores code
- [ ] Asset picker inserts `sprite("hero")`
- [ ] Asset picker inserts `play("jump")`
- [ ] Compile error appears in the inline error panel
- [ ] Stop tears down a running loop

## Browser QA

- [ ] Chrome desktop
- [ ] Safari desktop
- [ ] Firefox desktop
- [ ] Chromebook Chrome
- [ ] Narrow viewport shows desktop/laptop fallback without overlapping controls
- [ ] Share URL round-trips a 200-line program
- [ ] Share URL round-trips a non-ASCII title/source

## Canonical Game QA

On a machine with LLVM 18 and SDL2 display support:

```sh
cargo test -p gbasic --test canonical_games
./target/debug/gbasic examples/pong.gb -o /tmp/gbasic-pong --run
./target/debug/gbasic examples/flappy.gb -o /tmp/gbasic-flappy --run
./target/debug/gbasic examples/angrybirds.gb -o /tmp/gbasic-angrybirds --run
```

Manual pass criteria:

- [ ] Window opens
- [ ] Program does not panic
- [ ] Frame loop updates
- [ ] Input works where expected
- [ ] Text/score rendering is visible where expected

## Deploy

The compile service is container-ready:

```sh
docker build -t gbasic-compile-service -f services/compile/Dockerfile .
docker run -p 8080:8080 gbasic-compile-service
```

Static playground deployment only needs the `playground/` directory. Set:

```html
<script>
  window.__GBASIC_COMPILE_URL = "https://<compile-service>/compile";
  window.__GBASIC_TELEMETRY_URL = "https://<compile-service>/telemetry";
</script>
```

before `app.js` if the deployment URL differs from the default local service.

## Rollback

If the playground fails:

1. Revert the static site deployment to the previous build.
2. Keep the compile service running if `/healthz` is healthy.
3. If compile errors spike, scale compile service to zero or block `/compile`.
4. Post a short tester update with the known issue and expected retry window.

If the compile service fails:

1. Check `/healthz`.
2. Check container logs for `compile timed out`, `rate limit exceeded`, or `spawn`.
3. Verify `GBASIC_BIN` points to a release compiler binary.
4. Roll back the service container to the previous image.

## Hot-Fixing Lessons

Lesson files are static:

- `playground/lessons/lesson-N.md`
- `playground/lessons/lesson-N.starter.gb`
- `playground/lessons/lesson-N.solution.gb`
- `playground/lessons/manifest.json`

For copy-only fixes, update the static site and redeploy. For code fixtures, run
the lesson fixture smoke test in CI before deploying.

## Tagging

After the launch PR stack lands and the smoke test passes:

```sh
git checkout main
git pull
git tag v0.3.0-kids
git push origin v0.3.0-kids
```

## First Feedback Loop

Soft-launch to 5 friends-with-kids first. Ask only:

1. Could the child start Lesson 1 without help?
2. Did Run produce visible output?
3. Where did they get stuck?
4. Did sharing work?
5. What did they try to make after the lessons?

Hold broad announcement until the first 24 hours of feedback are reviewed.
