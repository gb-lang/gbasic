pub mod token;

pub use token::{tokenize, SpannedToken, Token};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_let_binding() {
        let tokens = tokenize("let x = 42");
        let kinds: Vec<_> = tokens.iter().map(|t| &t.token).collect();
        assert_eq!(
            kinds,
            vec![
                &Token::Let,
                &Token::Ident("x".into()),
                &Token::Eq,
                &Token::Int(42),
                &Token::Eof,
            ]
        );
    }

    #[test]
    fn test_case_insensitive_keywords() {
        let tokens = tokenize("LET X = 42");
        let kinds: Vec<_> = tokens.iter().map(|t| &t.token).collect();
        assert_eq!(
            kinds,
            vec![
                &Token::Let,
                &Token::Ident("x".into()),
                &Token::Eq,
                &Token::Int(42),
                &Token::Eof,
            ]
        );
    }

    #[test]
    fn test_method_chain() {
        let tokens = tokenize("Screen.Layer(1).Sprite(\"hero\").Draw()");
        let kinds: Vec<_> = tokens.iter().map(|t| &t.token).collect();
        assert_eq!(
            kinds,
            vec![
                &Token::Screen,
                &Token::Dot,
                &Token::Ident("layer".into()),
                &Token::LParen,
                &Token::Int(1),
                &Token::RParen,
                &Token::Dot,
                &Token::Ident("sprite".into()),
                &Token::LParen,
                &Token::String("hero".into()),
                &Token::RParen,
                &Token::Dot,
                &Token::Ident("draw".into()),
                &Token::LParen,
                &Token::RParen,
                &Token::Eof,
            ]
        );
    }

    #[test]
    fn test_operators() {
        let tokens = tokenize("a + b * c == d && !e");
        let kinds: Vec<_> = tokens
            .iter()
            .map(|t| &t.token)
            .filter(|t| !matches!(t, Token::Eof))
            .collect();
        assert_eq!(
            kinds,
            vec![
                &Token::Ident("a".into()),
                &Token::Plus,
                &Token::Ident("b".into()),
                &Token::Star,
                &Token::Ident("c".into()),
                &Token::EqEq,
                &Token::Ident("d".into()),
                &Token::AmpAmp,
                &Token::Bang,
                &Token::Ident("e".into()),
            ]
        );
    }

    #[test]
    fn test_float_literal() {
        let tokens = tokenize("3.14");
        assert_eq!(tokens[0].token, Token::Float(3.14));
    }

    #[test]
    fn test_string_literal() {
        let tokens = tokenize(r#""hello world""#);
        assert_eq!(tokens[0].token, Token::String("hello world".into()));
    }

    #[test]
    fn test_function_def() {
        let tokens = tokenize("fn update(dt: Float) -> Void { }");
        let kinds: Vec<_> = tokens.iter().map(|t| &t.token).collect();
        assert_eq!(
            kinds,
            vec![
                &Token::Fn,
                &Token::Ident("update".into()),
                &Token::LParen,
                &Token::Ident("dt".into()),
                &Token::Colon,
                &Token::TyFloat,
                &Token::RParen,
                &Token::Arrow,
                &Token::TyVoid,
                &Token::LBrace,
                &Token::RBrace,
                &Token::Eof,
            ]
        );
    }

    #[test]
    fn test_error_recovery() {
        let tokens = tokenize("let x = @42");
        // Should produce Error token for @ but continue
        assert!(tokens.iter().any(|t| t.token == Token::Error));
        assert!(tokens.iter().any(|t| t.token == Token::Int(42)));
    }

    #[test]
    fn test_comments_skipped() {
        let tokens = tokenize("let x = 1 // this is a comment");
        let kinds: Vec<_> = tokens
            .iter()
            .map(|t| &t.token)
            .filter(|t| !matches!(t, Token::Newline | Token::Eof))
            .collect();
        assert_eq!(
            kinds,
            vec![
                &Token::Let,
                &Token::Ident("x".into()),
                &Token::Eq,
                &Token::Int(1),
            ]
        );
    }

    #[test]
    fn test_fun_keyword() {
        let tokens = tokenize("fun greet(name) { }");
        assert_eq!(tokens[0].token, Token::Fun);
    }

    #[test]
    fn test_and_or_not_keywords() {
        let tokens = tokenize("x and y or not z");
        let kinds: Vec<_> = tokens
            .iter()
            .map(|t| &t.token)
            .filter(|t| !matches!(t, Token::Eof))
            .collect();
        assert_eq!(
            kinds,
            vec![
                &Token::Ident("x".into()),
                &Token::And,
                &Token::Ident("y".into()),
                &Token::Or,
                &Token::Not,
                &Token::Ident("z".into()),
            ]
        );
    }

    #[test]
    fn test_string_escape_newline() {
        let tokens = tokenize(r#""hello\nworld""#);
        match &tokens[0].token {
            Token::String(s) => assert_eq!(s, "hello\nworld"),
            other => panic!("expected String, got {:?}", other),
        }
    }

    #[test]
    fn test_string_escape_tab() {
        let tokens = tokenize(r#""col1\tcol2""#);
        match &tokens[0].token {
            Token::String(s) => assert_eq!(s, "col1\tcol2"),
            other => panic!("expected String, got {:?}", other),
        }
    }
}
