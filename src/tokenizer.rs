use file_info::CodePoint;
use std::mem;
use std::iter::Peekable;
use std::str::Chars;
use std::error::Error;

use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt;

#[derive(Debug)]
pub struct TokenizationError {
    token: Token,
    desc: String
}

impl Display for TokenizationError {
    fn fmt(&self, _: &mut Formatter) -> fmt::Result {
        unimplemented!()
    }
}

impl TokenizationError {
    fn new(token: Token, desc: String) -> TokenizationError {
        return TokenizationError {
            token: token,
            desc: desc
        }
    }
}

impl Error for TokenizationError {
    fn description(&self) -> &str {
        "Tokenization error!!!"
    }

    fn cause(&self) -> Option<&Error> {
        None
    }
}

#[derive(Debug)]
pub struct Token {
    text: String,
    token_type: TokenType,
    file_info: CodePoint
}

impl Token {
    pub fn new() -> Token {
        return Token {
            text: String::from(""),
            token_type: TokenType::Undefined,
            file_info: CodePoint {
                line_number_from: 0,
                column_number_from: 0,

                line_number_to: 0,
                column_number_to: 0,
            }
        };
    }

    pub fn get_type(&self) -> TokenType {
        return self.token_type;
    }

    pub fn get_text(&self) -> String {
        return self.text.clone();
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum TokenType {
    Alphanumeric,
    Numeric,

    OpenParenthesis,
    CloseParenthesis,

    OpenBlock,
    CloseBlock,

    ParameterDivider,
    SubElement,

    StaticAssignment,
    DynamicAssignment,

    Symbol,
    StaticString,

    Operator,

    EndOfStatement,
    Undefined
}

struct Tokenizer<'a> {
    tokens: Vec<Token>,
    char_stream: Peekable<Chars<'a>>,
    current_char: char,

    line_number: usize,
    column_number: usize,
}

pub fn tokenize(string: &str) -> Result<Vec<Token>, TokenizationError> {
    let mut tokenizer = Tokenizer::new();
    return tokenizer.tokenize(string);
}

impl<'a> Tokenizer<'a> {
    fn new() -> Tokenizer<'a> {
        return Tokenizer {
            line_number: 1,
            column_number: 0,

            char_stream: "".chars().peekable(),
            current_char: '\n',
            tokens: Vec::new()
        };
    }

    fn reset(&mut self) {
        self.line_number = 1;
        self.column_number = 0;

        self.char_stream = "".chars().peekable();
        self.current_char = '\n';
        self.tokens = Vec::new();
    }

    fn increment_file_info(&mut self) {
        if let Some(c) = self.peek_char() {
            match c {
                '\n' => {
                    self.line_number += 1;
                    self.column_number = 1;
                }
                _ => {
                    self.column_number += 1;
                }
            }
        }
    }

    fn next_char(&mut self) -> Option<char> {
        self.increment_file_info();
        return match self.char_stream.next() {
            Some(c) => {
                self.current_char = c;
                Some(c)
            }
            None => None
        };
    }

    fn add_next_char(&mut self, token: &mut Token) -> Option<char> {
        let next = self.next_char();

        if let Some(c) = next {
            token.text.push(c);
        }

        return next;
    }

    fn peek_char(&mut self) -> Option<char> {
        return match self.char_stream.peek() {
            Some(c) => Some(*c),
            None => None
        };
    }

    fn save_token(&mut self, mut token: Token) {
        token.file_info.column_number_to = self.column_number;
        token.file_info.line_number_to = self.line_number;

        self.tokens.push(token);
    }

    fn new_token(&mut self, tp: TokenType) -> Token {
        let token = Token {
            text: self.current_char.to_string(),
            token_type: tp,
            file_info: CodePoint {
                line_number_from: self.line_number,
                column_number_from: self.column_number,

                line_number_to: self.line_number,
                column_number_to: self.column_number,
            }
        };
        return token;
    }

    fn save_new_token(&mut self, tp: TokenType) {
        let token = self.new_token(tp);
        self.save_token(token);
    }

    fn tokenize(&mut self, target: &'a str) -> Result<Vec<Token>, TokenizationError> {
        self.char_stream = target.chars().peekable();
        let res = self.tokenize_using_state();
        self.reset();

        return res;
    }

    fn tokenize_number(&mut self) -> Token {
        let mut token = self.new_token(TokenType::Numeric);

        loop {
            match self.peek_char() {
                Some(c) => match c {
                    '0' ... '9' | '.' => { self.add_next_char(&mut token); }
                    _ => { break; }
                },
                None => {
                    break;
                }
            }
        }

        return token;
    }

