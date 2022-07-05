use crate::codemap::CodeMap;
use crate::fileio::read_file;
use crate::lexer::{Lexer, Token, TokenKind};
use std::collections::HashMap;

pub struct Parser<'a> {
    tokens: &'a [Token],
    pub output: String,
    pub map: CodeMap,
    pub deflist: HashMap<String, &'a [Token]>,
    index: usize,
    line: usize,
    filename: String,
}

impl<'a> Parser<'a> {
    pub fn new(filename: &str, tokens: &'a [Token]) -> Self {
        Self {
            tokens,
            output: String::new(),
            map: CodeMap::new(),
            deflist: HashMap::new(),
            index: 0,
            line: 1,
            filename: filename.to_owned(),
        }
    }

    pub fn parse(&mut self) -> Result<(), String> {
        self.map.filenames.push(self.filename.to_owned());
        self.map.add_entry(0, self.line);
        loop {
            match self.parse_single_expr() {
                Ok(Some(())) => {}
                Ok(None) => break,
                Err(e) => {
                    return Err(format!(
                        "\x1b[91mError on line {} of {}:\x1b[0m\n  {}",
                        self.line, self.map.filenames[0], e,
                    ))
                }
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
                self.consume_whitespace();
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
            TokenKind::String(d) => {
                let d = &d.to_owned();
                self.output.push('"');
                self.output.push_str(d);
                self.output.push('"');
                self.next();
            }
            TokenKind::Include | TokenKind::Define | TokenKind::Undef => {
                self.parse_directive()?;
            }
            TokenKind::None => {}
        }

        Ok(Some(()))
    }

    fn parse_directive(&mut self) -> Result<Option<()>, String> {
        // Get the type of directive, if it's a directive and exists
        let directive = match self.next() {
            Some(t)
                if matches!(
                    t.kind,
                    TokenKind::Include | TokenKind::Define | TokenKind::Undef
                ) =>
            {
                t.kind.to_owned()
            }
            Some(t) => return Err(format!("parse_directive() called on {:?}", t)),
            None => return Err("parse_directive() called on EOF".to_owned()),
        };

        self.consume_whitespace(); // Ignore whitespace if it's there

        // Get arguments passed to the directive, up until a newline
        let mut param_span = (self.index, self.index);
        loop {
            match self.peek() {
                Some(t) if !matches!(t.kind, TokenKind::Newline) => {
                    param_span.1 += 1;
                    self.next();
                }
                _ => break,
            };
        }

        match directive {
            TokenKind::Include => {
                // If there's not exactly one parameter, error
                if param_span.1 - param_span.0 == 0 {
                    return Err(
                        "#INCLUDE expects exactly one string parameter. No parameters found."
                            .to_owned(),
                    );
                }

                // Get the file and insert it into the program
                if let TokenKind::String(path) = &self.tokens[param_span.0].kind {
                    let subprogram = read_file(path.as_str())?; // Read file
                    let mut lexer = Lexer::new(path.as_str(), subprogram); // Lex the file
                    lexer.tokenize()?;
                    let mut parser = Parser::new(path.as_str(), lexer.tokens.as_slice()); // Parse the file
                    parser.parse()?;
                    self.output.push_str(&parser.output); // Add contents of the other file
                    self.map.push(&parser.map); // Add the codemap of the other file
                } else {
                    return Err(format!(
                        "#INCLUDE expects just one string parameter. Found {:?}",
                        self.tokens[self.index].kind
                    ));
                }
            }
            TokenKind::Define => {
                if param_span.1 - param_span.0 == 0 {
                    return Err(
                        "#DEFINE expects at least one parameter, with the first always being a name or string. No parameters found."
                        .to_owned()
                    );
                }

                match &self.tokens[param_span.0].kind {
                    TokenKind::Code(def) => {
                        if self.deflist.contains_key(def) {
                            println!(
                                "\x1b[95mBASM-PREPROCESSOR: \x1b[33mWarning on line {} of {}:\x1b[0m\n  #DEFINE is called on '{}', but it was previously defined (value was overwritten)", 
                                self.line,
                                self.filename,
                                def
                            );
                        }
                        self.deflist.insert(def.to_owned(), &self.tokens[param_span.0 + 1..param_span.1]);
                    }
                    t => return Err(format!(
                        "#DEFINE expects a name as its first argument to be used as the constant's name.\n  Found {:?}",
                        t
                    )),
                };
            }
            TokenKind::Undef => {
                // If there's not exactly one parameter, error
                if param_span.1 - param_span.0 == 0 {
                    return Err(
                        "#UNDEF expects exactly one string parameter. No parameters found."
                            .to_owned(),
                    );
                }

                match &self.tokens[param_span.0].kind {
                    TokenKind::Code(def) => match self.deflist.remove(def) {
                        Some(_) => {},
                        None => println!("\x1b[95mBASM-PREPROCESSOR: \x1b[33mWarning on line {} of {}:\x1b[0m\n  #UNDEF is called on '{}', but it was not previously defined", self.line, self.filename, def),
                    }
                    t => return Err(format!(
                        "#UNDEF expects one name parameter\n  Found {:?}",
                        t
                    )),
                };
            }
            t => {
                return Err(format!(
                    "parse_directive() expected a directive, found '{:?}'",
                    t
                ))
            }
        }

        Ok(Some(()))
    }

    fn consume_whitespace(&mut self) {
        // If whitespace is found, skip over it
        if let Some(Token {
            kind: TokenKind::Whitespace,
            ..
        }) = self.peek()
        {
            self.index += 1;
        }
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
