use crate::codemap::CodeMap;
use crate::fileio::read_file;
use crate::lexer::{Lexer, Token, TokenKind};

pub struct Parser {
    tokens: Vec<Token>,
    filename: String,
    index: usize,
    line: usize,
    pub map: CodeMap,
    pub output: String,
}

impl Parser {
    pub fn new(filename: String, tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            filename,
            index: 0,
            line: 1,
            map: CodeMap::new(),
            output: String::new(),
        }
    }

    pub fn parse(&mut self) -> Result<(), String> {
        self.map.filenames.push(self.filename.to_owned());
        self.map.add_entry(0, self.line);
        loop {
            match self.parse_single_expr() {
                Ok(Some(())) => {}
                Ok(None) => break,
                Err(e) => return Err(format!(
                    "Error on line {} of {}:\n  {}", 
                    self.line, 
                    self.map.filenames[0],
                    e,
                )),
            }
        }
        Ok(())
    }

    fn parse_single_expr(&mut self) -> Result<Option<()>, String> {
        let tok = match self.peek() {
            Some(t) => t,
            None => return Ok(None),
        };

        match &tok.kind {
            TokenKind::Newline => {
                self.output.push('\n');
                self.line += 1;
                self.next();
                self.map.add_entry(0, self.line);
            }
            TokenKind::Whitespace => {
                self.output.push(' ');
                self.next();
            }
            TokenKind::Code(d) => {
                let d = &d.to_owned();
                self.output.push_str(d);
                self.next();
            }
            TokenKind::Include | TokenKind::Define | TokenKind::Undef => {
                self.parse_directive()?;
            }
            t => return Err(format!("Unexpected token {:#?}", t)),
        }

        Ok(Some(()))
    }

    fn parse_directive(&mut self) -> Result<Option<()>, String> {
        // Get the type of directive, if it's a directive and exists
        let directive = match self.next() {
            Some(t) if matches!(
                t.kind,
                TokenKind::Include | TokenKind::Define | TokenKind::Undef
            ) => {
                t.kind.to_owned()
            },
            Some(t) => return Err(format!("parse_directive() called on {:#?}", t)),
            None => return Err("parse_directive() called on EOF".to_owned()),
        };

        // Get arguments passed to the directive, up until a newline
        let mut params = Vec::<TokenKind>::new();
        loop {
            match self.peek() {
                Some(t) if matches!(
                    t.kind,
                    TokenKind::Integer(_) | TokenKind::String(_) | TokenKind::Code(_)
                ) => {
                    params.push(t.kind.to_owned());
                    self.next();
                }
                _ => break,
            };
        }

        match directive {
            TokenKind::Include => {
                if params.len() != 1 {
                    return Err("#include expects exactly one string parameter".to_owned());
                }
                if let TokenKind::String(path) = &params[0] {
                    let subprogram = read_file(path)?;                          // Read file
                    let mut lexer = Lexer::new(path.to_owned(), subprogram);    // Lex the file
                    lexer.tokenize()?;
                    let mut parser = Self::new(path.to_owned(), lexer.tokens);  // Parse the file
                    parser.parse()?;
                    self.output.push_str(&parser.output);       // Add contents of the other file
                    self.map.push(&parser.map);                 // Add the codemap of the other file
                }
            }
            _ => return Err("Not implemented".to_owned()),
        }

        Ok(Some(()))
    }

    fn peek(&self) -> Option<&Token> {
        if self.index < self.tokens.len() {
            self.tokens.get(self.index)
        } else {
            None
        }
    }

    fn next(&mut self) -> Option<&Token> {
        match self.tokens.get(self.index) {
            Some(t) if self.index != self.tokens.len() => {
                self.index += 1;
                Some(t)
            }
            _ => None,
        }
    }
}
