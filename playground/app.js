// G-Basic Playground.
//
// Sends source to the compile service (services/compile/) which returns
// a wasm bundle + runtime.js. The playground instantiates that locally
// in a per-run iframe sandbox and runs it against the canvas.

const COMPILE_ENDPOINT =
  (typeof window !== "undefined" && window.__GBASIC_COMPILE_URL) ||
  "http://localhost:8080/compile";
const ASSET_MANIFEST_URL = "assets/manifest.json";

const SAMPLE_PROGRAM = `// Welcome to G-Basic!
// Click ▶ Run to see your program in action.

print("Hello, world!")
`;

const KEYWORDS = [
  "let", "fun", "fn", "if", "else", "while", "for", "in", "to",
  "return", "break", "continue", "match", "true", "false",
  "and", "or", "not"
];

let editor = null;
let running = false;
let runnerFrame = null;

const runBtn = document.getElementById("run-btn");
const stopBtn = document.getElementById("stop-btn");
const shareBtn = document.getElementById("share-btn");
const canvas = document.getElementById("canvas");
const consoleEl = document.getElementById("console");
const runnerHost = document.getElementById("runner-host");
const spriteAssetsEl = document.getElementById("sprite-assets");
const soundAssetsEl = document.getElementById("sound-assets");

require.config({
  paths: { vs: "https://cdn.jsdelivr.net/npm/monaco-editor@0.45.0/min/vs" }
});

