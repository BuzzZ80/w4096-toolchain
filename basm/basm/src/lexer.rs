#[derive(Debug, Clone)]
pub enum TokenKind {
    // Types
    String(String),
    Label(String),
    Integer(u16),

    // Symbols
    Comma,
    OpenParen,
    CloseParen,
    Plus,
    Minus,
    Times,
    Div,

    // Keywords
    // Registers
    Ac,
    Br,
    Ix,
    Sp,
    Imm,
    Stack,

    // Instructions
    Mov,
    Add,
    Adc,
    Sub,
    Sbb,
    Sbw,
    Swb,
    Nnd,
    And,
    Aib,
    Anb,
    Bia,
    Bna,
    Ora,
    Nor,
    Jmp,
    Hlt,
    Jsr,
    Ret,
    Dec,
    Inc,
    Cmp,
    Xor,
    Xnr,
    Clc,
    Clz,
    Sec,
    Sez,

    // Conditionals
    C,
    Z,
    Nc,
    Nz,
    Cz,
    Ncz,

    // Assembler directives
    Org,
    Db,

    //Other
    None,
}

#[derive(Debug)]
pub struct Token {
    pub kind: TokenKind,
    pub span: usize,
    pub line: usize,
}

#[derive(Debug)]
pub struct Lexer<'a> {
    data: &'a str,
    span: (usize, usize),
    line: usize,
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

/// Returns the integer value of any decimal, binary, or hex string that data starts with
fn tokenize_number(data: &str) -> Result<Token, String> {
    let (read, bytes_read) = take_while(data, |c| c.is_alphanumeric())?;

    let result_num = if read.len() > 2 {
        match &read[0..2] {
            "0x" => u16::from_str_radix(&read[2..], 16),
            "0o" => u16::from_str_radix(&read[2..], 8),
            "0b" => u16::from_str_radix(&read[2..], 2),
            _ => read.parse::<u16>(),
        }
    } else {
        read.parse::<u16>()
    };

    let num = match result_num {
        Ok(n) => n,
        Err(_) => return Err(format!("Could not parse number: '{}'", read)),
    };

    Ok(Token{
        kind: TokenKind::Integer(num), 
        span: bytes_read,
        line: 0,
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
        span: bytes_read,
        line: 0,
    })
}

/// Returns a keyword or label from the start of data
fn tokenize_identifier(data: &str) -> Result<Token, String> {
    let (read, bytes_read) = take_while(data, |c| c == '_' || c.is_alphanumeric())?;

    let token_kind = match &read.to_lowercase()[..] {
        "mov" => TokenKind::Mov,
        "add" => TokenKind::Add,
        "adc" => TokenKind::Adc,
        "sub" => TokenKind::Sub,
        "sbb" => TokenKind::Sbb,
        "sbw" => TokenKind::Sbw,
        "swb" => TokenKind::Swb,
        "nnd" => TokenKind::Nnd,
        "and" => TokenKind::And,
        "aib" => TokenKind::Aib,
        "anb" => TokenKind::Anb,
        "bia" => TokenKind::Bia,
        "bna" => TokenKind::Bna,
        "ora" => TokenKind::Ora,
        "nor" => TokenKind::Nor,
        "jmp" => TokenKind::Jmp,
        "hlt" => TokenKind::Hlt,
        "jsr" => TokenKind::Jsr,
        "ret" => TokenKind::Ret,
        "dec" => TokenKind::Dec,
        "inc" => TokenKind::Inc,
        "cmp" => TokenKind::Cmp,
        "xor" => TokenKind::Xor,
        "xnr" => TokenKind::Xnr,
        "clc" => TokenKind::Clc,
        "clz" => TokenKind::Clz,
        "sec" => TokenKind::Sec,
        "sez" => TokenKind::Sez,
        "c" => TokenKind::C,
        "z" => TokenKind::Z,
        "nc" => TokenKind::Nc,
        "nz" => TokenKind::Nz,
        "cz" => TokenKind::Cz,
        "ncz" => TokenKind::Ncz,
        "ac" => TokenKind::Ac,
        "br" => TokenKind::Br,
        "ix" => TokenKind::Ix,
        "sp" => TokenKind::Sp,
        "imm" => TokenKind::Imm,
        "stack" => TokenKind::Stack,
        s => TokenKind::Label(s.to_owned()),
    };

    Ok(Token{
        kind: token_kind, 
        span: bytes_read, 
        line: 0,
    })
}

