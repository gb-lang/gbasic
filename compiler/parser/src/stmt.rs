use crate::Parser;
use gbasic_common::ast::*;
use gbasic_common::error::GBasicError;
use gbasic_common::types::Type;
use gbasic_lexer::Token;

impl Parser {
    pub fn parse_statement(&mut self) -> Result<Statement, GBasicError> {
        self.skip_newlines();
        match self.current() {
            Token::Let => self.parse_let(),
            Token::Fun | Token::Fn => self.parse_fn(),
            Token::If => self.parse_if(),
            Token::For => self.parse_for(),
            Token::While => self.parse_while(),
            Token::Match => self.parse_match(),
            Token::Return => self.parse_return(),
            Token::Break => {
                let span = self.current_span();
                self.advance();
                self.consume_terminator();
                Ok(Statement::Break { span })
            }
            Token::Continue => {
                let span = self.current_span();
                self.advance();
                self.consume_terminator();
                Ok(Statement::Continue { span })
            }
            Token::LBrace => {
                let block = self.parse_block()?;
                Ok(Statement::Block(block))
            }
            _ => {
                let expr = self.parse_expression()?;
                let span = expr.span();
                self.consume_terminator();
                Ok(Statement::Expression { expr, span })
            }
        }
    }

    fn consume_terminator(&mut self) {
        while matches!(self.current(), Token::Newline | Token::Semicolon) {
            self.advance();
        }
    }

    fn parse_let(&mut self) -> Result<Statement, GBasicError> {
        let start = self.current_span();
        self.advance(); // consume 'let'

        let name = self.parse_identifier()?;

        let type_ann = if matches!(self.current(), Token::Colon) {
            self.advance();
            Some(self.parse_type()?)
        } else {
            None
        };

        self.expect(&Token::Eq)?;
        let value = self.parse_expression()?;
        let span = start.merge(value.span());
        self.consume_terminator();

        Ok(Statement::Let {
            name,
            type_ann,
            value,
            span,
        })
    }

    fn parse_fn(&mut self) -> Result<Statement, GBasicError> {
        let start = self.current_span();
        self.advance(); // consume 'fn'

        let name = self.parse_identifier()?;
        self.expect(&Token::LParen)?;
        let params = self.parse_param_list()?;
        self.expect(&Token::RParen)?;

        let return_type = if matches!(self.current(), Token::Arrow) {
            self.advance();
            Some(self.parse_type()?)
        } else {
            None
        };

        let body = self.parse_block()?;
        let span = start.merge(body.span);

        Ok(Statement::Function(FunctionDecl {
            name,
            params,
            return_type,
            body,
            span,
        }))
    }

    fn parse_if(&mut self) -> Result<Statement, GBasicError> {
        let start = self.current_span();
        self.advance(); // consume 'if'

        let condition = self.parse_expression()?;
        let then_block = self.parse_block()?;

        self.skip_newlines();
        let else_block = if matches!(self.current(), Token::Else) {
            self.advance();
            Some(self.parse_block()?)
        } else {
            None
        };

        let end = else_block
            .as_ref()
            .map(|b| b.span)
            .unwrap_or(then_block.span);
        let span = start.merge(end);

        Ok(Statement::If {
            condition,
            then_block,
            else_block,
            span,
        })
    }

    fn parse_for(&mut self) -> Result<Statement, GBasicError> {
        let start = self.current_span();
        self.advance(); // consume 'for'

        let variable = self.parse_identifier()?;
        self.expect(&Token::In)?;
        let iterable = self.parse_expression()?;
        let body = self.parse_block()?;
        let span = start.merge(body.span);

        Ok(Statement::For {
            variable,
            iterable,
            body,
            span,
        })
    }

    fn parse_while(&mut self) -> Result<Statement, GBasicError> {
        let start = self.current_span();
        self.advance(); // consume 'while'

        let condition = self.parse_expression()?;
        let body = self.parse_block()?;
        let span = start.merge(body.span);

        Ok(Statement::While {
            condition,
            body,
            span,
        })
    }

    fn parse_match(&mut self) -> Result<Statement, GBasicError> {
        let start = self.current_span();
        self.advance(); // consume 'match'

        let subject = self.parse_expression()?;
        self.skip_newlines();
        self.expect(&Token::LBrace)?;

        let mut arms = Vec::new();
        self.skip_newlines();
        while !matches!(self.current(), Token::RBrace | Token::Eof) {
            let pattern = self.parse_pattern()?;
            self.expect(&Token::Arrow)?;
            let body = self.parse_block()?;
            let span = body.span;
            arms.push(MatchArm {
                pattern,
                body,
                span,
            });
            self.skip_newlines();
            // optional comma between arms
            if matches!(self.current(), Token::Comma) {
                self.advance();
            }
            self.skip_newlines();
        }

        let end = self.expect(&Token::RBrace)?;
        let span = start.merge(end);

        Ok(Statement::Match {
            subject,
            arms,
            span,
        })
    }