    fn tokenize_word(&mut self) -> Token {
        let mut token = self.new_token(TokenType::Alphanumeric);

        loop {
            match self.peek_char() {
                Some(c) => match c {
                    'a' ... 'z' | 'A' ... 'Z' | '_' | '0' ... '9' => { self.add_next_char(&mut token); }
                    _ => { break; }
                },
                None => {
                    break;
                }
            }
        }

        return token;
    }

    fn tokenize_string(&mut self) -> Result<Token, TokenizationError> {
        let mut token = self.new_token(TokenType::StaticString);

        loop {
            match self.add_next_char(&mut token) {
                Some(c) => match c {
                    '\\' => { self.add_next_char(&mut token); }
                    '"' => { break; }
                    _ => {}
                },
                None => {
                    let msg = format!("Invalid end of input for \"string\"");
                    return Err(TokenizationError::new(token, msg));
                }
            }
        }

        return Ok(token);
    }

    fn tokenize_static_assignment(&mut self) -> Token {
        let mut token = self.new_token(TokenType::StaticAssignment);
        self.add_next_char(&mut token);
        return token;
    }

    fn tokenize_dynamic_assignment(&mut self) -> Token {
        let mut token = self.new_token(TokenType::DynamicAssignment);
        self.add_next_char(&mut token);
        return token;
    }

    fn tokenize_symbol(&mut self) -> Token {
        let token = self.new_token(TokenType::Symbol);
        return token;
    }

    fn tokenize_operator(&mut self) -> Token {
        let token = self.new_token(TokenType::Operator);
        return token;
    }

    fn tokenize_using_state(&mut self) -> Result<Vec<Token>, TokenizationError> {
        while let Some(c) = self.next_char() {
            match c {
                '0' ... '9' => {
                    let token = self.tokenize_number();
                    self.save_token(token);
                }
                'a' ... 'z' | 'A' ... 'Z' | '_' => {
                    let token = self.tokenize_word();
                    self.save_token(token);
                }
                '"' => {
                    let token = self.tokenize_string()?;
                    self.save_token(token);
                }
                '(' => {
                    let token = self.new_token(TokenType::OpenParenthesis);
                    self.save_token(token);
                }
                ')' => {
                    let token = self.new_token(TokenType::CloseParenthesis);
                    self.save_token(token);
                }
                '+' | '-' | '*' | '^' => {
                    let token = self.tokenize_operator();
                    self.save_token(token);
                }
                '{' => {
                    let token = self.new_token(TokenType::OpenBlock);
                    self.save_token(token);
                }
                '}' => {
                    let token = self.new_token(TokenType::CloseBlock);
                    self.save_token(token);
                }
                '/' => {
                    // Add comment support
                    let token = self.tokenize_operator();
                    self.save_token(token);
                }
                ':' => {
                    match self.peek_char() {
                        Some(c) => {
                            match c {
                                ':' => {
                                    let token = self.tokenize_static_assignment();
                                    self.save_token(token);
                                }
                                '=' => {
                                    let token = self.tokenize_dynamic_assignment();
                                    self.save_token(token);
                                }
                                'a' ... 'z' | 'A' ... 'Z' => {
                                    let token = self.tokenize_symbol();
                                    self.save_token(token);
                                }
                                _ => {
                                    let token = self.new_token(TokenType::Undefined);
                                    let msg = format!("Invalid character preceding (:): {}", c);
                                    return Err(TokenizationError::new(token, msg));
                                }
                            }
                        }
                        None => {
                            let token = self.new_token(TokenType::Undefined);
                            let msg = format!("Invalid end of input after :");
                            return Err(TokenizationError::new(token, msg));
                        }
                    }
                }
                ';' => {
                    self.save_new_token(TokenType::EndOfStatement);
                }
                ',' => {
                    self.save_new_token(TokenType::ParameterDivider);
                }
                '.' => {
                    self.save_new_token(TokenType::SubElement);
                }
                ' ' | '\n' | '\r' | '\t' => {}
                _ => {
                    let token = self.new_token(TokenType::Undefined);
                    let msg = format!("Invalid end of input: {}", c);
                    return Err(TokenizationError::new(token, msg));
                }
            }
        }

        let tokens = mem::replace(&mut self.tokens, Vec::new());
        return Ok(tokens);
    }
}
