use crate::codemap::{CodeMap, LineEntry};

#[derive(Debug)]
pub enum TokenKind {
    Code(String),   // For raw code
    Newline,        // \n
    Whitespace,     // Spaces
    None,    // For things that should be ignored
    Include, // For including other asm files
    Define,  // For defining constants
    Undef,   // For undefining constants
    Integer(i64),
    String(String),
}

#[derive(Debug)]
pub struct Token {
    pub kind: TokenKind,
    pub span: usize,
}

enum ReadKind {
    PassThrough,
    ReadParameters,
}

pub struct Lexer {
    pub data: String,
    pub tokens: Vec<Token>,
    pub map: CodeMap,
    span: (usize, usize),
    line: usize,
    state: ReadKind,
}

/// Returns a portion of a data from the start until pred returns false
fn take_while<F>(data: &str, pred: F) -> Result<(&str, usize), String>
where
    F: Fn(char) -> bool,
{
    let mut index = 0;

    for c in data.chars() {
        if !pred(c) {
            break;
        }
        index += c.len_utf8();
    }

    if index == 0 {
        Err("No matches".to_owned())
    } else {
        Ok((&data[..index], index))
    }
}

/// Returns the length of a span of whitespace excluding newlines
fn skip_white_space(data: &str) -> usize {
    match take_while(data, |c| c.is_whitespace() && c != '\n') {
        Ok((_, bytes_read)) => bytes_read,
        Err(_) => 0,
    }
}

/// Returns the length of a span from a ; to a newline
fn skip_comment(data: &str) -> usize {
    if data.starts_with(';') {
        let bytes_read = match take_while(data, |c| c != '\n') {
            Ok((_, bytes_read)) => bytes_read,
            Err(_) => panic!("Unexpected EOF in skip_comment"),
        };
        return bytes_read;
    }

    0
}

/// Returns the length of a span from a ; to a newline
fn skip_code(data: &str) -> usize {
    let bytes_read = match take_while(data, |c| c != '\n' && c != ';') {
        Ok((_, bytes_read)) => bytes_read,
        Err(_) => panic!("Unexpected EOF in skip_line"),
    };
    return bytes_read;
}

/// Returns the integer value of any decimal, binary, or hex string that data starts with
fn tokenize_number(data: &str) -> Result<Token, String> {
    let (read, bytes_read) = take_while(data, |c| c.is_alphanumeric())?;

    let result_num = if read.len() > 2 {
        match &read[0..2] {
            "0x" => i64::from_str_radix(&read[2..], 16),
            "0b" => i64::from_str_radix(&read[2..], 2),
            _ => read.parse::<i64>(),
        }
    } else {
        read.parse::<i64>()
    };

    let num = match result_num {
        Ok(n) => n,
        Err(_) => return Err(format!("Could not parse number: '{}'", read)),
    };

    Ok(Token {
        kind: TokenKind::Integer(num),
        span: bytes_read,
    })
}

/// Returns a String from the 2nd char of data to the next ", will break if there's no "
fn tokenize_string_literal(data: &str) -> Result<Token, String> {
    let mut final_string = String::new();
    let mut bytes_read = 0;

    let mut chars = data.chars();
    chars.next();

    loop {
        let next = match chars.next() {
            Some('"') => break,
            Some('\\') => {
                bytes_read += 1;
                match chars.next() {
                    Some('n') => '\n',
                    Some('0') => '\0',
                    Some('\\') => '\\',
                    Some('\"') => '"',
                    Some(c) => return Err(format!("{} is not a valid escape character", c)),
                    None => return Err("Reached EOF before finding a \"".to_owned()),
                }
            }
            Some(c) => c,
            None => return Err("Reached EOF before finding a \"".to_owned()),
        };

        bytes_read += next.len_utf8();

        final_string.push(next);
    }

    if bytes_read == 0 {
        return Err("No matches".to_owned());
    }

    Ok(Token {
        kind: TokenKind::String(final_string),
        span: bytes_read + 2,
    })
}

/// Tokenizes a single directive
fn tokenize_directive(data: &str) -> Result<Token, String> {
    let (read, bytes_read) = take_while(data, |c| c == '_' || c == '#' || c.is_alphanumeric())?;

    let token_kind = match &read.to_lowercase()[..] {
        "#include" => TokenKind::Include,
        "#define" => TokenKind::Define,
        "#undef" => TokenKind::Undef,
        s => return Err(format!("Unknown preprocessor directive '{}'.", s)),
    };

    Ok(Token {
        kind: token_kind,
        span: bytes_read,
    })
}

impl Lexer {
    pub fn new(data: String) -> Self {
        let map = CodeMap::new();
        let tokens = Vec::new();
        let span = (0, data.len());
        let line = 1;
        let state = ReadKind::PassThrough;

        Self {
            data,
            tokens,
            map,
            span,
            line,
            state,
        }
    }

    pub fn tokenize(&mut self) -> Result<(), String> {
        while self.span.0 < self.span.1 {
            let (kind, span) = match self
                .data
                .chars()
                .nth(self.span.0)
                .unwrap_or_else(|| panic!("Lexer object span broke. Did you forget a '\"'?\n"))
            {
                c if c.is_whitespace() && c != '\n' => match self.state {
                    ReadKind::PassThrough => {
                        (TokenKind::Whitespace, skip_white_space(self.get_selected()))
                    }
                    ReadKind::ReadParameters => {
                        (TokenKind::None, skip_white_space(self.get_selected()))
                    }
                },
                '\n' => {
                    self.line += 1;
                    (TokenKind::Newline, 1)
                }
                ';' => (TokenKind::None, skip_comment(self.get_selected())),
                _ => match self.tokenize_one_token() {
                    Ok(tok) => (tok.kind, tok.span),
                    Err(e) => return Err(format!("Error on line {}:\n  {}", self.line, e)),
                }
            };
            self.consume(span);
            match kind {
                TokenKind::None => {}
                _ => {
                    self.map.line_entries.push(
                        LineEntry {
                            filename_index: 0,
                            line: self.line,
                        }
                    );
                    self.tokens.push(Token { kind, span });
                }
            }
        }

        Ok(())
    }

    fn tokenize_one_token(&mut self) -> Result<Token, String> {
        let data = &self.get_selected().to_owned();
        let next = match data.chars().next() {
            Some(c) => c,
            None => return Err("Unexpected EOF".to_owned()),
        };

        let tok = match next {
            '#' => {
                self.state = ReadKind::ReadParameters;
                tokenize_directive(data)?
            }
            '"' => tokenize_string_literal(data)?,
            '0'..='9' => tokenize_number(data)?,
            _ => {
                self.state = ReadKind::PassThrough;
                let span = skip_code(data);
                Token {
                    kind: TokenKind::Code(data[0..span].to_owned()),
                    span,
                }
            }
        };

        Ok(tok)
    }

    fn consume(&mut self, amount: usize) {
        let (start, end) = self.span;
        self.span = (start + amount, end);
    }

    fn get_selected(&self) -> &str {
        &self.data[self.span.0..self.span.1]
    }
}
