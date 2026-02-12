use gbasic_parser::parse;

#[test]
fn test_let_with_type() {
    let program = parse("let x: Int = 42").unwrap();
    insta::assert_yaml_snapshot!(program);
}

#[test]
fn test_function_decl() {
    let program = parse("fun add(a: Int, b: Int) -> Int { return a + b }").unwrap();
    insta::assert_yaml_snapshot!(program);
}

#[test]
fn test_method_chain() {
    let program = parse("Screen.DrawRect(10, 20, 100, 50, 255, 0, 0)").unwrap();
    insta::assert_yaml_snapshot!(program);
}

#[test]
fn test_for_loop() {
    let program = parse("for i in 0..10 { print(i) }").unwrap();
    insta::assert_yaml_snapshot!(program);
}

#[test]
fn test_match_stmt() {
    let program = parse("match x { 1 -> { print(\"one\") } _ -> { print(\"other\") } }").unwrap();
    insta::assert_yaml_snapshot!(program);
}
