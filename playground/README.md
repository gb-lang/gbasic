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

## Files

- `index.html` — page shell, loads Monaco from CDN
- `app.js` — editor init, Run/Stop/Share handlers, sandboxed WASM runner, asset picker
- `style.css` — layout and theme
- `assets/` — web-served copy of the starter asset pack

## Plan

See [`docs/kids-launch-timeline.md`](../docs/kids-launch-timeline.md) for the day-by-day plan.