    fn parse_return(&mut self) -> Result<Statement, GBasicError> {
        let start = self.current_span();
        self.advance(); // consume 'return'

        let value = if matches!(
            self.current(),
            Token::Newline | Token::Semicolon | Token::RBrace | Token::Eof
        ) {
            None
        } else {
            Some(self.parse_expression()?)
        };

        let span = value
            .as_ref()
            .map(|v| start.merge(v.span()))
            .unwrap_or(start);
        self.consume_terminator();

        Ok(Statement::Return { value, span })
    }

    fn parse_pattern(&mut self) -> Result<Pattern, GBasicError> {
        match self.current().clone() {
            Token::Int(v) => {
                let span = self.current_span();
                self.advance();
                Ok(Pattern::Literal(Literal {
                    kind: LiteralKind::Int(v),
                    span,
                }))
            }
            Token::Float(v) => {
                let span = self.current_span();
                self.advance();
                Ok(Pattern::Literal(Literal {
                    kind: LiteralKind::Float(v),
                    span,
                }))
            }
            Token::String(ref s) => {
                let s = s.clone();
                let span = self.current_span();
                self.advance();
                Ok(Pattern::Literal(Literal {
                    kind: LiteralKind::String(s),
                    span,
                }))
            }
            Token::True => {
                let span = self.current_span();
                self.advance();
                Ok(Pattern::Literal(Literal {
                    kind: LiteralKind::Bool(true),
                    span,
                }))
            }
            Token::False => {
                let span = self.current_span();
                self.advance();
                Ok(Pattern::Literal(Literal {
                    kind: LiteralKind::Bool(false),
                    span,
                }))
            }
            Token::Ident(ref name) if name == "_" => {
                let span = self.current_span();
                self.advance();
                Ok(Pattern::Wildcard(span))
            }
            Token::Ident(ref name) => {
                let name = name.clone();
                let span = self.current_span();
                self.advance();
                Ok(Pattern::Identifier(Identifier { name, span }))
            }
            _ => Err(GBasicError::SyntaxError {
                message: format!("expected pattern, found '{}'", self.current()),
                span: self.current_span(),
            }),
        }
    }

    pub fn parse_block(&mut self) -> Result<Block, GBasicError> {
        self.skip_newlines();
        let start = self.expect(&Token::LBrace)?;
        let mut statements = Vec::new();
        self.skip_newlines();

        while !matches!(self.current(), Token::RBrace | Token::Eof) {
            match self.parse_statement() {
                Ok(stmt) => statements.push(stmt),
                Err(e) => {
                    self.error(e);
                    self.synchronize();
                }
            }
            self.skip_newlines();
        }

        let end = self.expect(&Token::RBrace)?;
        Ok(Block {
            statements,
            span: start.merge(end),
        })
    }

    pub fn parse_identifier(&mut self) -> Result<Identifier, GBasicError> {
        if let Token::Ident(ref name) = self.current().clone() {
            let name = name.clone();
            let span = self.current_span();
            self.advance();
            Ok(Identifier { name, span })
        } else {
            Err(GBasicError::SyntaxError {
                message: format!("expected identifier, found '{}'", self.current()),
                span: self.current_span(),
            })
        }
    }

    fn parse_param_list(&mut self) -> Result<Vec<Parameter>, GBasicError> {
        let mut params = Vec::new();
        if !matches!(self.current(), Token::RParen) {
            params.push(self.parse_param()?);
            while matches!(self.current(), Token::Comma) {
                self.advance();
                params.push(self.parse_param()?);
            }
        }
        Ok(params)
    }

    fn parse_param(&mut self) -> Result<Parameter, GBasicError> {
        let name = self.parse_identifier()?;
        let type_ann = if matches!(self.current(), Token::Colon) {
            self.advance();
            Some(self.parse_type()?)
        } else {
            None
        };
        let span = name.span.merge(self.tokens[self.pos - 1].span);
        Ok(Parameter {
            name,
            type_ann,
            span,
        })
    }

    pub fn parse_type(&mut self) -> Result<Type, GBasicError> {
        match self.current() {
            Token::TyInt => {
                self.advance();
                Ok(Type::Int)
            }
            Token::TyFloat => {
                self.advance();
                Ok(Type::Float)
            }
            Token::TyString => {
                self.advance();
                Ok(Type::String)
            }
            Token::TyBool => {
                self.advance();
                Ok(Type::Bool)
            }
            Token::TyVoid => {
                self.advance();
                Ok(Type::Void)
            }
            Token::LBracket => {
                self.advance();
                let inner = self.parse_type()?;
                self.expect(&Token::RBracket)?;
                Ok(Type::Array(Box::new(inner)))
            }
            _ => Err(GBasicError::SyntaxError {
                message: format!("expected type, found '{}'", self.current()),
                span: self.current_span(),
            }),
        }
    }
}
