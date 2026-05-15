// G-Basic Playground.
//
// Sends source to the compile service (services/compile/) which returns
// a wasm bundle + runtime.js. The playground instantiates that locally
// in a per-run iframe sandbox and runs it against the canvas.

const COMPILE_ENDPOINT =
  (typeof window !== "undefined" && window.__GBASIC_COMPILE_URL) ||
  "http://localhost:8080/compile";
const ASSET_MANIFEST_URL = "assets/manifest.json";
const LESSON_MANIFEST_URL = "lessons/manifest.json";
const LESSON_PROGRESS_KEY = "gbasic.lessonProgress.v1";

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
let lessons = [];
let currentLesson = 1;
let lessonProgress = loadLessonProgress();

const runBtn = document.getElementById("run-btn");
const stopBtn = document.getElementById("stop-btn");
const shareBtn = document.getElementById("share-btn");
const canvas = document.getElementById("canvas");
const consoleEl = document.getElementById("console");
const runnerHost = document.getElementById("runner-host");
const spriteAssetsEl = document.getElementById("sprite-assets");
const soundAssetsEl = document.getElementById("sound-assets");
const lessonKickerEl = document.getElementById("lesson-kicker");
const lessonTitleEl = document.getElementById("lesson-title");
const lessonGoalEl = document.getElementById("lesson-goal");
const lessonBodyEl = document.getElementById("lesson-body");
const lessonDotsEl = document.getElementById("lesson-dots");
const prevLessonBtn = document.getElementById("prev-lesson-btn");
const nextLessonBtn = document.getElementById("next-lesson-btn");
const loadStarterBtn = document.getElementById("load-starter-btn");
const showSolutionBtn = document.getElementById("show-solution-btn");
const completeLessonBtn = document.getElementById("complete-lesson-btn");

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
  loadProgramFromUrl();
});

runBtn.addEventListener("click", runProgram);
stopBtn.addEventListener("click", stopProgram);
shareBtn.addEventListener("click", shareProgram);
prevLessonBtn.addEventListener("click", () => goToLesson(currentLesson - 1));
nextLessonBtn.addEventListener("click", () => goToLesson(currentLesson + 1));
loadStarterBtn.addEventListener("click", () => loadLessonCode("starter"));
showSolutionBtn.addEventListener("click", () => loadLessonCode("solution"));
completeLessonBtn.addEventListener("click", completeCurrentLesson);
loadAssetPanel();
loadLessons();

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
  if (!editor) return;
  const source = editor.getValue();
  const title = window.prompt("Name this program", "My G-Basic game") || "My G-Basic game";
  const params = new URLSearchParams();
  params.set("code", encodeProgram(source));
  params.set("title", title);
  const url = new URL(window.location.href);
  url.hash = params.toString();
  navigator.clipboard?.writeText(url.toString()).then(
    () => log("Share link copied."),
    () => log(url.toString())
  );
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

async function loadLessons() {
  try {
    const res = await fetch(LESSON_MANIFEST_URL);
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    lessons = await res.json();
    currentLesson = lessonFromLocation() || 1;
    await renderLesson(currentLesson);
  } catch (e) {
    console.warn("lesson manifest unavailable:", e);
  }
}

function lessonFromLocation() {
  const pathMatch = window.location.pathname.match(/\/learn\/(\d+)/);
  if (pathMatch) return Number(pathMatch[1]);
  const hashMatch = window.location.hash.match(/learn\/(\d+)/);
  if (hashMatch) return Number(hashMatch[1]);
  return null;
}

async function goToLesson(id) {
  if (!lessons.some((lesson) => lesson.id === id)) return;
  history.pushState({}, "", `#/learn/${id}`);
  await renderLesson(id);
}

window.addEventListener("popstate", () => {
  const id = lessonFromLocation();
  if (id) renderLesson(id);
});

async function renderLesson(id) {
  const lesson = lessons.find((item) => item.id === id) || lessons[0];
  if (!lesson) return;
  currentLesson = lesson.id;

  lessonKickerEl.textContent = `Lesson ${lesson.id} of ${lessons.length}`;
  lessonTitleEl.textContent = lesson.title;
  lessonGoalEl.textContent = lesson.goal;
  prevLessonBtn.disabled = lesson.id === 1;
  nextLessonBtn.disabled = lesson.id === lessons.length;
  completeLessonBtn.textContent = lessonProgress.has(String(lesson.id)) ? "Done ✓" : "Done";

  const md = await fetchText(lesson.markdown);
  lessonBodyEl.innerHTML = markdownSummary(md);
  renderLessonDots();
}

function renderLessonDots() {
  lessonDotsEl.textContent = "";
  for (const lesson of lessons) {
    const dot = document.createElement("span");
    dot.className = "lesson-dot";
    if (lesson.id === currentLesson) dot.classList.add("active");
    if (lessonProgress.has(String(lesson.id))) dot.classList.add("done");
    dot.title = `Lesson ${lesson.id}`;
    lessonDotsEl.appendChild(dot);
  }
}

async function loadLessonCode(kind) {
  const lesson = lessons.find((item) => item.id === currentLesson);
  if (!lesson || !editor) return;
  const url = kind === "solution" ? lesson.solution : lesson.starter;
  editor.setValue(await fetchText(url));
  editor.focus();
}

function completeCurrentLesson() {
  lessonProgress.add(String(currentLesson));
  saveLessonProgress();
  renderLessonDots();
  completeLessonBtn.textContent = "Done ✓";
}

async function fetchText(url) {
  const res = await fetch(url);
  if (!res.ok) throw new Error(`${url}: HTTP ${res.status}`);
  return res.text();
}

function markdownSummary(md) {
  const beforeStarter = md.split("## Starter Code")[0];
  return beforeStarter
    .split("\n")
    .filter((line) => line.trim() && !line.startsWith("# "))
    .map((line) => `<p>${inlineMarkdown(line.trim())}</p>`)
    .join("");
}

function inlineMarkdown(text) {
  return text
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/`([^`]+)`/g, "<code>$1</code>");
}

function loadLessonProgress() {
  try {
    return new Set(JSON.parse(localStorage.getItem(LESSON_PROGRESS_KEY) || "[]"));
  } catch {
    return new Set();
  }
}

function saveLessonProgress() {
  localStorage.setItem(LESSON_PROGRESS_KEY, JSON.stringify([...lessonProgress]));
}

function loadProgramFromUrl() {
  if (!editor || !window.location.hash.startsWith("#code=")) return;
  const params = new URLSearchParams(window.location.hash.slice(1));
  const encoded = params.get("code");
  if (!encoded) return;
  try {
    editor.setValue(decodeProgram(encoded));
    const title = params.get("title");
    if (title) log(`Opened shared program: ${title}`);
  } catch (e) {
    log(`Could not open shared program: ${e.message}`);
  }
}

function encodeProgram(source) {
  const bytes = new TextEncoder().encode(source);
  let binary = "";
  for (const byte of bytes) binary += String.fromCharCode(byte);
  return btoa(binary).replace(/\+/g, "-").replace(/\//g, "_").replace(/=+$/g, "");
}

function decodeProgram(encoded) {
  const padded = encoded.replace(/-/g, "+").replace(/_/g, "/").padEnd(Math.ceil(encoded.length / 4) * 4, "=");
  const binary = atob(padded);
  const bytes = new Uint8Array(binary.length);
  for (let i = 0; i < binary.length; i++) bytes[i] = binary.charCodeAt(i);
  return new TextDecoder().decode(bytes);
}
