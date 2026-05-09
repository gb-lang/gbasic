// G-Basic Playground.
//
// Sends source to the compile service (services/compile/) which returns
// a wasm bundle + runtime.js. The playground instantiates that locally
// and runs it against the canvas. The real WASM runtime wiring (sprite
// + sound) lands Day 2; for now successful compiles are reported and
// the wasm size is logged so the round-trip is observable.

const COMPILE_ENDPOINT =
  (typeof window !== "undefined" && window.__GBASIC_COMPILE_URL) ||
  "http://localhost:8080/compile";

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

const runBtn = document.getElementById("run-btn");
const stopBtn = document.getElementById("stop-btn");
const shareBtn = document.getElementById("share-btn");
const canvas = document.getElementById("canvas");
const consoleEl = document.getElementById("console");

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
  log("[Day 2] WASM instantiation + runtime canvas painting lands next.");

  // Day 2 will replace this placeholder with eval(result.js) +
  // WebAssembly.instantiate(wasmBytes). For now we only verify the
  // round-trip and surface compile output to the console.
  ctx.fillStyle = "#cdd6f4";
  ctx.font = "20px sans-serif";
  ctx.fillText("compile OK — runtime wiring lands Day 2", 40, 60);

  canvas.focus();
  finishRun();
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
  // STUB: real stop arrives once WASM is wired in (Day 2).
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

function sleep(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}
