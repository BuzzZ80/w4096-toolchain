#[derive(Debug, Clone)]
pub enum TokenKind {
    Code(String), // For raw code
    Newline,      // \n
    Whitespace,   // Spaces
    None,         // For things that should be ignored
    Include,      // For including other asm files
    Define,       // For defining constants
    Undef,        // For undefining constants
    String(String),
}

#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub span: usize,
}

pub struct Lexer {
    pub data: String,
    pub tokens: Vec<Token>,
    span: (usize, usize),
    line: usize,
    filename: String,
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

fn tokenize_word(data: &str) -> Result<Token, String> {
    let (read, bytes_read) = take_while(data, |c| c == '_' || c.is_alphanumeric())?;
    Ok(Token {
        kind: TokenKind::Code(read.to_owned()),
        span: bytes_read,
    })
}

fn tokenize_other(data: &str) -> Result<Token, String> {
    let (read, bytes_read) = take_while(data, |c| !c.is_whitespace())?;
    Ok(Token {
        kind: TokenKind::Code(read.to_owned()),
        span: bytes_read,
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
    pub fn new(filename: &str, data: String) -> Self {
        let filename = filename.to_owned();
        let tokens = Vec::new();
        let span = (0, data.len());
        let line = 1;

        Self {
            data,
            tokens,
            span,
            line,
            filename,
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
                c if c.is_whitespace() && c != '\n' => {
                    (TokenKind::Whitespace, skip_white_space(self.get_selected()))
                }
                '\n' => {
                    self.line += 1;
                    (TokenKind::Newline, 1)
                }
                ';' => (TokenKind::None, skip_comment(self.get_selected())),
                _ => match self.tokenize_one_token() {
                    Ok(tok) => (tok.kind, tok.span),
                    Err(e) => {
                        return Err(format!(
                            "\x1b[91mError on line {} of {}:\x1b[0m\n  {}",
                            self.line, self.filename, e
                        ))
                    }
                },
            };
            self.consume(span);
            match kind {
                TokenKind::None => {}
                _ => self.tokens.push(Token { kind, span }),
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

        match next {
            '#' => tokenize_directive(data),
            '"' => tokenize_string_literal(data),
            c if c.is_alphanumeric() => tokenize_word(data),
            _ => tokenize_other(data),
        }
    }

    fn consume(&mut self, amount: usize) {
        let (start, end) = self.span;
        self.span = (start + amount, end);
    }

    fn get_selected(&self) -> &str {
        &self.data[self.span.0..self.span.1]
    }
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match &self.kind {
            TokenKind::Code(s) => write!(f, "{}", s),
            TokenKind::Newline => write!(f, "\n"),
            TokenKind::Whitespace => write!(f, " "),
            TokenKind::None => Ok(()),
            TokenKind::Include => write!(f, "#INCLUDE"),
            TokenKind::Define => write!(f, "#DEFINE"),
            TokenKind::Undef => write!(f, "#UNDEF"),
            TokenKind::String(s) => write!(f, r#""{}""#, s),
        }
    }
}
