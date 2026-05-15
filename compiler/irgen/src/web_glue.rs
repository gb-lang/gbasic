//! JS glue code generator for WASM web target.
//!
//! Generates `runtime.js` (providing all `runtime_*` functions as WASM imports)
//! and `index.html` for running G-Basic programs in the browser.

use gbasic_common::error::GBasicError;
use std::fs;
use std::path::Path;

/// Generate index.html + runtime.js in the given output directory.
pub fn generate_web_output(output_dir: &str) -> Result<(), GBasicError> {
    let dir = Path::new(output_dir);
    fs::write(dir.join("runtime.js"), RUNTIME_JS).map_err(|e| GBasicError::CodegenError {
        span: None,
        message: format!("failed to write runtime.js: {e}"),
    })?;
    fs::write(dir.join("index.html"), INDEX_HTML).map_err(|e| GBasicError::CodegenError {
        span: None,
        message: format!("failed to write index.html: {e}"),
    })?;
    Ok(())
}

/// Returns the list of all runtime function names provided by the JS glue.
/// Used by parity tests to ensure web and desktop stay in sync.
pub fn js_runtime_function_names() -> Vec<&'static str> {
    vec![
        // Print / IO
        "runtime_print",
        "runtime_print_int",
        "runtime_print_float",
        "runtime_print_str_part",
        "runtime_print_int_part",
        "runtime_print_float_part",
        "runtime_print_newline",
        "runtime_string_concat",
        "runtime_string_eq",
        "runtime_int_to_str",
        "runtime_float_to_str",
        // Screen
        "runtime_screen_init",
        "runtime_screen_clear",
        "runtime_screen_set_pixel",
        "runtime_screen_draw_rect",
        "runtime_screen_draw_line",
        "runtime_screen_draw_circle",
        "runtime_screen_present",
        "runtime_screen_width",
        "runtime_screen_height",
        "runtime_screen_center_x",
        "runtime_screen_center_y",
        "runtime_screen_sprite_load",
        "runtime_screen_sprite_at",
        "runtime_screen_sprite_scale",
        "runtime_screen_sprite_draw",
        "ensure_screen_init",
        // Input
        "runtime_input_key_pressed",
        "runtime_input_mouse_x",
        "runtime_input_mouse_y",
        "runtime_input_mouse_clicked",
        "runtime_input_poll",
        // Math
        "runtime_math_sin",
        "runtime_math_cos",
        "runtime_math_sqrt",
        "runtime_math_abs",
        "runtime_math_floor",
        "runtime_math_ceil",
        "runtime_math_pow",
        "runtime_math_max",
        "runtime_math_min",
        "runtime_math_random",
        "runtime_math_pi",
        "runtime_math_random_range",
        "runtime_math_clamp",
        // System
        "runtime_system_time",
        "runtime_system_sleep",
        "runtime_system_exit",
        "runtime_system_wait",
        "runtime_system_log",
        "runtime_system_frame_begin",
        "runtime_system_frame_end",
        "runtime_system_frame_time",
        "runtime_frame_auto",
        "runtime_frame_auto_end",
        // Sound
        "runtime_sound_beep",
        "runtime_sound_effect_load",
        "runtime_sound_effect_play",
        "runtime_sound_effect_volume",
        // Memory
        "runtime_memory_set",
        "runtime_memory_get",
        // IO
        "runtime_io_print",
        "runtime_io_read_file",
        "runtime_io_write_file",
        // Asset
        "runtime_asset_load",
        // Objects
        "runtime_create_rect",
        "runtime_create_circle",
        "runtime_set_position",
        "runtime_set_position_x",
        "runtime_set_position_y",
        "runtime_set_color",
        "runtime_set_velocity",
        "runtime_set_velocity_x",
        "runtime_set_velocity_y",
        "runtime_set_gravity",
        "runtime_set_solid",
        "runtime_set_bounces",
        "runtime_set_visible",
        "runtime_set_layer",
        "runtime_get_position_x",
        "runtime_get_position_y",
        "runtime_get_velocity_x",
        "runtime_get_velocity_y",
        "runtime_get_size_width",
        "runtime_get_size_height",
        "runtime_object_move",
        "runtime_object_collides",
        "runtime_object_contains",
        "runtime_object_remove",
        // Arrays
        "runtime_array_new",
        "runtime_array_add",
        "runtime_array_get",
        "runtime_array_length",
        "runtime_array_remove_value",
        // Text rendering
        "runtime_draw_text",
    ]
}

