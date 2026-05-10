# G-Basic Playground

Browser-based playground where kids can write and run G-Basic programs without installing anything.

## Status

Day 1 scaffold (morning): Monaco editor + output canvas + Run/Stop/Share buttons + a stubbed "compile" path that naively renders `print("…")` calls so the plumbing is verifiable.

The real compile path lands in Day 1 afternoon (`services/compile/`). The real WASM runtime lands in Day 2.

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
- `app.js` — editor init, button handlers, stub run path
- `style.css` — layout and theme

## Plan

See [`docs/kids-launch-timeline.md`](../docs/kids-launch-timeline.md) for the day-by-day plan.
