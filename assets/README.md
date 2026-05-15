# Assets

Default CC0 starter assets for G-Basic programs.

The kids-launch playground and desktop runtime can resolve simple names:

```gbasic
let hero = sprite("hero")
play("jump")
```

The runtime first tries an explicit file path, then falls back to this bundled
starter pack:

- `sprites/*.png` for the web playground
- `sprites/*.bmp` for the desktop SDL runtime
- `sounds/*.wav` for web and desktop sound effects
- `manifest.json` for playground discovery and the asset picker

See `CREDITS.md` for the full asset list and license notes.