require(["vs/editor/editor.main"], () => {
  monaco.languages.register({ id: "gbasic" });
  monaco.languages.setMonarchTokensProvider("gbasic", {
    keywords: KEYWORDS,
    tokenizer: {
      root: [
        [/\/\/.*$/, "comment"],
        [/\/\*/, "comment", "@comment"],
        [/"([^"\\]|\\.)*"/, "string"],
        [/\b\d+(\.\d+)?\b/, "number"],
        [/\b(let|fun|fn|if|else|while|for|in|to|return|break|continue|match|true|false|and|or|not)\b/, "keyword"],
        [/[A-Z][A-Za-z0-9_]*/, "type"],
        [/[a-z_][a-zA-Z0-9_]*/, "identifier"]
      ],
      comment: [
        [/[^\/*]+/, "comment"],
        [/\*\//, "comment", "@pop"],
        [/[\/*]/, "comment"]
      ]
    }
  });

  editor = monaco.editor.create(document.getElementById("editor"), {
    value: SAMPLE_PROGRAM,
    language: "gbasic",
    theme: "vs-dark",
    fontSize: 16,
    minimap: { enabled: false },
    automaticLayout: true,
    scrollBeyondLastLine: false
  });
});

runBtn.addEventListener("click", runProgram);
stopBtn.addEventListener("click", stopProgram);
shareBtn.addEventListener("click", shareProgram);
loadAssetPanel();

// Capture keyboard for the canvas when the program is running.
canvas.addEventListener("click", () => canvas.focus());

async function runProgram() {
  if (running || !editor) return;
  running = true;
  runBtn.disabled = true;
  stopBtn.disabled = false;
  consoleEl.textContent = "";

  const ctx = canvas.getContext("2d");
  ctx.fillStyle = "#1e1e2e";
  ctx.fillRect(0, 0, canvas.width, canvas.height);

  const source = editor.getValue();
  log("Compiling…");

  let result;
  try {
    const res = await fetch(COMPILE_ENDPOINT, {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify({ source }),
    });
    result = await res.json();
    if (!res.ok && !result?.errors) {
      result = { errors: `compile service: HTTP ${res.status}` };
    }
  } catch (e) {
    result = {
      errors:
        `compile service unreachable at ${COMPILE_ENDPOINT}\n` +
        `(${e.message})\n\n` +
        `start it locally with:\n  cargo run -p gbasic-compile-service`,
    };
  }

  if (result.errors) {
    log("✖ compile failed:");
    log(result.errors);
    finishRun();
    return;
  }

  const wasmBytes = base64ToBytes(result.wasm);
  log(`✓ compiled (${wasmBytes.length} bytes wasm, ${result.js.length} chars js)`);
  log("Starting sandboxed runtime…");

  mountRunner(result.js, result.wasm);
  canvas.focus();
}

function finishRun() {
  running = false;
  runBtn.disabled = false;
  stopBtn.disabled = true;
}

function base64ToBytes(b64) {
  const bin = atob(b64);
  const out = new Uint8Array(bin.length);
  for (let i = 0; i < bin.length; i++) out[i] = bin.charCodeAt(i);
  return out;
}

function stopProgram() {
  unmountRunner();
  running = false;
  runBtn.disabled = false;
  stopBtn.disabled = true;
  log("[stopped]");
}

function shareProgram() {
  // STUB: full share-URL flow lands Day 5.
  log("Share lands Day 5.");
}

function log(msg) {
  consoleEl.textContent += msg + "\n";
  consoleEl.scrollTop = consoleEl.scrollHeight;
}

function mountRunner(runtimeJs, wasmBase64) {
  unmountRunner();

  const spriteRoot = new URL("assets/sprites/", window.location.href).href;
  const soundRoot = new URL("assets/sounds/", window.location.href).href;
  const html = `<!DOCTYPE html>
<html>
<head>
  <meta charset="utf-8" />
  <style>
    html, body { margin: 0; height: 100%; overflow: hidden; background: #1e1e2e; color: #cdd6f4; font-family: system-ui, sans-serif; }
    #canvas { width: 100%; height: calc(100% - 84px); display: block; outline: none; background: #1e1e2e; }
    #output { height: 84px; margin: 0; padding: 8px; overflow: auto; background: #181825; color: #a6e3a1; font: 12px ui-monospace, SFMono-Regular, Menlo, Consolas, monospace; white-space: pre-wrap; }
  </style>
</head>
<body>
  <canvas id="canvas" width="800" height="600" tabindex="0"></canvas>
  <pre id="output"></pre>
  <script>
    window.onerror = function(msg, src, line) {
      document.getElementById("output").textContent += "ERROR: " + msg + " at line " + line + "\\n";
    };
  </script>
  <script>${escapeScript(runtimeJs)}</script>
  <script>
    function base64ToBytes(b64) {
      const bin = atob(b64);
      const out = new Uint8Array(bin.length);
      for (let i = 0; i < bin.length; i++) out[i] = bin.charCodeAt(i);
      return out;
    }

    if (typeof configureGBasicRuntime === "function") {
      configureGBasicRuntime({
        assetSpriteRoot: ${JSON.stringify(spriteRoot)},
        assetSoundRoot: ${JSON.stringify(soundRoot)}
      });
    }

    loadAndRunBytes(base64ToBytes(${JSON.stringify(wasmBase64)}))
      .then(() => document.getElementById("canvas").focus())
      .catch((e) => {
        document.getElementById("output").textContent += "LOAD ERROR: " + e.message + "\\n";
        console.error(e);
      });
  </script>
</body>
</html>`;

  runnerFrame = document.createElement("iframe");
  runnerFrame.title = "G-Basic program output";
  runnerFrame.setAttribute("sandbox", "allow-scripts");
  runnerFrame.srcdoc = html;
  runnerHost.replaceChildren(runnerFrame);
  runnerFrame.addEventListener("load", () => {
    log("Runtime started. Click the canvas to focus keyboard input.");
  }, { once: true });
}

function unmountRunner() {
  if (!runnerFrame) return;
  runnerFrame.remove();
  runnerFrame = null;
  runnerHost.replaceChildren(canvas);
}

function escapeScript(js) {
  return js
    .replace(/<\/script/gi, "<\\/script")
    .replace(/<!--/g, "<\\!--");
}

async function loadAssetPanel() {
  try {
    const res = await fetch(ASSET_MANIFEST_URL);
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    const manifest = await res.json();
    renderAssetButtons(spriteAssetsEl, manifest.sprites || [], (name) => `sprite("${name}")`);
    renderAssetButtons(soundAssetsEl, manifest.sounds || [], (name) => `play("${name}")`);
  } catch (e) {
    console.warn("asset manifest unavailable:", e);
  }
}

function renderAssetButtons(container, assets, snippetFor) {
  if (!container) return;
  container.textContent = "";
  for (const asset of assets) {
    const button = document.createElement("button");
    button.type = "button";
    button.textContent = asset.name;
    button.title = `Insert ${asset.name}`;
    button.addEventListener("click", () => insertSnippet(snippetFor(asset.name)));
    container.appendChild(button);
  }
}

function insertSnippet(snippet) {
  if (!editor) return;
  const selection = editor.getSelection();
  editor.executeEdits("asset-picker", [{ range: selection, text: snippet, forceMoveMarkers: true }]);
  editor.focus();
}
