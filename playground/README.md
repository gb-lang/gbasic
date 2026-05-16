# G-Basic Playground

Browser-based playground where kids can write and run G-Basic programs without installing anything.

## Status

The playground has a Monaco editor, compile-service integration, a sandboxed
WASM runner, and a starter asset picker. It expects the compile service from
`services/compile/` to be running locally or reachable through
`window.__GBASIC_COMPILE_URL`.

## Local dev

Zero-build static site. Serve the folder and open it.

```sh
python3 -m http.server -d playground 8000
# or
npx serve playground
```

Visit http://localhost:8000 and click ▶ Run.

## GitHub Pages

The hosted demo is published from `playground/` by the
`Playground Pages` GitHub Actions workflow.

```text
push to main
    |
    v
validate playground files
    |
    v
upload playground/ as Pages artifact
    |
    v
deploy to https://gb-lang.github.io/gbasic/
```

GitHub Pages only serves static files. To make the Run button work on the
hosted demo, set the repository variable `GBASIC_COMPILE_URL` to a deployed
compile-service `/compile` endpoint. Optionally set `GBASIC_TELEMETRY_URL` to
override telemetry separately.

## Files

- `index.html` — page shell, loads Monaco from CDN
- `app.js` — editor init, Run/Stop/Share handlers, sandboxed WASM runner, asset picker
- `style.css` — layout and theme
- `assets/` — web-served copy of the starter asset pack

## Plan

See [`docs/kids-launch-timeline.md`](../docs/kids-launch-timeline.md) for the day-by-day plan.
