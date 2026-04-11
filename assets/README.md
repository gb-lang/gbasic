# Assets

Default placeholder assets for G-Basic programs.

| File | Description |
|------|-------------|
| `beep.wav` | Minimal valid WAV file (short silence). Used as a default/fallback sound effect. |
| `default_sprite.bmp` | 8x8 white BMP image. Used as a default/fallback sprite. |

These files are intentionally minimal. They exist so that G-Basic example programs
that reference `play("beep")` or load a default sprite have something valid to work
with out of the box, without requiring users to supply their own assets first.

Replace them with your own assets as needed.
