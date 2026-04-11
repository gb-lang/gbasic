use crate::Parser;
use gbasic_common::ast::*;
use gbasic_common::error::GBasicError;
use gbasic_common::shortcuts::SHORTCUTS;
use gbasic_lexer::Token;

impl Parser {
    /// Attempt to desugar a shortcut call into a MethodChain.
    ///
    /// Given `print("hi")` (already parsed as `Call { callee: Identifier("print"), args }`),
    /// look up "print" in SHORTCUTS and rewrite to `Screen.Layer(0).Print("hi")`.
    ///
    /// Returns `Some(MethodChain)` if the name is a known shortcut, `None` otherwise.
    pub fn desugar_shortcut(
        name: &str,
        args: Vec<Expression>,
        span: gbasic_common::span::Span,
    ) -> Option<Expression> {
        let shortcut = SHORTCUTS.iter().find(|s| s.name == name)?;

        let base = match shortcut.namespace {
            "Screen" => NamespaceRef::Screen,
            "Sound" => NamespaceRef::Sound,
            "Input" => NamespaceRef::Input,
            "Math" => NamespaceRef::Math,
            "System" => NamespaceRef::System,
            "Memory" => NamespaceRef::Memory,
            "IO" => NamespaceRef::IO,
            "Asset" => NamespaceRef::Asset,
            _ => return None,
        };

        // Parse prefix_chain like "Layer(0).Print" or "Random" or "Keyboard.Key"
        // into a Vec<MethodCall>. The last segment gets the user-supplied args;
        // intermediate segments with "(N)" get their literal int arg.
        let mut chain: Vec<MethodCall> = Vec::new();
        let segments: Vec<&str> = shortcut.prefix_chain.split('.').collect();

        for (i, seg) in segments.iter().enumerate() {
            let is_last = i == segments.len() - 1;
            // Check if segment has inline args like "Layer(0)"
            if let Some(paren_pos) = seg.find('(') {
                let method_name = seg[..paren_pos].to_string();
                let inner = &seg[paren_pos + 1..seg.len() - 1]; // strip parens
                let seg_args: Vec<Expression> = if inner.is_empty() {
                    Vec::new()
                } else {
                    // Parse the literal int arg
                    inner
                        .split(',')
                        .filter_map(|s| s.trim().parse::<i64>().ok())
                        .map(|v| {
                            Expression::Literal(Literal {
                                kind: LiteralKind::Int(v),
                                span,
                            })
                        })
                        .collect()
                };
                chain.push(MethodCall {
                    method: Identifier {
                        name: method_name.to_ascii_lowercase(),
                        span,
                    },
                    args: seg_args,
                    span,
                });
            } else {
                // Plain method name — last segment gets user args, others get none
                let seg_args = if is_last { args.clone() } else { Vec::new() };
                chain.push(MethodCall {
                    method: Identifier {
                        name: seg.to_ascii_lowercase(),
                        span,
                    },
                    args: seg_args,
                    span,
                });
            }
        }

        // If the last segment had inline parens (e.g. "Layer(0)"), we need to append
        // a final method call with the user args. This happens when prefix_chain ends
        // with a parenthesized segment like "Effect.Play" — but "Play" has no parens so
        // the user args are already attached above. Only needed if last seg had parens.
        let last_seg = segments.last().unwrap_or(&"");
        if last_seg.contains('(') {
            // Last segment was something like "Layer(0)" — append user args as a new call
            // This case doesn't appear in current shortcuts but handle defensively.
            // Actually for "Effect.Play" the last seg is "Play" (no parens), so args go there.
            // For "Layer(0).Print" the last seg is "Print" (no parens), so args go there.
            // This branch is unreachable with current shortcuts — no action needed.
        }

        Some(Expression::MethodChain { base, chain, span })
    }

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
            Token::Asset => NamespaceRef::Asset,
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
                message: format!("{base} must be followed by a method call, e.g. {base}.Layer(1)",),
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
            chain.push(MethodCall { method, args, span });
        }

        let end_span = chain.last().map(|c| c.span).unwrap_or(start);
        let span = start.merge(end_span);

        Ok(Expression::MethodChain { base, chain, span })
    }
}
