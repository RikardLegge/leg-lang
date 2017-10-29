use file_info::CodePoint;
use std::mem;

#[derive(Debug)]
pub struct Token {
    str: String,
    tp: TokenType,
    file_info: CodePoint
}

#[derive(Debug)]
#[derive(Copy, Clone)]
pub enum TokenType {
    Alphanumeric,
    Numeric,

    Parenthesis,

    StaticAssignment,
    DynamicAssignment,

    Token,
    String,

    EndOfStatement,
    Undefined
}

struct Tokenizer<'a> {
    tokens: Vec<Token>,
    target: &'a str,

    start_index: usize,
    current_index: usize,
    current_type: TokenType
}

pub fn tokenize(string: &str) -> Result<Vec<Token>, String> {
    let mut tokenizer = Tokenizer::new();
    return tokenizer.tokenize(string);
}

impl <'a>Tokenizer<'a> {
    fn new() -> Tokenizer<'a> {
        return Tokenizer {
            start_index: 0,
            current_index: 0,
            current_type: TokenType::Undefined,

            target: "",
            tokens: Vec::new()
        };
    }

    fn reset(&mut self) {
        self.tokens = Vec::new();
        self.target = "";

        self.start_index = 0;
        self.current_index = 0;
        self.current_type = TokenType::Undefined;
    }

    fn next_index(&mut self) -> usize {
        let index = self.current_index;
        self.current_index += 1;

        return index;
    }

    fn next_char(&mut self) -> Option<char> {
        let index = self.next_index();
        return self.target.chars().nth(index);
    }

    fn peek_char(&mut self) -> Option<char> {
        return self.target.chars().nth(self.current_index);
    }

    fn save_token(&mut self) {
        let text = String::from(&self.target[self.start_index..self.current_index - 1]);
        self.save_token_with_text(text);
    }

    fn save_token_include_current(&mut self) {
        let text = String::from(&self.target[self.start_index..self.current_index]);
        self.save_token_with_text(text);
    }

    fn save_token_with_text(&mut self, text: String) {
        let file_info = CodePoint {
            start_index: self.start_index,
            end_index: self.current_index
        };
        let token = Token { str: text, tp: self.current_type, file_info: file_info };
        self.tokens.push(token);

        self.reset_index();
    }

    fn reset_index(&mut self) {
        self.start_index = self.current_index;
        self.current_type = TokenType::Undefined;
    }

    fn end_previous_token(&mut self) {
        match self.current_type {
            TokenType::Undefined => {}
            _ => {
                self.save_token();
            }
        }
    }

    fn tokenize(&mut self, target: &'a str) -> Result<Vec<Token>, String> {
        self.target = target;
        let res = self.tokenize_using_state();
        self.reset();

        return res;
    }

    fn tokenize_using_state(&mut self) -> Result<Vec<Token>, String> {
        while let Some(c) = self.next_char() {
            match c {
                '0' ... '9' => {
                    match self.current_type {
                        TokenType::Undefined => self.current_type = TokenType::Numeric,
                        TokenType::Numeric | TokenType::Alphanumeric => {}
                        _ => {
                            let msg = format!("Tokenizer: Invalid use of numeric characters!");
                            return Err(msg);
                        }
                    }
                }
                'a' ... 'z' | 'A' ... 'Z' | '_' => {
                    match self.current_type {
                        TokenType::Undefined => self.current_type = TokenType::Alphanumeric,
                        TokenType::Alphanumeric => {}
                        _ => {
                            let msg = format!("Tokenizer: Invalid use of alphanumeric characters!");
                            return Err(msg);
                        }
                    }
                }
                '"' => {
                    self.end_previous_token();
                    self.current_type = TokenType::String;

                    loop {
                        match self.next_char() {
                            Some(c) => match c {
                                '\\' => { self.next_char(); }
                                '"' => {
                                    self.save_token_include_current();
                                    break;
                                }
                                _ => {}
                            },
                            None => {
                                let msg = format!("Tokenizer: Invalid end of input for \"string\" {}", c);
                                return Err(msg);
                            }
                        }
                    }
                }
                '(' | ')' => {
                    self.end_previous_token();
                    self.current_type = TokenType::Parenthesis;
                    self.save_token_include_current();
                }
                ':' => {
                    self.end_previous_token();

                    match self.peek_char() {
                        Some(c) => match c {
                            ':' => {
                                self.next_index();
                                self.current_type = TokenType::StaticAssignment;
                            }
                            '=' => {
                                self.next_index();
                                self.current_type = TokenType::DynamicAssignment;
                            }
                            'a' ... 'z' | 'A' ... 'Z' => {
                                self.current_type = TokenType::Token;
                                self.save_token_include_current();
                            }
                            _ => {
                                let msg = format!("Tokenizer: Invalid end of input for : {}", c);
                                return Err(msg);
                            }
                        },
                        None => {
                            let msg = format!("Tokenizer: Invalid end of input for :");
                            return Err(msg);
                        }
                    }
                }
                ' ' | '\n' | '\r' | '\t' => {
                    match self.current_type {
                        TokenType::Undefined => {
                            self.reset_index()
                        }
                        _ => {
                            self.save_token();
                        }
                    }
                }
                ';' => {
                    self.end_previous_token();
                    self.current_type = TokenType::EndOfStatement;
                    self.save_token_include_current();
                }
                _ => {
                    let msg = format!("Invalid end of input: {}", c);
                    return Err(msg);
                }
            }
        }

        let tokens = mem::replace(&mut self.tokens, Vec::new());
        return Ok(tokens);
    }
}
