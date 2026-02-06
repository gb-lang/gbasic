use crate::Parser;
use gbasic_common::ast::*;
use gbasic_common::error::GBasicError;
use gbasic_common::span::Span;
use gbasic_lexer::Token;

impl Parser {
    /// Parse an expression using Pratt/precedence-climbing.
    pub fn parse_expression(&mut self) -> Result<Expression, GBasicError> {
        self.parse_assignment()
    }

    fn parse_assignment(&mut self) -> Result<Expression, GBasicError> {
        let expr = self.parse_or()?;

        if matches!(self.current(), Token::Eq) {
            self.advance();
            let value = self.parse_assignment()?;
            let span = expr.span().merge(value.span());
            return Ok(Expression::Assignment {
                target: Box::new(expr),
                value: Box::new(value),
                span,
            });
        }

        Ok(expr)
    }

    fn parse_or(&mut self) -> Result<Expression, GBasicError> {
        let mut left = self.parse_and()?;
        while matches!(self.current(), Token::PipePipe) {
            self.advance();
            let right = self.parse_and()?;
            let span = left.span().merge(right.span());
            left = Expression::BinaryOp {
                left: Box::new(left),
                op: BinaryOp::Or,
                right: Box::new(right),
                span,
            };
        }
        Ok(left)
    }

    fn parse_and(&mut self) -> Result<Expression, GBasicError> {
        let mut left = self.parse_equality()?;
        while matches!(self.current(), Token::AmpAmp) {
            self.advance();
            let right = self.parse_equality()?;
            let span = left.span().merge(right.span());
            left = Expression::BinaryOp {
                left: Box::new(left),
                op: BinaryOp::And,
                right: Box::new(right),
                span,
            };
        }
        Ok(left)
    }

    fn parse_equality(&mut self) -> Result<Expression, GBasicError> {
        let mut left = self.parse_comparison()?;
        while matches!(self.current(), Token::EqEq | Token::BangEq) {
            let op = if matches!(self.current(), Token::EqEq) {
                BinaryOp::Eq
            } else {
                BinaryOp::Neq
            };
            self.advance();
            let right = self.parse_comparison()?;
            let span = left.span().merge(right.span());
            left = Expression::BinaryOp {
                left: Box::new(left),
                op,
                right: Box::new(right),
                span,
            };
        }
        Ok(left)
    }

    fn parse_comparison(&mut self) -> Result<Expression, GBasicError> {
        let mut left = self.parse_additive()?;
        while matches!(
            self.current(),
            Token::Lt | Token::Gt | Token::LtEq | Token::GtEq
        ) {
            let op = match self.current() {
                Token::Lt => BinaryOp::Lt,
                Token::Gt => BinaryOp::Gt,
                Token::LtEq => BinaryOp::Le,
                Token::GtEq => BinaryOp::Ge,
                _ => unreachable!(),
            };
            self.advance();
            let right = self.parse_additive()?;
            let span = left.span().merge(right.span());
            left = Expression::BinaryOp {
                left: Box::new(left),
                op,
                right: Box::new(right),
                span,
            };
        }
        Ok(left)
    }

    fn parse_additive(&mut self) -> Result<Expression, GBasicError> {
        let mut left = self.parse_multiplicative()?;
        while matches!(self.current(), Token::Plus | Token::Minus) {
            let op = if matches!(self.current(), Token::Plus) {
                BinaryOp::Add
            } else {
                BinaryOp::Sub
            };
            self.advance();
            let right = self.parse_multiplicative()?;
            let span = left.span().merge(right.span());
            left = Expression::BinaryOp {
                left: Box::new(left),
                op,
                right: Box::new(right),
                span,
            };
        }
        Ok(left)
    }

    fn parse_multiplicative(&mut self) -> Result<Expression, GBasicError> {
        let mut left = self.parse_unary()?;
        while matches!(self.current(), Token::Star | Token::Slash | Token::Percent) {
            let op = match self.current() {
                Token::Star => BinaryOp::Mul,
                Token::Slash => BinaryOp::Div,
                Token::Percent => BinaryOp::Mod,
                _ => unreachable!(),
            };
            self.advance();
            let right = self.parse_unary()?;
            let span = left.span().merge(right.span());
            left = Expression::BinaryOp {
                left: Box::new(left),
                op,
                right: Box::new(right),
                span,
            };
        }
        Ok(left)
    }

    fn parse_unary(&mut self) -> Result<Expression, GBasicError> {
        match self.current() {
            Token::Bang => {
                let start = self.current_span();
                self.advance();
                let operand = self.parse_unary()?;
                let span = start.merge(operand.span());
                Ok(Expression::UnaryOp {
                    op: UnaryOp::Not,
                    operand: Box::new(operand),
                    span,
                })
            }
            Token::Minus => {
                let start = self.current_span();
                self.advance();
                let operand = self.parse_unary()?;
                let span = start.merge(operand.span());
                Ok(Expression::UnaryOp {
                    op: UnaryOp::Neg,
                    operand: Box::new(operand),
                    span,
                })
            }
            _ => self.parse_postfix(),
        }
    }

