use crate::Parser;
use gbasic_common::ast::*;
use gbasic_common::error::GBasicError;
use gbasic_lexer::Token;

impl Parser {
    /// Parse a namespace method chain: `Screen.Layer(1).Sprite("hero").Draw()`
    pub fn parse_method_chain(&mut self) -> Result<Expression, GBasicError> {
        let start = self.current_span();

        let base = match self.current() {
            Token::Screen => NamespaceRef::Screen,
            Token::Sound => NamespaceRef::Sound,
            Token::Input => NamespaceRef::Input,
            Token::Math => NamespaceRef::Math,
            Token::System => NamespaceRef::System,
            Token::Memory => NamespaceRef::Memory,
            Token::IO => NamespaceRef::IO,
            _ => {
                return Err(GBasicError::SyntaxError {
                    message: format!("expected namespace, found '{}'", self.current()),
                    span: self.current_span(),
                });
            }
        };
        self.advance();

        let mut chain = Vec::new();

        // Expect at least one .Method(args) call
        if !matches!(self.current(), Token::Dot) {
            return Err(GBasicError::SyntaxError {
                message: format!(
                    "{base} must be followed by a method call, e.g. {base}.Layer(1)",
                ),
                span: self.current_span(),
            });
        }

        while matches!(self.current(), Token::Dot) {
            self.advance(); // consume '.'
            let method = self.parse_identifier()?;

            // Allow both Method(args) and Field (no parens, treated as zero-arg call)
            let (args, end) = if matches!(self.current(), Token::LParen) {
                self.advance();
                let args = self.parse_arg_list()?;
                let end = self.expect(&Token::RParen)?;
                (args, end)
            } else {
                (Vec::new(), method.span)
            };

            let span = method.span.merge(end);
            chain.push(MethodCall {
                method,
                args,
                span,
            });
        }

        let end_span = chain.last().map(|c| c.span).unwrap_or(start);
        let span = start.merge(end_span);

        Ok(Expression::MethodChain { base, chain, span })
    }
}
