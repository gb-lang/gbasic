//! Parity test: ensure all runtime functions used by codegen are provided by the JS glue.

#[test]
fn test_runtime_function_parity() {
    let js_fns = gbasic_irgen::web_glue::js_runtime_function_names();

    // Read the JS runtime source and verify each function name appears in it
    let runtime_js = include_str!("../../irgen/src/web_glue.rs");

    let mut missing = Vec::new();
    for name in &js_fns {
        // Each function should appear as a key in the env object: `name(`
        if !runtime_js.contains(&format!("{name}(")) && !runtime_js.contains(&format!("{name} (")) {
            missing.push(*name);
        }
    }

    assert!(
        missing.is_empty(),
        "JS runtime is missing these functions:\n{}",
        missing.join("\n")
    );
}

#[test]
fn test_all_codegen_runtime_calls_covered() {
    // Read the llvm_backend source and extract all runtime function names called
    let backend_src = include_str!("../../irgen/src/llvm_backend.rs");
    let js_fns: std::collections::HashSet<&str> =
        gbasic_irgen::web_glue::js_runtime_function_names()
            .into_iter()
            .collect();

    let mut uncovered = Vec::new();

    // Find all call_runtime("name" patterns
    for line in backend_src.lines() {
        if let Some(start) = line.find("call_runtime(\"") {
            let rest = &line[start + 14..];
            if let Some(end) = rest.find('"') {
                let name = &rest[..end];
                if !js_fns.contains(name) {
                    uncovered.push(name.to_string());
                }
            }
        }
        // Also check add_function("runtime_ patterns
        if let Some(start) = line.find("add_function(\"runtime_") {
            let rest = &line[start + 14..];
            if let Some(end) = rest.find('"') {
                let name = &rest[..end];
                if !js_fns.contains(name) {
                    uncovered.push(name.to_string());
                }
            }
        }
    }

    // Deduplicate
    uncovered.sort();
    uncovered.dedup();

    assert!(
        uncovered.is_empty(),
        "These runtime functions from codegen are not covered by JS glue:\n{}",
        uncovered.join("\n")
    );
}
