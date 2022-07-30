use super::lexer::{Token, TokenKind};
use std::fmt;

#[derive(Debug)]
pub struct Parser {
    tokens: Vec<Token>,
    index: usize,
    line: usize,
}

#[derive(Debug, Clone)]
pub enum ExprKind {
    Instruction(TokenKind), // Conditions only
    Op(TokenKind),          // Instructions only
    String(String),
    Reference(bool),     // is indexed?
    Register(TokenKind), // Registers only
    Directive(TokenKind),

    Expression,
    Term,
    Factor,
    Unary,
    Primary,

    Integer(u16),
    Label(String),

    Operator(TokenKind),
}

#[derive(Clone)]
pub struct Expr {
    pub kind: ExprKind,
    pub exprs: Vec<Expr>,
    pub line: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            index: 0,
            line: 1,
        }
    }

    fn peek(&self) -> Option<&Token> {
        if self.index != self.tokens.len() {
            self.tokens.get(self.index)
        } else {
            None
        }
    }

    fn next(&mut self) -> Option<&Token> {
        match self.tokens.get(self.index) {
            Some(t) if self.index != self.tokens.len() => {
                self.index += 1;
                self.line = t.line;
                Some(t)
            }
            _ => None,
        }
    }

    /*
     *[X] statement   = instruction | directive | label
     *
     *[X] instruction = op | op "?" CONDITION
     *[/] op          = OPCODE | OPCODE (hardware | expression) | OPCODE (hardware | expression), (hardware | expression)
     *[ ] hardware    = REGISTER | \(REGISTER | expression\ (+IX)?) | \(\(REGISTER | expression\ (+IX)?)\ (+IX)?)
     *[/] expression  = term
     *[ ] term        = factor (("+" | "-"") factor)*
     *[ ] factor      = unary (("-" | "+") unary)*
     *[ ] unary       = ("+" | "-" | "~") unary
     *                  | primary
     *[ ] primary     = INTEGER | LABEL
     *
     *[X] directive   = DIRECTIVE (expression | BYTE | STRING)*
     *
     *[X] label       = LABEL ":"
     */

    pub fn parse(&mut self) -> Result<Vec<Expr>, (String, usize)> {
        let mut output: Vec<Expr> = Vec::new();

        loop {
            match self.parse_one_statement() {
                Ok(Some(statement)) => output.push(statement),
                Ok(None) => break,
                Err(e) => return Err((e, self.line)),
            };
        }

        Ok(output)
    }

    pub fn parse_one_statement(&mut self) -> Result<Option<Expr>, String> {
        if let Some(i) = self.instruction()? {
            Ok(Some(i))
        } else if let Some(d) = self.directive()? {
            Ok(Some(d))
        } else if let Some(l) = self.label()? {
            Ok(Some(l))
        } else if let Some(t) = self.peek() {
            Err(format!("Unexpected token '{}'", t))
        } else {
            Ok(None)
        }
    }

    fn instruction(&mut self) -> Result<Option<Expr>, String> {
        // Peek for next token
        let peek = match self.peek() {
            Some(t) => t,
            None => return Ok(None),
        };

        // Check if there's a '-' for the conditional, and if so, get the condition
        let kind = if matches!(peek.kind, TokenKind::Minus) {
            self.next(); // Consume the '-'

            // Get the next word for the condition if it exists
            let peek = match self.peek() {
                Some(t) => t,
                None => return Err(format!("Expected condition after '-', found EOF")),
            };

            // Check that the peeked token is in fact a condition, and if so, set that to op's cond
            let kind = match &peek.kind {
                TokenKind::C
                | TokenKind::Z
                | TokenKind::Nc
                | TokenKind::Nz
                | TokenKind::Cz
                | TokenKind::Ncz => ExprKind::Instruction(peek.kind.to_owned()),
                t => return Err(format!("Expected condition after '-', found {:?}", t)),
            };

            // Consume conditional token
            self.next();
            kind
        } else {
            ExprKind::Instruction(TokenKind::None)
        };

        // Ensure that there's an operation to be read in
        let op = match self.op()? {
            Some(op) => op,
            None => return Ok(None),
        };

        let line = op.line;
        Ok(Some(Expr {
            kind,
            exprs: vec![op],
            line,
        }))
    }

    fn op(&mut self) -> Result<Option<Expr>, String> {
        let op_token = match self.peek() {
            Some(t) => t,
            None => return Ok(None),
        };

        let mut op = match op_token.kind {
            TokenKind::Mov
            | TokenKind::Add
            | TokenKind::Adc
            | TokenKind::Sub
            | TokenKind::Sbb
            | TokenKind::Sbw
            | TokenKind::Swb
            | TokenKind::Nnd
            | TokenKind::And
            | TokenKind::Aib
            | TokenKind::Anb
            | TokenKind::Bia
            | TokenKind::Bna
            | TokenKind::Ora
            | TokenKind::Nor
            | TokenKind::Jmp
            | TokenKind::Hlt
            | TokenKind::Jsr
            | TokenKind::Ret
            | TokenKind::Dec
            | TokenKind::Inc
            | TokenKind::Cmp
            | TokenKind::Xor
            | TokenKind::Xnr
            | TokenKind::Clc
            | TokenKind::Clz
            | TokenKind::Sec
            | TokenKind::Sez => Expr {
                kind: ExprKind::Op(op_token.kind.to_owned()),
                exprs: vec![],
                line: op_token.line,
            },
            _ => return Ok(None),
        };

        self.next();

        match self.hardware_or_expression()? {
            Some(val) => op.exprs.push(val),
            None => return Ok(Some(op)),
        }

        // Check for a comma
        match self.peek() {
            Some(t) if matches!(t.kind, TokenKind::Comma) => (),
            _ => return Ok(Some(op)),
        };

        self.next();

        match self.hardware_or_expression()? {
            Some(val) => op.exprs.push(val),
            None => return Err("No 2nd parameter after ','".to_owned()),
        }

        Ok(Some(op))
    }

    fn hardware_or_expression(&mut self) -> Result<Option<Expr>, String> {
        if let Some(expr) = self.hardware()? {
            Ok(Some(expr))
        } else if let Some(expr) = self.expression()? {
            Ok(Some(expr))
        } else {
            Ok(None)
        }
    }

    fn hardware(&mut self) -> Result<Option<Expr>, String> {
        if let Some(expr) = self.register()? {
            return Ok(Some(expr));
        }

        if let Some(expr) = self.expression()? {
            return Ok(Some(expr));
        }

        // Check for an open parentheses to see if it's a reference instead of a value
        let (tk, line) = match self.peek() {
            Some(t) => (t.kind.to_owned(), t.line),
            None => return Ok(None),
        };

        if !matches!(tk, TokenKind::OpenParen) {
            return Ok(None);
        }
        self.next();

        let contents = match self.hardware_or_expression()? {
            Some(e) => e,
            None => return Err("Expected expression or register, found EOF".to_owned()),
        };

        // so many checks.....
        let is_indexed = match self.next() {
            Some(t) if matches!(t.kind, TokenKind::CloseParen) => false,
            Some(t) if matches!(t.kind, TokenKind::Plus) => {
                match self.next() {
                    Some(t) if matches!(t.kind, TokenKind::Ix) => {
                        match self.next() {
                            Some(t) if matches!(t.kind, TokenKind::CloseParen) => true,
                            Some(t) => return Err(format!("Expected ')', found {t}")),
                            None => return Err("Expected ')', found EOF".to_owned()),
                        }
                    },
                    Some(t) => return Err(format!("Expected IX after +, found {t} (this is probably an implementation error, my bad)")),
                    None => return Err("Expected IX after +, found EOF (this is probably an implementation error, my bad)".to_owned()),
                }
            }
            Some(t) => return Err(format!("Expected ')' or `+IX`, found {t}")),
            None => return Err("Expected ')' or '`+IX`', found EOF".to_owned()),
        };

        Ok(Some(Expr {
            kind: ExprKind::Reference(is_indexed),
            exprs: vec![contents],
            line,
        }))
    }

    fn register(&mut self) -> Result<Option<Expr>, String> {
        let (tk, line) = match self.peek() {
            Some(t) => (t.kind.to_owned(), t.line),
            None => return Ok(None),
        };

        // Check if there's a register token
        match tk {
            TokenKind::Ac
            | TokenKind::Br
            | TokenKind::Ix
            | TokenKind::Sp
            | TokenKind::Imm
            | TokenKind::Stack => {
                self.next();
                Ok(Some(Expr {
                    kind: ExprKind::Register(tk),
                    exprs: vec![],
                    line,
                }))
            }
            _ => Ok(None),
        }
    }

    fn expression(&mut self) -> Result<Option<Expr>, String> {
        let mut expr = Expr {
            kind: ExprKind::Expression,
            exprs: vec![],
            line: match self.peek() {
                Some(t) => t.line,
                None => return Ok(None),
            },
        };

        match self.term()? {
            Some(e) => expr.exprs.push(e),
            None => return Ok(None),
        };

        Ok(Some(expr))
    }

    fn term(&mut self) -> Result<Option<Expr>, String> {
        let mut expr = Expr {
            kind: ExprKind::Term,
            exprs: vec![],
            line: match self.peek() {
                Some(t) => t.line,
                None => return Ok(None),
            },
        };

        match self.factor()? {
            Some(e) => expr.exprs.push(e),
            None => return Ok(None),
        };

        loop {
            match self.peek() {
                Some(t) if matches!(t.kind, TokenKind::Plus | TokenKind::Minus) => {
                    expr.exprs.push(Expr {
                        kind: ExprKind::Operator(t.kind.to_owned()),
                        exprs: vec![],
                        line: t.line,
                    });
                }
                _ => break,
            }
            self.next();

            if matches!(
                self.peek(),
                Some(Token {
                    kind: TokenKind::Ix,
                    ..
                })
            ) {
                self.index -= 1;
                expr.exprs.pop();
                break;
            }

            match self.factor()? {
                Some(e) => expr.exprs.push(e),
                None => return Err("Expected value after + or - operator".to_owned()),
            }
        }

        Ok(Some(expr))
    }

    fn factor(&mut self) -> Result<Option<Expr>, String> {
        let mut expr = Expr {
            kind: ExprKind::Factor,
            exprs: vec![],
            line: match self.peek() {
                Some(t) => t.line,
                None => return Ok(None),
            },
        };

        match self.unary()? {
            Some(e) => expr.exprs.push(e),
            None => return Ok(None),
        };

        loop {
            match self.peek() {
                Some(t) if matches!(t.kind, TokenKind::Times | TokenKind::Div) => {
                    expr.exprs.push(Expr {
                        kind: ExprKind::Operator(t.kind.to_owned()),
                        exprs: vec![],
                        line: t.line,
                    });
                }
                _ => break,
            }
            self.next();
            match self.unary()? {
                Some(e) => expr.exprs.push(e),
                None => return Err("Expected value after * or / operator".to_owned()),
            }
        }

        Ok(Some(expr))
    }

    fn unary(&mut self) -> Result<Option<Expr>, String> {
        let mut expr = Expr {
            kind: ExprKind::Unary,
            exprs: vec![],
            line: match self.peek() {
                Some(t) => t.line,
                None => return Ok(None),
            },
        };

        match self.peek() {
            Some(t) if matches!(t.kind, TokenKind::Plus | TokenKind::Minus) => {
                expr.exprs.push(Expr {
                    kind: ExprKind::Operator(t.kind.to_owned()),
                    exprs: vec![],
                    line: t.line,
                });
                self.next();
                match self.unary()? {
                    Some(e) => expr.exprs.push(e),
                    None => return Err("Expected value after unary +, -, or ~ operator".to_owned()),
                }
            }
            _ => match self.primary()? {
                Some(e) => expr.exprs.push(e),
                None => return Ok(None),
            },
        };

        Ok(Some(expr))
    }

    fn primary(&mut self) -> Result<Option<Expr>, String> {
        let mut expr = Expr {
            kind: ExprKind::Primary,
            exprs: vec![],
            line: match self.peek() {
                Some(t) => t.line,
                None => return Ok(None),
            },
        };

        match self.peek() {
            Some(Token {
                kind: TokenKind::Integer(n),
                span: _,
                line,
            }) => {
                expr.exprs.push(Expr {
                    kind: ExprKind::Integer(*n),
                    exprs: vec![],
                    line: *line,
                });
                self.next();
            }
            Some(Token {
                kind: TokenKind::Label(_),
                ..
            }) => {
                let (kind, line) = match self.peek() {
                    Some(Token {
                        kind: TokenKind::Label(l),
                        span: _,
                        line,
                    }) => (ExprKind::Label(l.to_owned()), *line),
                    _ => return Ok(None),
                };

                self.next();

                match self.peek() {
                    Some(Token {
                        kind: TokenKind::Colon,
                        ..
                    }) => {
                        self.index -= 1;
                        return Ok(None);
                    }
                    _ => (),
                };

                expr.exprs.push(Expr {
                    kind,
                    exprs: vec![],
                    line,
                });
            }
            _ => return Ok(None),
        }

        Ok(Some(expr))
    }

    fn directive(&mut self) -> Result<Option<Expr>, String> {
        let directive_token = match self.peek() {
            Some(t) => t,
            None => return Ok(None),
        };

        let kind = match &directive_token.kind {
            TokenKind::Org | TokenKind::Db => &directive_token.kind,
            _ => return Ok(None),
        };

        let mut directive = Expr {
            kind: ExprKind::Directive(kind.to_owned()),
            exprs: vec![],
            line: directive_token.line,
        };

        self.next();

        loop {
            if let Some(expr) = self.expression()? {
                directive.exprs.push(expr);
            };

            match self.peek() {
                Some(Token {
                    kind: TokenKind::String(s),
                    span: _,
                    line,
                }) => {
                    directive.exprs.push(Expr {
                        kind: ExprKind::String(s.to_owned()),
                        exprs: vec![],
                        line: *line,
                    });
                    self.next();
                }
                _ => break,
            };
        }

        Ok(Some(directive))
    }

    fn label(&mut self) -> Result<Option<Expr>, String> {
        let (label_token_kind, line) = match self.peek() {
            Some(t) => (&t.kind, t.line),
            None => return Ok(None),
        };

        let kind = match label_token_kind {
            TokenKind::Label(n) => ExprKind::Label(n.to_owned()),
            _ => return Ok(None),
        };

        self.next();

        let colon_token = match self.peek() {
            Some(t) => t.to_owned(),
            None => return Ok(None),
        };

        match &colon_token.kind {
            TokenKind::Colon => (),
            _ => return Err(format!("Unknown label {:?}", kind)),
        };

        self.next();

        Ok(Some(Expr {
            kind,
            exprs: vec![],
            line,
        }))
    }
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?} [", self.kind)?;
        for expr in &self.exprs {
            write!(f, " {}", expr)?;
        }
        write!(f, " ]")?;
        fmt::Result::Ok(())
    }
}

impl fmt::Debug for Expr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.kind)?;
        for expr in &self.exprs {
            write!(f, " {}", expr)?;
        }
        fmt::Result::Ok(())
    }
}
