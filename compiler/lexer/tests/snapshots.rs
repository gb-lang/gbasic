use gbasic_lexer::tokenize;

#[test]
fn test_full_program() {
    let tokens = tokenize("let x = 42\nprint(x)");
    insta::assert_debug_snapshot!(tokens);
}

#[test]
fn test_method_chain() {
    let tokens = tokenize("Screen.Init(800, 600)");
    insta::assert_debug_snapshot!(tokens);
}

#[test]
fn test_operators() {
    let tokens = tokenize("1 + 2 * 3 == 7 and true");
    insta::assert_debug_snapshot!(tokens);
}

#[test]
fn test_control_flow() {
    let tokens = tokenize("if true { break } else { continue }");
    insta::assert_debug_snapshot!(tokens);
}

#[test]
fn test_string_literal() {
    let tokens = tokenize(r#""hello world""#);
    insta::assert_debug_snapshot!(tokens);
}