    fn parse_postfix(&mut self) -> Result<Expression, GBasicError> {
        let mut expr = self.parse_primary()?;

        loop {
            match self.current() {
                Token::LParen => {
                    self.advance();
                    let args = self.parse_arg_list()?;
                    let end = self.expect(&Token::RParen)?;
                    let span = expr.span().merge(end);
                    expr = Expression::Call {
                        callee: Box::new(expr),
                        args,
                        span,
                    };
                }
                Token::LBracket => {
                    self.advance();
                    let index = self.parse_expression()?;
                    let end = self.expect(&Token::RBracket)?;
                    let span = expr.span().merge(end);
                    expr = Expression::Index {
                        object: Box::new(expr),
                        index: Box::new(index),
                        span,
                    };
                }
                Token::Dot => {
                    self.advance();
                    if let Token::Ident(name) = self.current().clone() {
                        let field_span = self.current_span();
                        self.advance();
                        let span = expr.span().merge(field_span);
                        expr = Expression::FieldAccess {
                            object: Box::new(expr),
                            field: Identifier {
                                name,
                                span: field_span,
                            },
                            span,
                        };
                    } else {
                        return Err(GBasicError::SyntaxError {
                            message: format!(
                                "expected field name after '.', found '{}'",
                                self.current()
                            ),
                            span: self.current_span(),
                        });
                    }
                }
                _ => break,
            }
        }

        Ok(expr)
    }

    fn parse_primary(&mut self) -> Result<Expression, GBasicError> {
        match self.current().clone() {
            Token::Int(v) => {
                let span = self.current_span();
                self.advance();
                Ok(Expression::Literal(Literal {
                    kind: LiteralKind::Int(v),
                    span,
                }))
            }
            Token::Float(v) => {
                let span = self.current_span();
                self.advance();
                Ok(Expression::Literal(Literal {
                    kind: LiteralKind::Float(v),
                    span,
                }))
            }
            Token::String(ref s) => {
                let s = s.clone();
                let span = self.current_span();
                self.advance();
                if s.contains('{') {
                    self.parse_string_interp(&s, span)
                } else {
                    Ok(Expression::Literal(Literal {
                        kind: LiteralKind::String(s),
                        span,
                    }))
                }
            }
            Token::True => {
                let span = self.current_span();
                self.advance();
                Ok(Expression::Literal(Literal {
                    kind: LiteralKind::Bool(true),
                    span,
                }))
            }
            Token::False => {
                let span = self.current_span();
                self.advance();
                Ok(Expression::Literal(Literal {
                    kind: LiteralKind::Bool(false),
                    span,
                }))
            }
            Token::Screen | Token::Sound | Token::Input | Token::Math | Token::System
            | Token::Memory | Token::IO => {
                self.parse_method_chain()
            }
            Token::Ident(ref name) => {
                let name = name.clone();
                let span = self.current_span();
                self.advance();
                Ok(Expression::Identifier(Identifier { name, span }))
            }
            Token::LParen => {
                self.advance();
                let expr = self.parse_expression()?;
                self.expect(&Token::RParen)?;
                Ok(expr)
            }
            Token::LBracket => {
                let start = self.current_span();
                self.advance();
                let elements = self.parse_arg_list()?;
                let end = self.expect(&Token::RBracket)?;
                Ok(Expression::Array {
                    elements,
                    span: start.merge(end),
                })
            }
            _ => Err(GBasicError::SyntaxError {
                message: format!("unexpected token '{}'", self.current()),
                span: self.current_span(),
            }),
        }
    }

    pub fn parse_arg_list(&mut self) -> Result<Vec<Expression>, GBasicError> {
        let mut args = Vec::new();
        if !matches!(self.current(), Token::RParen | Token::RBracket) {
            args.push(self.parse_expression()?);
            while matches!(self.current(), Token::Comma) {
                self.advance();
                args.push(self.parse_expression()?);
            }
        }
        Ok(args)
    }

    /// Parse a string with `{expr}` interpolation into StringInterp parts.
    fn parse_string_interp(&mut self, s: &str, span: Span) -> Result<Expression, GBasicError> {
        let mut parts = Vec::new();
        let mut current = String::new();
        let mut chars = s.chars().peekable();

        while let Some(ch) = chars.next() {
            if ch == '{' {
                // Collect the expression text until matching '}'
                if !current.is_empty() {
                    parts.push(StringPart::Lit(std::mem::take(&mut current)));
                }
                let mut expr_text = String::new();
                let mut depth = 1;
                for ch in chars.by_ref() {
                    if ch == '{' {
                        depth += 1;
                        expr_text.push(ch);
                    } else if ch == '}' {
                        depth -= 1;
                        if depth == 0 {
                            break;
                        }
                        expr_text.push(ch);
                    } else {
                        expr_text.push(ch);
                    }
                }
                if depth != 0 {
                    return Err(GBasicError::SyntaxError {
                        message: "unclosed '{' in string interpolation".to_string(),
                        span,
                    });
                }
                // Parse the expression text as a sub-expression
                let tokens = gbasic_lexer::tokenize(&expr_text);
                let mut sub_parser = Parser::new(tokens);
                let expr = sub_parser.parse_expression().map_err(|_| GBasicError::SyntaxError {
                    message: format!("invalid expression in string interpolation: {{{expr_text}}}"),
                    span,
                })?;
                parts.push(StringPart::Expr(expr));
            } else {
                current.push(ch);
            }
        }

        if !current.is_empty() {
            parts.push(StringPart::Lit(current));
        }

        Ok(Expression::StringInterp { parts, span })
    }
}