const INDEX_HTML: &str = r##"<!DOCTYPE html>
<html>
<head>
<meta charset="utf-8">
<title>G-Basic</title>
<style>
  body { margin: 0; background: #111; display: flex; flex-direction: column; align-items: center; justify-content: center; height: 100vh; }
  canvas { border: 1px solid #333; image-rendering: pixelated; }
  #output { color: #0f0; font-family: monospace; padding: 8px; max-width: 800px; white-space: pre-wrap; }
</style>
</head>
<body>
<canvas id="canvas" width="800" height="600"></canvas>
<div id="output"></div>
<script>window.onerror = function(msg, src, line, col, err) { document.getElementById("output").textContent += "ERROR: " + msg + " at " + src + ":" + line + "\n"; };</script>
<script src="runtime.js"></script>
<script>
  loadAndRun("game.wasm").catch(e => { document.getElementById("output").textContent += "LOAD ERROR: " + e.message + "\n"; console.error(e); });
</script>
</body>
</html>
"##;

const RUNTIME_JS: &str = r##"
// G-Basic Web Runtime — provides all runtime_* functions as WASM imports
// Uses Asyncify for non-blocking game loops: runtime_frame_auto_end yields
// to the browser via requestAnimationFrame, then resumes WASM execution.

const state = {
  canvas: null,
  ctx: null,
  width: 800,
  height: 600,
  initialized: false,
  keys: new Set(),
  mouseX: 0,
  mouseY: 0,
  mouseClicked: false,
  memory: null,
  memoryMap: new Map(),
  objects: [],
  arrays: [],
  frameStart: 0,
  frameDt: 16.67,
  printBuffer: "",
  outputDiv: null,
  wasmExports: null,
  // Sprite cache: array of { image, x, y, scale, ready, error }
  sprites: [],
  // Audio (lazy-init on first sound call; needs user gesture to resume)
  audioContext: null,
  soundCache: null,    // Map<name, AudioBuffer | null>
  soundLoading: null,  // Map<name, Promise>
  soundVolumes: null,  // Map<name, number>
  // Asset path roots (overridable by host page)
  assetSpriteRoot: "assets/sprites/",
  assetSoundRoot: "assets/sounds/",
  // Asyncify state — buffer dynamically allocated after WASM loads
  asyncify: {
    dataAddr: 0,
    dataStart: 0,
    dataEnd: 0,
    sleeping: false,
  },
};

// ---- Asyncify helpers ----
// Asyncify works by instrumenting the WASM binary so that:
// 1. When we want to "sleep", we call asyncify_start_unwind(dataAddr)
//    which causes all WASM functions on the call stack to save state and return.
// 2. To resume, we call asyncify_start_rewind(dataAddr) then re-invoke _start().
//    The instrumented code detects rewind mode and fast-forwards to the sleep point.

function asyncifyStartUnwind() {
  // Write the stack bounds into the data area header
  const view = new Int32Array(state.memory.buffer, state.asyncify.dataAddr, 2);
  view[0] = state.asyncify.dataStart;
  view[1] = state.asyncify.dataEnd;
  state.wasmExports.asyncify_start_unwind(state.asyncify.dataAddr);
}

function asyncifyStopUnwind() {
  state.wasmExports.asyncify_stop_unwind();
}

function asyncifyStartRewind() {
  // Reset the stack pointer for rewind
  const view = new Int32Array(state.memory.buffer, state.asyncify.dataAddr, 2);
  // Don't reset view[0] — it was set by the unwind and rewind reads it back
  state.wasmExports.asyncify_start_rewind(state.asyncify.dataAddr);
}

function asyncifyStopRewind() {
  state.wasmExports.asyncify_stop_rewind();
}

// ---- String helpers ----
function readCStr(ptr) {
  const bytes = new Uint8Array(state.memory.buffer);
  let end = ptr;
  while (bytes[end] !== 0) end++;
  return new TextDecoder().decode(bytes.slice(ptr, end));
}

function writeCStr(str) {
  const encoded = new TextEncoder().encode(str);
  const ptr = state.wasmExports.wasm_alloc(encoded.length + 1);
  const bytes = new Uint8Array(state.memory.buffer);
  bytes.set(encoded, ptr);
  bytes[ptr + encoded.length] = 0;
  return ptr;
}

function appendOutput(text) {
  if (!state.outputDiv) state.outputDiv = document.getElementById("output");
  if (state.outputDiv) state.outputDiv.textContent += text;
  console.log(text);
}

// ---- Screen ----
function ensureCanvas() {
  if (state.initialized) return;
  state.canvas = document.getElementById("canvas");
  if (state.canvas) {
    state.canvas.width = state.width;
    state.canvas.height = state.height;
    state.ctx = state.canvas.getContext("2d");
  }
  state.initialized = true;

  document.addEventListener("keydown", e => {
    const k = e.key.toLowerCase();
    state.keys.add(k);
    // Map arrow keys to short names used by G-Basic
    if (k === "arrowleft") state.keys.add("left");
    if (k === "arrowright") state.keys.add("right");
    if (k === "arrowup") state.keys.add("up");
    if (k === "arrowdown") state.keys.add("down");
    e.preventDefault();
  });
  document.addEventListener("keyup", e => {
    const k = e.key.toLowerCase();
    state.keys.delete(k);
    if (k === "arrowleft") state.keys.delete("left");
    if (k === "arrowright") state.keys.delete("right");
    if (k === "arrowup") state.keys.delete("up");
    if (k === "arrowdown") state.keys.delete("down");
  });
  if (state.canvas) {
    state.canvas.addEventListener("mousemove", e => {
      const r = state.canvas.getBoundingClientRect();
      state.mouseX = e.clientX - r.left;
      state.mouseY = e.clientY - r.top;
    });
    state.canvas.addEventListener("mousedown", () => { state.mouseClicked = true; });
    state.canvas.addEventListener("mouseup", () => { state.mouseClicked = false; });
  }
}

function colorStr(r, g, b) { return `rgb(${r},${g},${b})`; }

function configureGBasicRuntime(options = {}) {
  if (typeof options.assetSpriteRoot === "string") state.assetSpriteRoot = options.assetSpriteRoot;
  if (typeof options.assetSoundRoot === "string") state.assetSoundRoot = options.assetSoundRoot;
}

function resetRuntimeState() {
  state.canvas = null;
  state.ctx = null;
  state.width = 800;
  state.height = 600;
  state.initialized = false;
  state.keys.clear();
  state.mouseX = 0;
  state.mouseY = 0;
  state.mouseClicked = false;
  state.memory = null;
  state.memoryMap.clear();
  state.objects = [];
  state.arrays = [];
  state.frameStart = 0;
  state.frameDt = 16.67;
  state.printBuffer = "";
  state.outputDiv = null;
  state.wasmExports = null;
  state.sprites = [];
  state.asyncify = {
    dataAddr: 0,
    dataStart: 0,
    dataEnd: 0,
    sleeping: false,
  };
}

// ---- Sprite helpers ----
// Asynchronous load: returns a handle synchronously, image streams in.
// Drawing before the image finishes is a silent no-op (first frame might
// skip; subsequent frames render normally).
function loadSpriteByName(name) {
  const root = state.assetSpriteRoot;
  // Try common extensions in order; pick whichever the host actually serves.
  const candidates = [`${root}${name}.png`, `${root}${name}.jpg`, `${root}${name}.jpeg`, `${root}${name}`];
  const sprite = { image: null, x: 0, y: 0, scale: 1, ready: false, error: false, name };
  state.sprites.push(sprite);
  let attempt = 0;
  const tryNext = () => {
    if (attempt >= candidates.length) {
      sprite.error = true;
      console.warn(`sprite "${name}": no candidate URL resolved`);
      return;
    }
    const img = new Image();
    img.decoding = "async";
    img.onload = () => { sprite.image = img; sprite.ready = true; };
    img.onerror = () => { attempt++; tryNext(); };
    img.src = candidates[attempt];
  };
  tryNext();
  return state.sprites.length - 1;
}

// ---- Audio helpers ----
function ensureAudio() {
  if (!state.audioContext) {
    try {
      state.audioContext = new (window.AudioContext || window.webkitAudioContext)();
      state.soundCache = new Map();
      state.soundLoading = new Map();
      state.soundVolumes = new Map();
    } catch (e) {
      console.warn("Web Audio unavailable:", e.message);
      return false;
    }
  }
  // AudioContext starts suspended in modern browsers — only resumes after a
  // user gesture. The caller (the program's Run click) will already have
  // triggered one, but we resume defensively in case audio is the first
  // thing the program does.
  if (state.audioContext.state === "suspended") {
    state.audioContext.resume().catch(() => {});
  }
  return true;
}

function loadSoundByName(name) {
  if (state.soundCache.has(name)) return Promise.resolve(state.soundCache.get(name));
  if (state.soundLoading.has(name)) return state.soundLoading.get(name);
  const root = state.assetSoundRoot;
  const candidates = [`${root}${name}.wav`, `${root}${name}.mp3`, `${root}${name}.ogg`, `${root}${name}`];
  const promise = (async () => {
    for (const url of candidates) {
      try {
        const res = await fetch(url);
        if (!res.ok) continue;
        const arr = await res.arrayBuffer();
        const buf = await state.audioContext.decodeAudioData(arr);
        state.soundCache.set(name, buf);
        return buf;
      } catch (_) { /* try next */ }
    }
    console.warn(`sound "${name}": no candidate URL resolved`);
    state.soundCache.set(name, null);
    return null;
  })();
  state.soundLoading.set(name, promise);
  promise.finally(() => state.soundLoading.delete(name));
  return promise;
}

function playSoundByName(name) {
  if (!ensureAudio()) return;
  loadSoundByName(name).then((buf) => {
    if (!buf) return;
    const src = state.audioContext.createBufferSource();
    src.buffer = buf;
    const gain = state.audioContext.createGain();
    const vol = state.soundVolumes.get(name);
    gain.gain.value = vol == null ? 1.0 : vol;
    src.connect(gain).connect(state.audioContext.destination);
    src.start();
  });
}

// WASM i64 <-> JS BigInt conversion helpers
function N(v) { return typeof v === "bigint" ? Number(v) : v; }  // BigInt -> Number
function I(v) { return BigInt(v | 0); }  // Number -> i64 BigInt
function B(v) { return v ? 1n : 0n; }   // bool -> i64 BigInt

// ---- Object model ----
function createObject(type, props) {
  const obj = {
    type,
    x: 0, y: 0,
    vx: 0, vy: 0,
    w: N(props.w || 0), h: N(props.h || 0), r: N(props.r || 0),
    cr: 255, cg: 255, cb: 255,
    gravity: 0,
    solid: false,
    bounces: false,
    visible: true,
    layer: 0,
    removed: false,
  };
  state.objects.push(obj);
  return state.objects.length - 1;
}

function drawObjects() {
  const sorted = state.objects
    .map((o, i) => ({ ...o, idx: i }))
    .filter(o => o.visible && !o.removed)
    .sort((a, b) => a.layer - b.layer);
  const ctx = state.ctx;
  if (!ctx) return;
  for (const o of sorted) {
    ctx.fillStyle = colorStr(o.cr, o.cg, o.cb);
    if (o.type === "rect") {
      ctx.fillRect(o.x, o.y, o.w, o.h);
    } else if (o.type === "circle") {
      ctx.beginPath();
      ctx.arc(o.x, o.y, o.r, 0, Math.PI * 2);
      ctx.fill();
    }
  }
}

function objBounds(o) {
  // Match desktop: rects are top-left based, circles are center-based
  if (o.type === "rect") return [o.x, o.y, o.x + o.w, o.y + o.h];
  return [o.x - o.r, o.y - o.r, o.x + o.r, o.y + o.r];
}

function updatePhysics() {
  const objs = state.objects;
  for (const obj of objs) {
    if (obj.removed || !obj.visible) continue;
    obj.vy += obj.gravity;
    obj.x += obj.vx;
    obj.y += obj.vy;
    // Bounce off screen edges
    if (obj.bounces) {
      const [x1, y1, x2, y2] = objBounds(obj);
      if (x1 <= 0 || x2 >= state.width) {
        obj.vx = -obj.vx;
        if (x1 <= 0) obj.x -= x1;
        if (x2 >= state.width) obj.x -= x2 - state.width;
      }
      if (y1 <= 0 || y2 >= state.height) {
        obj.vy = -obj.vy;
        if (y1 <= 0) obj.y -= y1;
        if (y2 >= state.height) obj.y -= y2 - state.height;
      }
    }
  }
  // Bounce off solid objects
  for (let i = 0; i < objs.length; i++) {
    const a = objs[i];
    if (a.removed || !a.bounces) continue;
    for (let j = 0; j < objs.length; j++) {
      if (i === j) continue;
      const b = objs[j];
      if (b.removed || !b.solid) continue;
      const [ax1, ay1, ax2, ay2] = objBounds(a);
      const [bx1, by1, bx2, by2] = objBounds(b);
      if (ax1 < bx2 && ax2 > bx1 && ay1 < by2 && ay2 > by1) {
        const overlapX = Math.min(ax2, bx2) - Math.max(ax1, bx1);
        const overlapY = Math.min(ay2, by2) - Math.max(ay1, by1);
        if (overlapX < overlapY) {
          a.vx = -a.vx;
          a.x += (a.x < b.x) ? -overlapX : overlapX;
        } else {
          a.vy = -a.vy;
          a.y += (a.y < b.y) ? -overlapY : overlapY;
        }
      }
    }
  }
}

// ---- WASM import object ----
// All i64 params arrive as BigInt; all i64 returns must be BigInt.
// N() converts BigInt->Number for JS math; I() converts Number->BigInt for i64 returns.
function buildImports(memory) {
  state.memory = memory;
  return {
    env: {
      // Print (ptr=i32, v=i64 or f64)
      runtime_print(ptr) { appendOutput(readCStr(ptr) + "\n"); },
      runtime_print_int(v) { appendOutput(N(v) + "\n"); },
      runtime_print_float(v) { appendOutput(v + "\n"); },
      runtime_print_str_part(ptr) { state.printBuffer += readCStr(ptr); },
      runtime_print_int_part(v) { state.printBuffer += N(v).toString(); },
      runtime_print_float_part(v) { state.printBuffer += v.toString(); },
      runtime_print_newline() { appendOutput(state.printBuffer + "\n"); state.printBuffer = ""; },
      runtime_string_concat(a, b) { return writeCStr(readCStr(a) + readCStr(b)); },
      runtime_string_eq(a, b) { return B(readCStr(a) === readCStr(b)); },
      runtime_int_to_str(v) { return writeCStr(N(v).toString()); },
      runtime_float_to_str(v) { return writeCStr(v.toString()); },

      // Screen (i64 params for dimensions/colors)
      runtime_screen_init(w, h) { state.width = N(w); state.height = N(h); ensureCanvas(); },
      runtime_screen_clear(r, g, b) {
        ensureCanvas();
        if (state.ctx) { state.ctx.fillStyle = colorStr(N(r), N(g), N(b)); state.ctx.fillRect(0, 0, state.width, state.height); }
      },
      runtime_screen_set_pixel(x, y, r, g, b) {
        if (state.ctx) { state.ctx.fillStyle = colorStr(N(r), N(g), N(b)); state.ctx.fillRect(N(x), N(y), 1, 1); }
      },
      runtime_screen_draw_rect(x, y, w, h, r, g, b) {
        if (state.ctx) { state.ctx.fillStyle = colorStr(N(r), N(g), N(b)); state.ctx.fillRect(N(x), N(y), N(w), N(h)); }
      },
      runtime_screen_draw_line(x1, y1, x2, y2, r, g, b) {
        if (state.ctx) { state.ctx.strokeStyle = colorStr(N(r), N(g), N(b)); state.ctx.beginPath(); state.ctx.moveTo(N(x1), N(y1)); state.ctx.lineTo(N(x2), N(y2)); state.ctx.stroke(); }
      },
      runtime_screen_draw_circle(x, y, rad, r, g, b) {
        if (state.ctx) { state.ctx.fillStyle = colorStr(N(r), N(g), N(b)); state.ctx.beginPath(); state.ctx.arc(N(x), N(y), N(rad), 0, Math.PI * 2); state.ctx.fill(); }
      },
      runtime_screen_present() { },
      runtime_screen_width() { ensureCanvas(); return I(state.width); },
      runtime_screen_height() { ensureCanvas(); return I(state.height); },
      runtime_screen_center_x() { ensureCanvas(); return state.width / 2.0; },
      runtime_screen_center_y() { ensureCanvas(); return state.height / 2.0; },
      runtime_screen_sprite_load(ptr) {
        ensureCanvas();
        return I(loadSpriteByName(readCStr(ptr)));
      },
      runtime_screen_sprite_at(id, x, y) {
        const s = state.sprites[N(id)];
        if (s) { s.x = N(x); s.y = N(y); }
        return I(N(id));
      },
      runtime_screen_sprite_scale(id, scale) {
        const s = state.sprites[N(id)];
        if (s) s.scale = Math.max(0, scale);
        return I(N(id));
      },
      runtime_screen_sprite_draw(id) {
        const s = state.sprites[N(id)];
        if (!s || !s.ready || !state.ctx) return;
        const w = s.image.width * s.scale;
        const h = s.image.height * s.scale;
        state.ctx.drawImage(s.image, s.x, s.y, w, h);
      },
      ensure_screen_init() { ensureCanvas(); },

      // Input (returns i64)
      runtime_input_key_pressed(ptr) { return B(state.keys.has(readCStr(ptr))); },
      runtime_input_mouse_x() { return I(state.mouseX); },
      runtime_input_mouse_y() { return I(state.mouseY); },
      runtime_input_mouse_clicked() { return B(state.mouseClicked); },
      runtime_input_poll() { },

      // Math (f64 in/out, no BigInt needed)
      runtime_math_sin(v) { return Math.sin(v); },
      runtime_math_cos(v) { return Math.cos(v); },
      runtime_math_sqrt(v) { return Math.sqrt(v); },
      runtime_math_abs(v) { return Math.abs(v); },
      runtime_math_floor(v) { return Math.floor(v); },
      runtime_math_ceil(v) { return Math.ceil(v); },
      runtime_math_pow(a, b) { return Math.pow(a, b); },
      runtime_math_max(a, b) { return Math.max(a, b); },
      runtime_math_min(a, b) { return Math.min(a, b); },
      runtime_math_random() { return Math.random(); },
      runtime_math_pi() { return Math.PI; },
      runtime_math_random_range(min, max) { const lo = N(min), hi = N(max); return I(lo + Math.floor(Math.random() * (hi - lo + 1))); },
      runtime_math_clamp(v, lo, hi) { return Math.max(lo, Math.min(hi, v)); },

      // System
      runtime_system_time() { return performance.now() / 1000.0; },
      runtime_system_sleep(_ms) { },
      runtime_system_wait(secs) { },
      runtime_system_log(ptr) { console.log("[log]", readCStr(ptr)); },
      runtime_system_exit(_code) { },
      runtime_system_frame_begin() { state.frameStart = performance.now(); },
      runtime_system_frame_end() { state.frameDt = performance.now() - state.frameStart; },
      runtime_system_frame_time() { return state.frameDt / 1000.0; },
      runtime_frame_auto() {
        if (state.wasmExports && state.wasmExports.asyncify_get_state) {
          const s = state.wasmExports.asyncify_get_state();
          if (s !== 0) return;
        }
        ensureCanvas();
        state.frameStart = performance.now();
      },
      runtime_frame_auto_end() {
        const s = state.wasmExports.asyncify_get_state();
        if (s === 2) {
          state.wasmExports.asyncify_stop_rewind();
          return;
        }
        if (s !== 0) return;
        const dt = performance.now() - state.frameStart;
        state.frameDt = dt;
        updatePhysics();
        drawObjects();
        state.asyncify.sleeping = true;
        asyncifyStartUnwind();
      },

      // Sound (i64 params)
      runtime_sound_beep(freq, dur) {
        try {
          const actx = new (window.AudioContext || window.webkitAudioContext)();
          const osc = actx.createOscillator();
          osc.frequency.value = N(freq);
          osc.connect(actx.destination);
          osc.start();
          osc.stop(actx.currentTime + N(dur) / 1000);
        } catch(e) {}
      },
      runtime_sound_effect_load(ptr) {
        // Fire-and-forget pre-warm of the cache. Returns 0 because the
        // ABI does not use the handle elsewhere — play/volume key by name.
        if (ensureAudio()) loadSoundByName(readCStr(ptr));
        return I(0);
      },
      runtime_sound_effect_play(ptr) {
        playSoundByName(readCStr(ptr));
      },
      runtime_sound_effect_volume(ptr, vol) {
        if (!ensureAudio()) return;
        state.soundVolumes.set(readCStr(ptr), vol);
      },

      // Memory (i64 val)
      runtime_memory_set(ptr, val) { state.memoryMap.set(readCStr(ptr), N(val)); },
      runtime_memory_get(ptr) { return I(state.memoryMap.get(readCStr(ptr)) || 0); },

      // IO
      runtime_io_print(ptr) { appendOutput(readCStr(ptr) + "\n"); },
      runtime_io_read_file(ptr) { return writeCStr(""); },
      runtime_io_write_file(pathPtr, dataPtr) { },

      // Asset
      runtime_asset_load(ptr) { return I(0); },

      // Objects (handle=i64, positions=f64, colors=i64)
      runtime_create_rect(w, h) { return I(createObject("rect", { w, h })); },
      runtime_create_circle(r) { return I(createObject("circle", { r })); },
      runtime_set_position(handle, x, y) { const o = state.objects[N(handle)]; if (o) { o.x = x; o.y = y; } },
      runtime_set_position_x(handle, x) { const o = state.objects[N(handle)]; if (o) o.x = x; },
      runtime_set_position_y(handle, y) { const o = state.objects[N(handle)]; if (o) o.y = y; },
      runtime_set_color(handle, r, g, b) { const o = state.objects[N(handle)]; if (o) { o.cr = N(r); o.cg = N(g); o.cb = N(b); } },
      runtime_set_velocity(handle, vx, vy) { const o = state.objects[N(handle)]; if (o) { o.vx = vx; o.vy = vy; } },
      runtime_set_velocity_x(handle, vx) { const o = state.objects[N(handle)]; if (o) o.vx = vx; },
      runtime_set_velocity_y(handle, vy) { const o = state.objects[N(handle)]; if (o) o.vy = vy; },
      runtime_set_gravity(handle, g) { const o = state.objects[N(handle)]; if (o) o.gravity = g; },
      runtime_set_solid(handle, v) { const o = state.objects[N(handle)]; if (o) o.solid = N(v) !== 0; },
      runtime_set_bounces(handle, v) { const o = state.objects[N(handle)]; if (o) o.bounces = N(v) !== 0; },
      runtime_set_visible(handle, v) { const o = state.objects[N(handle)]; if (o) o.visible = N(v) !== 0; },
      runtime_set_layer(handle, l) { const o = state.objects[N(handle)]; if (o) o.layer = N(l); },
      runtime_get_position_x(handle) { return state.objects[N(handle)]?.x || 0.0; },
      runtime_get_position_y(handle) { return state.objects[N(handle)]?.y || 0.0; },
      runtime_get_velocity_x(handle) { return state.objects[N(handle)]?.vx || 0.0; },
      runtime_get_velocity_y(handle) { return state.objects[N(handle)]?.vy || 0.0; },
      runtime_get_size_width(handle) { const o = state.objects[N(handle)]; return o ? (o.w || o.r * 2) : 0.0; },
      runtime_get_size_height(handle) { const o = state.objects[N(handle)]; return o ? (o.h || o.r * 2) : 0.0; },
      runtime_object_move(handle, dx, dy) { const o = state.objects[N(handle)]; if (o) { o.x += dx; o.y += dy; } },
      runtime_object_collides(h1, h2) {
        const a = state.objects[N(h1)], b = state.objects[N(h2)];
        if (!a || !b) return I(0);
        const [ax1,ay1,ax2,ay2] = objBounds(a);
        const [bx1,by1,bx2,by2] = objBounds(b);
        return B(ax1 < bx2 && ax2 > bx1 && ay1 < by2 && ay2 > by1);
      },
      runtime_object_contains(handle, px, py) {
        const o = state.objects[N(handle)];
        if (!o) return I(0);
        if (o.type === "rect") {
          return B(px >= o.x && px <= o.x + o.w && py >= o.y && py <= o.y + o.h);
        } else {
          const dx = px - o.x, dy = py - o.y;
          return B(dx*dx + dy*dy <= o.r*o.r);
        }
      },
      runtime_object_remove(handle) { const o = state.objects[N(handle)]; if (o) o.removed = true; },

      // Arrays (handle=i64, val=i64)
      runtime_array_new() { state.arrays.push([]); return I(state.arrays.length - 1); },
      runtime_array_add(handle, val) { const a = state.arrays[N(handle)]; if (a) a.push(N(val)); },
      runtime_array_get(handle, idx) { return I(state.arrays[N(handle)]?.[N(idx)] || 0); },
      runtime_array_length(handle) { return I(state.arrays[N(handle)]?.length || 0); },
      runtime_array_remove_value(handle, val) {
        const arr = state.arrays[N(handle)];
        if (arr) { const i = arr.indexOf(N(val)); if (i >= 0) arr.splice(i, 1); }
      },

      // Text rendering (ptr=i32, coords=i64, colors=i64)
      runtime_draw_text(ptr, x, y, r, g, b) {
        if (state.ctx) {
          state.ctx.fillStyle = colorStr(N(r), N(g), N(b));
          state.ctx.font = "16px monospace";
          state.ctx.fillText(readCStr(ptr), N(x), N(y));
        }
      },
    },
  };
}

async function loadAndRunBytes(bytes) {
  resetRuntimeState();
  const memory = new WebAssembly.Memory({ initial: 32, maximum: 256 });
  const imports = buildImports(memory);
  imports.env.memory = memory;

  const { instance } = await WebAssembly.instantiate(bytes, imports);

  // Use WASM's own memory if exported
  if (instance.exports.memory) {
    state.memory = instance.exports.memory;
  }
  state.wasmExports = instance.exports;

  // Check if Asyncify is available (wasm-opt --asyncify was applied)
  const hasAsyncify = typeof instance.exports.asyncify_start_unwind === "function";

  if (!hasAsyncify) {
    console.warn("Asyncify not available — game loops will block the browser");
    if (instance.exports._start) instance.exports._start();
    return;
  }

  // Allocate asyncify data buffer using wasm_alloc (safe, above heap)
  const ASYNCIFY_BUF_SIZE = 16384; // 16KB
  const dataAddr = instance.exports.wasm_alloc(8 + ASYNCIFY_BUF_SIZE);
  state.asyncify.dataAddr = dataAddr;
  state.asyncify.dataStart = dataAddr + 8;
  state.asyncify.dataEnd = dataAddr + 8 + ASYNCIFY_BUF_SIZE;
  console.log("Asyncify buffer at", dataAddr, "to", state.asyncify.dataEnd);

  try {
    // Run _start(). It will return either:
    // a) Normally (program finished, no game loop)
    // b) After unwinding (game loop yielded at runtime_frame_auto_end)
    console.log("[gbasic] Running _start()...");
    instance.exports._start();
    console.log("[gbasic] _start() returned, sleeping:", state.asyncify.sleeping);

    if (state.asyncify.sleeping) {
      asyncifyStopUnwind();
      console.log("[gbasic] Unwind complete, starting frame loop");

      let frameCount = 0;
      function onFrame() {
        try {
          if (!state.asyncify.sleeping) return;
          state.asyncify.sleeping = false;
          asyncifyStartRewind();
          instance.exports._start();

          if (state.asyncify.sleeping) {
            asyncifyStopUnwind();
            frameCount++;
            if (frameCount % 60 === 0) console.log("[gbasic] Frame", frameCount);
            requestAnimationFrame(onFrame);
          } else {
            console.log("[gbasic] Program finished after", frameCount, "frames");
          }
        } catch(e) {
          console.error("[gbasic] Frame error:", e);
          appendOutput("ERROR: " + e.message + "\n");
        }
      }
      requestAnimationFrame(onFrame);
    }
  } catch(e) {
    console.error("[gbasic] Error:", e);
    appendOutput("ERROR: " + e.message + "\n");
  }
}

async function loadAndRun(wasmUrl) {
  const response = await fetch(wasmUrl);
  const bytes = await response.arrayBuffer();
  return loadAndRunBytes(bytes);
}
"##;
