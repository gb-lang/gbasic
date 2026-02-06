pub mod expr;
pub mod stmt;
pub mod method_chain;

use gbasic_common::ast::*;
use gbasic_common::error::GBasicError;
use gbasic_common::span::Span;
use gbasic_lexer::{tokenize, SpannedToken, Token};

pub struct Parser {
    tokens: Vec<SpannedToken>,
    pos: usize,
    errors: Vec<GBasicError>,
}

impl Parser {
    pub fn new(tokens: Vec<SpannedToken>) -> Self {
        Self {
            tokens,
            pos: 0,
            errors: Vec::new(),
        }
    }

    pub fn current(&self) -> &Token {
        &self.tokens[self.pos].token
    }

    pub fn current_span(&self) -> Span {
        self.tokens[self.pos].span
    }

    pub fn peek(&self) -> &Token {
        self.current()
    }

    pub fn peek_ahead(&self, n: usize) -> &Token {
        let idx = (self.pos + n).min(self.tokens.len() - 1);
        &self.tokens[idx].token
    }

    pub fn advance(&mut self) -> &SpannedToken {
        let tok = &self.tokens[self.pos];
        if self.pos < self.tokens.len() - 1 {
            self.pos += 1;
        }
        tok
    }

    pub fn expect(&mut self, expected: &Token) -> Result<Span, GBasicError> {
        if self.current() == expected {
            Ok(self.advance().span)
        } else {
            Err(GBasicError::SyntaxError {
                message: format!("expected '{}', found '{}'", expected, self.current()),
                span: self.current_span(),
            })
        }
    }

    pub fn at(&self, token: &Token) -> bool {
        std::mem::discriminant(self.current()) == std::mem::discriminant(token)
            || self.current() == token
    }

    pub fn at_end(&self) -> bool {
        matches!(self.current(), Token::Eof)
    }

    pub fn skip_newlines(&mut self) {
        while matches!(self.current(), Token::Newline) {
            self.advance();
        }
    }

    pub fn error(&mut self, err: GBasicError) {
        self.errors.push(err);
    }

    /// Synchronize after an error by skipping to the next statement boundary.
    pub fn synchronize(&mut self) {
        loop {
            match self.current() {
                Token::Eof => return,
                Token::Let | Token::Fun | Token::Fn | Token::If | Token::For | Token::While
                | Token::Match | Token::Return | Token::Break | Token::Continue => return,
                Token::RBrace => {
                    self.advance();
                    return;
                }
                Token::Newline | Token::Semicolon => {
                    self.advance();
                    return;
                }
                _ => {
                    self.advance();
                }
            }
        }
    }

    pub fn parse_program(&mut self) -> Program {
        let start = self.current_span();
        let mut statements = Vec::new();

        self.skip_newlines();
        while !self.at_end() {
            match self.parse_statement() {
                Ok(stmt) => statements.push(stmt),
                Err(e) => {
                    self.error(e);
                    self.synchronize();
                }
            }
            self.skip_newlines();
        }

        let end = self.current_span();
        Program {
            statements,
            span: start.merge(end),
        }
    }
}