/// Tokenizes a single directive
fn tokenize_directive(data: &str) -> Result<Token, String> {
    let (read, bytes_read) = take_while(data, |c| c == '_' || c == '.' || c.is_alphanumeric())?;

    let token_kind = match &read.to_lowercase()[..] {
        ".org" => TokenKind::Org,
        ".db" => TokenKind::Db,
        s => return Err(format!("Unknown dot directive '{}'.", s)),
    };

    Ok(Token{
        kind: token_kind, 
        span: bytes_read, 
        line: 0,
    })
}

/// Tokenizes any character, string, integer, keyword, label, etc. Does not skip comments or whitespace
pub fn tokenize_one_token(data: &str) -> Result<Token, String> {
    let mut chars = data.chars();
    let next = match chars.next() {
        Some(c) => c,
        None => return Err("Unexpected EOF".to_owned()),
    };
    //let peek = chars.next().unwrap_or('\0');

    let token = match next {
        ',' => Token {
            kind: TokenKind::Comma,
            span: 1,
            line: 0,
        },
        '(' => Token {
            kind: TokenKind::OpenParen,
            span: 1,
            line: 0,
        },
        ')' => Token {
            kind: TokenKind::CloseParen,
            span: 1,
            line: 0,
        },
        '+' => Token {
            kind: TokenKind::Plus,
            span: 1,
            line: 0,
        },
        '-' => Token {
            kind: TokenKind::Minus,
            span: 1,
            line: 0,
        },
        '*' => Token {
            kind: TokenKind::Times,
            span: 1,
            line: 0,
        },
        '/' => Token {
            kind: TokenKind::Div,
            span: 1,
            line: 0,
        },
        '.' => tokenize_directive(data)?,
        '0'..='9' => tokenize_number(data)?,
        '"' => tokenize_string_literal(data)?,
        c if c.is_alphanumeric() || c == '_' => tokenize_identifier(data)?,
        c => return Err(format!("Unexpected character {}", c)),
    };

    Ok(token)
}

impl<'a> Lexer<'a> {
    /// Creates a new, valid Lexer struct from a &str
    pub fn new(data: &'a str) -> Self {
        Self {
            data,
            span: (0, data.len()),
            line: 1,
        }
    }

    /// Tokenizes all of self.data, returning a Vec of all the tokens to be passed to a parser
    pub fn tokenize(&mut self) -> Result<Vec<Token>, String> {
        let mut tokens = Vec::new();

        while self.span.0 != self.span.1 {
            let (val, consumed) = match self.data.chars().nth(self.span.0).unwrap_or_else(|| {
                panic!(
                    "Lexer object span broke.\n{:#?}\nDid you forget a '\"'?\n",
                    self
                )
            }) {
                c if c.is_whitespace() && c != '\n' => {
                    (TokenKind::None, skip_white_space(self.get_selected()))
                }
                ';' => (TokenKind::None, skip_comment(self.get_selected())),
                '\n' => {
                    self.line += 1;
                    (TokenKind::None, 1)
                }
                _ => match tokenize_one_token(self.get_selected()) {
                    Ok(tok) => (tok.kind, tok.span),
                    Err(e) => return Err(format!("Error on line {}:\n  {}", self.line, e)),
                },
            };

            self.consume(consumed);

            match val {
                TokenKind::None => {}
                _ => {
                    tokens.push(Token{
                        kind: val, 
                        span: consumed, 
                        line: self.line,
                    });
                }
            }
        }

        Ok(tokens)
    }

    /// Removes amount characters from the beginning of self.data by increasing self.span.0
    fn consume(&mut self, amount: usize) {
        let (start, end) = self.span;
        self.span = (start + amount, end);
    }

    /// Returns the portion of self.data selected by self.span
    fn get_selected(&self) -> &str {
        &self.data[self.span.0..self.span.1]
    }
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self.kind)
    }
}
