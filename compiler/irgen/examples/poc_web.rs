//! Web PoC: Hardcoded LLVM IR that produces a .wasm module + HTML/JS glue.
//!
//! Run: LLVM_SYS_180_PREFIX=/usr/local/opt/llvm@18 cargo run --example poc_web -p gbasic-irgen
//! Then: cd poc_web_output && python3 -m http.server 8080
//! Open: http://localhost:8080/poc_web.html

use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::targets::{
    CodeModel, FileType, InitializationConfig, RelocMode, Target, TargetTriple,
};
use inkwell::OptimizationLevel;
use std::fs;
use std::path::Path;

fn build_module(context: &Context) -> Module<'_> {
    let module = context.create_module("poc_web");
    let builder = context.create_builder();

    let i8_type = context.i8_type();
    let void_type = context.void_type();

    // Declare imported JS functions
    let clear_fn_type = void_type.fn_type(
        &[i8_type.into(), i8_type.into(), i8_type.into()],
        false,
    );
    let clear_fn = module.add_function("js_clear_screen", clear_fn_type, None);

    let present_fn_type = void_type.fn_type(&[], false);
    let present_fn = module.add_function("js_present", present_fn_type, None);

    // Build exported start function
    let start_fn_type = void_type.fn_type(&[], false);
    let start_fn = module.add_function("start", start_fn_type, None);

    let entry_bb = context.append_basic_block(start_fn, "entry");

    builder.position_at_end(entry_bb);
    // Clear to blue
    builder.build_call(
        clear_fn,
        &[
            i8_type.const_int(30, false).into(),
            i8_type.const_int(60, false).into(),
            i8_type.const_int(180, false).into(),
        ],
        "",
    ).unwrap();

    builder.build_call(present_fn, &[], "").unwrap();
    builder.build_return(None).unwrap();

    module
}

fn main() {
    let context = Context::create();
    let module = build_module(&context);

    module.verify().expect("Module verification failed");
    println!("LLVM IR verified successfully.");
    module.print_to_stderr();

    // Initialize wasm32 target
    Target::initialize_webassembly(&InitializationConfig::default());

    let triple = TargetTriple::create("wasm32-unknown-unknown");
    let target = Target::from_triple(&triple).expect("Failed to get wasm32 target");
    let machine = target
        .create_target_machine(
            &triple,
            "generic",
            "",
            OptimizationLevel::Default,
            RelocMode::PIC,
            CodeModel::Default,
        )
        .expect("Failed to create wasm32 target machine");

    // Create output directory
    let out_dir = Path::new("poc_web_output");
    fs::create_dir_all(out_dir).expect("Failed to create output dir");

    // Emit wasm object
    let obj_path = out_dir.join("poc_web.o");
    machine
        .write_to_file(&module, FileType::Object, &obj_path)
        .expect("Failed to write wasm object");

    // Use wasm-ld to link into final .wasm
    let wasm_path = out_dir.join("poc_web.wasm");
    let ld_result = std::process::Command::new("/usr/local/opt/llvm@18/bin/wasm-ld")
        .arg("--no-entry")
        .arg("--export-all")
        .arg("--allow-undefined")
        .arg("-o")
        .arg(wasm_path.to_str().unwrap())
        .arg(obj_path.to_str().unwrap())
        .status();

    match ld_result {
        Ok(s) if s.success() => println!("Wasm linked successfully."),
        Ok(s) => {
            eprintln!("wasm-ld failed: {}", s);
            eprintln!("Make sure LLVM 18 wasm-ld is available.");
            std::process::exit(1);
        }
        Err(e) => {
            eprintln!("Failed to run wasm-ld: {}", e);
            std::process::exit(1);
        }
    }

    // Clean up object file
    let _ = fs::remove_file(&obj_path);

    // Generate HTML
    let html = r#"<!DOCTYPE html>
<html>
<head>
    <title>G-Basic Web PoC</title>
    <style>
        body { margin: 0; display: flex; justify-content: center; align-items: center; height: 100vh; background: #111; }
        canvas { border: 1px solid #333; }
    </style>
</head>
<body>
    <canvas id="canvas" width="800" height="600"></canvas>
    <script src="poc_web.js"></script>
</body>
</html>"#;
    fs::write(out_dir.join("poc_web.html"), html).expect("Failed to write HTML");

    // Generate JS glue
    let js = r#"
const canvas = document.getElementById('canvas');
const ctx = canvas.getContext('2d');

const importObject = {
    env: {
        js_clear_screen: (r, g, b) => {
            ctx.fillStyle = `rgb(${r}, ${g}, ${b})`;
            ctx.fillRect(0, 0, canvas.width, canvas.height);
        },
        js_present: () => {
            // No-op — canvas updates are immediate
        }
    }
};

async function init() {
    const response = await fetch('poc_web.wasm');
    const bytes = await response.arrayBuffer();
    const { instance } = await WebAssembly.instantiate(bytes, importObject);
    instance.exports.start();
    console.log('G-Basic Web PoC running!');
}

init().catch(console.error);
"#;
    fs::write(out_dir.join("poc_web.js"), js).expect("Failed to write JS");

    println!("\nWeb PoC generated in poc_web_output/");
    println!("Run: cd poc_web_output && python3 -m http.server 8080");
    println!("Open: http://localhost:8080/poc_web.html");
}
