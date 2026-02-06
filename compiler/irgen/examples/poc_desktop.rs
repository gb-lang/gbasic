//! Desktop PoC: Hardcoded LLVM IR that opens a blue 800x600 SDL2 window.
//!
//! Generates an object file, links with the desktop runtime, produces an executable.
//! Run: cargo run --example poc_desktop -p gbasic-irgen
//! Then: ./poc_desktop

use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::targets::{
    CodeModel, FileType, InitializationConfig, RelocMode, Target, TargetMachine,
};
use inkwell::OptimizationLevel;
use std::path::Path;
use std::process::Command;

fn build_module(context: &Context) -> Module<'_> {
    let module = context.create_module("poc_desktop");
    let builder = context.create_builder();

    let i32_type = context.i32_type();
    let i8_type = context.i8_type();
    let void_type = context.void_type();

    // Declare extern runtime functions
    let init_fn_type = void_type.fn_type(&[i32_type.into(), i32_type.into()], false);
    let init_fn = module.add_function("runtime_init", init_fn_type, None);

    let clear_fn_type = void_type.fn_type(
        &[i8_type.into(), i8_type.into(), i8_type.into()],
        false,
    );
    let clear_fn = module.add_function("runtime_clear_screen", clear_fn_type, None);

    let present_fn_type = void_type.fn_type(&[], false);
    let present_fn = module.add_function("runtime_present", present_fn_type, None);

    let quit_fn_type = i32_type.fn_type(&[], false);
    let quit_fn = module.add_function("runtime_should_quit", quit_fn_type, None);

    let shutdown_fn_type = void_type.fn_type(&[], false);
    let shutdown_fn = module.add_function("runtime_shutdown", shutdown_fn_type, None);

    // Build main function
    let main_fn_type = i32_type.fn_type(&[], false);
    let main_fn = module.add_function("main", main_fn_type, None);

    let entry_bb = context.append_basic_block(main_fn, "entry");
    let loop_bb = context.append_basic_block(main_fn, "loop");
    let exit_bb = context.append_basic_block(main_fn, "exit");

    // Entry: call runtime_init(800, 600)
    builder.position_at_end(entry_bb);
    builder.build_call(
        init_fn,
        &[i32_type.const_int(800, false).into(), i32_type.const_int(600, false).into()],
        "",
    ).unwrap();
    builder.build_unconditional_branch(loop_bb).unwrap();

    // Loop: clear blue, present, check quit
    builder.position_at_end(loop_bb);
    builder.build_call(
        clear_fn,
        &[
            i8_type.const_int(30, false).into(),  // R
            i8_type.const_int(60, false).into(),  // G
            i8_type.const_int(180, false).into(), // B
        ],
        "",
    ).unwrap();
    builder.build_call(present_fn, &[], "").unwrap();

    let should_quit = builder
        .build_call(quit_fn, &[], "should_quit")
        .unwrap()
        .try_as_basic_value()
        .left()
        .unwrap()
        .into_int_value();

    let cmp = builder
        .build_int_compare(
            inkwell::IntPredicate::NE,
            should_quit,
            i32_type.const_int(0, false),
            "quit_cmp",
        )
        .unwrap();

    builder.build_conditional_branch(cmp, exit_bb, loop_bb).unwrap();

    // Exit: shutdown and return 0
    builder.position_at_end(exit_bb);
    builder.build_call(shutdown_fn, &[], "").unwrap();
    builder.build_return(Some(&i32_type.const_int(0, false))).unwrap();

    module
}

fn main() {
    let context = Context::create();
    let module = build_module(&context);

    // Verify the module
    module.verify().expect("Module verification failed");
    println!("LLVM IR verified successfully.");

    // Print IR for inspection
    module.print_to_stderr();

    // Initialize native target
    Target::initialize_native(&InitializationConfig::default())
        .expect("Failed to initialize native target");

    let triple = TargetMachine::get_default_triple();
    let target = Target::from_triple(&triple).expect("Failed to get target");
    let machine = target
        .create_target_machine(
            &triple,
            "generic",
            "",
            OptimizationLevel::Default,
            RelocMode::PIC,
            CodeModel::Default,
        )
        .expect("Failed to create target machine");

    // Emit object file
    let obj_path = Path::new("poc_desktop.o");
    machine
        .write_to_file(&module, FileType::Object, obj_path)
        .expect("Failed to write object file");
    println!("Object file written to poc_desktop.o");

    // Link with runtime
    // Find the runtime static library
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let workspace_root = Path::new(manifest_dir).parent().unwrap().parent().unwrap();
    let target_dir = workspace_root.join("target/debug");

    // The runtime staticlib name
    let lib_name = if cfg!(target_os = "macos") {
        "libgbasic_runtime_desktop.a"
    } else {
        "libgbasic_runtime_desktop.a"
    };

    let runtime_lib = target_dir.join(lib_name);
    println!("Looking for runtime at: {}", runtime_lib.display());

    // Find bundled SDL2 library from sdl2-sys build
    let build_dir = target_dir.join("build");
    let mut sdl2_lib_dir = None;
    if let Ok(entries) = std::fs::read_dir(&build_dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with("sdl2-sys-") {
                let lib_path = entry.path().join("out/lib");
                if lib_path.exists() {
                    sdl2_lib_dir = Some(lib_path);
                }
            }
        }
    }

    let sdl2_dir = sdl2_lib_dir.expect("Could not find bundled SDL2 library. Build runtime/desktop first.");
    println!("SDL2 bundled lib dir: {}", sdl2_dir.display());

    let mut cmd = Command::new("cc");
    cmd.arg("poc_desktop.o")
        .arg("-o")
        .arg("poc_desktop")
        .arg(runtime_lib.to_str().unwrap())
        .arg(format!("-L{}", sdl2_dir.display()))
        .arg("-lSDL2")
        .arg("-framework").arg("Cocoa")
        .arg("-framework").arg("IOKit")
        .arg("-framework").arg("CoreVideo")
        .arg("-framework").arg("CoreAudio")
        .arg("-framework").arg("AudioToolbox")
        .arg("-framework").arg("Carbon")
        .arg("-framework").arg("ForceFeedback")
        .arg("-framework").arg("GameController")
        .arg("-framework").arg("CoreHaptics")
        .arg("-framework").arg("Metal")
        .arg("-liconv");

    println!("Linking: {:?}", cmd);
    let link_result = cmd.status();

    match link_result {
        Ok(status) if status.success() => {
            println!("\nSuccess! Run ./poc_desktop to see a blue window.");
            println!("Press ESC or close the window to quit.");
        }
        Ok(status) => {
            eprintln!("Linking failed with status: {}", status);
            std::process::exit(1);
        }
        Err(e) => {
            eprintln!("Failed to run linker: {}", e);
            std::process::exit(1);
        }
    }

    // Clean up object file
    let _ = std::fs::remove_file("poc_desktop.o");
}
