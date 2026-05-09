// G-Basic Playground — Day 1 scaffold.
// Day 1 morning: editor + canvas + button wiring + stub Run.
// Day 1 afternoon: replaces stub Run with POST /compile to the axum service.

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

  // STUB compile path — Day 1 afternoon replaces this with a real
  // POST /compile call to the axum service.
  log("Compiling…");
  await sleep(150);

  const source = editor.getValue();
  const ctx = canvas.getContext("2d");
  ctx.fillStyle = "#1e1e2e";
  ctx.fillRect(0, 0, canvas.width, canvas.height);

  // Naive parse so the canvas isn't empty during scaffold demo.
  // Real interpretation comes from the WASM runtime in Day 2.
  const printed = [];
  for (const line of source.split("\n")) {
    const m = line.match(/^\s*print\s*\(\s*"([^"]*)"\s*\)/);
    if (m) printed.push(m[1]);
  }

  ctx.fillStyle = "#cdd6f4";
  ctx.font = "32px sans-serif";
  let y = 80;
  for (const text of printed) {
    ctx.fillText(text, 40, y);
    log(text);
    y += 48;
  }

  if (printed.length === 0) {
    log("[stub] real compilation lands Day 1 afternoon.");
  } else {
    log(`[stub] rendered ${printed.length} print() call${printed.length === 1 ? "" : "s"}.`);
  }

  canvas.focus();
  running = false;
  runBtn.disabled = false;
  stopBtn.disabled = true;
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