/// Parse source code into a Program AST.
pub fn parse(source: &str) -> Result<Program, Vec<GBasicError>> {
    let tokens = tokenize(source);
    let mut parser = Parser::new(tokens);
    let program = parser.parse_program();

    if parser.errors.is_empty() {
        Ok(program)
    } else {
        Err(parser.errors)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_let() {
        let program = parse("let x = 42").unwrap();
        assert_eq!(program.statements.len(), 1);
        assert!(matches!(&program.statements[0], Statement::Let { .. }));
    }

    #[test]
    fn test_parse_function() {
        let program = parse("fn greet(name: String) { }").unwrap();
        assert_eq!(program.statements.len(), 1);
        assert!(matches!(&program.statements[0], Statement::Function(_)));
    }

    #[test]
    fn test_parse_if_else() {
        let program = parse("if x == 1 { } else { }").unwrap();
        assert_eq!(program.statements.len(), 1);
        assert!(matches!(
            &program.statements[0],
            Statement::If { else_block: Some(_), .. }
        ));
    }

    #[test]
    fn test_parse_while() {
        let program = parse("while x > 0 { }").unwrap();
        assert_eq!(program.statements.len(), 1);
        assert!(matches!(&program.statements[0], Statement::While { .. }));
    }

    #[test]
    fn test_parse_for() {
        let program = parse("for i in items { }").unwrap();
        assert_eq!(program.statements.len(), 1);
        assert!(matches!(&program.statements[0], Statement::For { .. }));
    }

    #[test]
    fn test_parse_method_chain() {
        let program = parse("Screen.Layer(1).Draw()").unwrap();
        assert_eq!(program.statements.len(), 1);
        if let Statement::Expression { expr, .. } = &program.statements[0] {
            assert!(matches!(expr, Expression::MethodChain { .. }));
        } else {
            panic!("expected expression statement");
        }
    }

    #[test]
    fn test_parse_binary_precedence() {
        let program = parse("let x = 1 + 2 * 3").unwrap();
        if let Statement::Let { value, .. } = &program.statements[0] {
            // Should be Add(1, Mul(2, 3))
            if let Expression::BinaryOp { op, right, .. } = value {
                assert_eq!(*op, BinaryOp::Add);
                assert!(matches!(
                    right.as_ref(),
                    Expression::BinaryOp { op: BinaryOp::Mul, .. }
                ));
            } else {
                panic!("expected binary op");
            }
        }
    }

    #[test]
    fn test_parse_array_literal() {
        let program = parse("let xs = [1, 2, 3]").unwrap();
        if let Statement::Let { value, .. } = &program.statements[0] {
            if let Expression::Array { elements, .. } = value {
                assert_eq!(elements.len(), 3);
            } else {
                panic!("expected array");
            }
        }
    }

    #[test]
    fn test_parse_nested_expressions() {
        let program = parse("let x = (1 + 2) * 3").unwrap();
        assert_eq!(program.statements.len(), 1);
    }

    #[test]
    fn test_parse_return() {
        let program = parse("fn foo() { return 42 }").unwrap();
        if let Statement::Function(f) = &program.statements[0] {
            assert!(matches!(
                &f.body.statements[0],
                Statement::Return { value: Some(_), .. }
            ));
        }
    }

    #[test]
    fn test_error_recovery() {
        let result = parse("let = 42");
        assert!(result.is_err());
    }

    #[test]
    fn test_case_insensitive_parse() {
        let program = parse("LET X = 42").unwrap();
        assert_eq!(program.statements.len(), 1);
    }

    #[test]
    fn test_multiple_statements() {
        let src = "let x = 1\nlet y = 2\nlet z = x + y";
        let program = parse(src).unwrap();
        assert_eq!(program.statements.len(), 3);
    }

    #[test]
    fn test_print_builtin() {
        let program = parse(r#"print("Hello!")"#).unwrap();
        assert_eq!(program.statements.len(), 1);
        if let Statement::Expression { expr, .. } = &program.statements[0] {
            assert!(matches!(expr, Expression::Call { .. }));
        } else {
            panic!("expected expression statement");
        }
    }

    #[test]
    fn test_string_interpolation_simple() {
        let program = parse(r#"print("Hello, {name}!")"#).unwrap();
        if let Statement::Expression { expr, .. } = &program.statements[0] {
            if let Expression::Call { args, .. } = expr {
                assert!(matches!(&args[0], Expression::StringInterp { parts, .. } if parts.len() == 3));
            } else {
                panic!("expected call");
            }
        }
    }

    #[test]
    fn test_string_interpolation_with_expr() {
        let program = parse(r#"print("{x + y}")"#).unwrap();
        if let Statement::Expression { expr, .. } = &program.statements[0] {
            if let Expression::Call { args, .. } = expr {
                if let Expression::StringInterp { parts, .. } = &args[0] {
                    assert_eq!(parts.len(), 1);
                    assert!(matches!(&parts[0], StringPart::Expr(Expression::BinaryOp { .. })));
                } else {
                    panic!("expected interp");
                }
            }
        }
    }

    #[test]
    fn test_no_main_required() {
        let program = parse("let x = 42\nprint(\"done\")").unwrap();
        assert_eq!(program.statements.len(), 2);
        // Top-level code works without fn main
    }

    #[test]
    fn test_plain_string_no_interp() {
        let program = parse(r#"let s = "no braces here""#).unwrap();
        if let Statement::Let { value, .. } = &program.statements[0] {
            assert!(matches!(value, Expression::Literal(Literal { kind: LiteralKind::String(_), .. })));
        }
    }

    #[test]
    fn test_fun_keyword() {
        let program = parse("fun greet(name) { }").unwrap();
        assert_eq!(program.statements.len(), 1);
        assert!(matches!(&program.statements[0], Statement::Function(_)));
    }

    #[test]
    fn test_fn_still_works() {
        let program = parse("fn add(a: Int, b: Int) -> Int { a + b }").unwrap();
        assert!(matches!(&program.statements[0], Statement::Function(_)));
    }

    #[test]
    fn test_optional_param_types() {
        let program = parse("fun greet(who) { }").unwrap();
        if let Statement::Function(f) = &program.statements[0] {
            assert!(f.params[0].type_ann.is_none());
        }
    }

    #[test]
    fn test_mixed_param_types() {
        let program = parse("fun foo(a, b: Int, c) { }").unwrap();
        if let Statement::Function(f) = &program.statements[0] {
            assert!(f.params[0].type_ann.is_none());
            assert!(f.params[1].type_ann.is_some());
            assert!(f.params[2].type_ann.is_none());
        }
    }

    #[test]
    fn test_and_or_keywords() {
        let program = parse("if x > 0 and y < 10 { }").unwrap();
        assert_eq!(program.statements.len(), 1);
    }

    #[test]
    fn test_or_keyword() {
        let program = parse("if a or b { }").unwrap();
        if let Statement::If { condition, .. } = &program.statements[0] {
            assert!(matches!(condition, Expression::BinaryOp { op: BinaryOp::Or, .. }));
        }
    }

    #[test]
    fn test_not_keyword() {
        let program = parse("if not alive { }").unwrap();
        if let Statement::If { condition, .. } = &program.statements[0] {
            assert!(matches!(condition, Expression::UnaryOp { op: UnaryOp::Not, .. }));
        }
    }

    #[test]
    fn test_and_or_symbol_aliases() {
        // && and || still work
        let program = parse("if x && y || z { }").unwrap();
        assert_eq!(program.statements.len(), 1);
    }
}
