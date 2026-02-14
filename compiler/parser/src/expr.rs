use crate::Parser;
use gbasic_common::ast::*;
use gbasic_common::error::GBasicError;
use gbasic_common::span::Span;
use gbasic_lexer::Token;

/// Define a left-associative binary operator parser function.
macro_rules! define_binop_parser {
    ($name:ident, $next:ident, $( $pat:pat => $op:expr ),+ $(,)?) => {
        fn $name(&mut self) -> Result<Expression, GBasicError> {
            let mut left = self.$next()?;
            while matches!(self.current(), $( $pat )|+) {
                let op = match self.current() {
                    $( $pat => $op, )+
                    _ => unreachable!(),
                };
                self.advance();
                let right = self.$next()?;
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
    };
}

impl Parser {
    /// Parse an expression using Pratt/precedence-climbing.
    pub fn parse_expression(&mut self) -> Result<Expression, GBasicError> {
        self.parse_assignment()
    }

    fn parse_assignment(&mut self) -> Result<Expression, GBasicError> {
        let expr = self.parse_or()?;

        if matches!(self.current(), Token::DotDot) {
            self.advance();
            let end = self.parse_or()?;
            let span = expr.span().merge(end.span());
            return Ok(Expression::Range {
                start: Box::new(expr),
                end: Box::new(end),
                span,
            });
        }

        // `a to b` → inclusive range, desugars to `a..(b+1)`
        if matches!(self.current(), Token::Ident(s) if s == "to") {
            self.advance();
            let end_expr = self.parse_or()?;
            let end_span = end_expr.span();
            let span = expr.span().merge(end_span);
            // Synthesize end + 1
            let end_plus_one = Expression::BinaryOp {
                left: Box::new(end_expr),
                op: BinaryOp::Add,
                right: Box::new(Expression::Literal(Literal {
                    kind: LiteralKind::Int(1),
                    span: end_span,
                })),
                span: end_span,
            };
            return Ok(Expression::Range {
                start: Box::new(expr),
                end: Box::new(end_plus_one),
                span,
            });
        }

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

    define_binop_parser!(parse_or, parse_and,
        Token::PipePipe | Token::Or => BinaryOp::Or
    );

    define_binop_parser!(parse_and, parse_equality,
        Token::AmpAmp | Token::And => BinaryOp::And
    );

    define_binop_parser!(parse_equality, parse_comparison,
        Token::EqEq => BinaryOp::Eq,
        Token::BangEq => BinaryOp::Neq
    );

    define_binop_parser!(parse_comparison, parse_additive,
        Token::Lt => BinaryOp::Lt,
        Token::Gt => BinaryOp::Gt,
        Token::LtEq => BinaryOp::Le,
        Token::GtEq => BinaryOp::Ge
    );

    define_binop_parser!(parse_additive, parse_multiplicative,
        Token::Plus => BinaryOp::Add,
        Token::Minus => BinaryOp::Sub
    );

    define_binop_parser!(parse_multiplicative, parse_unary,
        Token::Star => BinaryOp::Mul,
        Token::Slash => BinaryOp::Div,
        Token::Percent => BinaryOp::Mod
    );

    fn parse_unary(&mut self) -> Result<Expression, GBasicError> {
        match self.current() {
            Token::Bang | Token::Not => {
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
            | Token::Memory | Token::IO | Token::Asset => {
                self.parse_method_chain()
            }
            Token::Ident(ref name) => {
                let name = name.clone();
                let span = self.current_span();
                self.advance();
                Ok(Expression::Identifier(Identifier { name, span }))
            }
            Token::LParen => {
                let start = self.current_span();
                self.advance();
                let expr = self.parse_expression()?;
                // Tuple syntax: (x, y) → Point(x, y), (r, g, b) → Color(r, g, b)
                if matches!(self.current(), Token::Comma) {
                    let mut args = vec![expr];
                    while matches!(self.current(), Token::Comma) {
                        self.advance();
                        args.push(self.parse_expression()?);
                    }
                    let end = self.expect(&Token::RParen)?;
                    let span = start.merge(end);
                    let name = if args.len() == 2 { "point" } else { "color" };
                    return Ok(Expression::Call {
                        callee: Box::new(Expression::Identifier(Identifier {
                            name: name.to_string(),
                            span: start,
                        })),
                        args,
                        span,
                    });
                }
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
