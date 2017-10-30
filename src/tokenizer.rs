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

    current_index: usize,
    line_number: usize,
    column_number: usize,
}

pub fn tokenize(string: &str) -> Result<Vec<Token>, String> {
    let mut tokenizer = Tokenizer::new();
    return tokenizer.tokenize(string);
}

impl<'a> Tokenizer<'a> {
    fn new() -> Tokenizer<'a> {
        return Tokenizer {
            current_index: 0,
            line_number: 1,
            column_number: 0,

            target: "",
            tokens: Vec::new()
        };
    }

    fn reset(&mut self) {
        self.tokens = Vec::new();
        self.target = "";

        self.current_index = 0;
        self.column_number = 0;
        self.line_number = 1;
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
        self.current_index += 1;
    }

    fn next_index(&mut self) -> usize {
        let index = self.current_index;
        self.increment_file_info();

        return index;
    }

    fn next_char(&mut self) -> Option<char> {
        let index = self.next_index();
        return self.target.chars().nth(index);
    }

    fn peek_char(&mut self) -> Option<char> {
        return self.target.chars().nth(self.current_index);
    }

    fn save_token(&mut self, mut token: Token) {
        let from = token.file_info.index_from;
        let to = self.current_index;
        let text = String::from(&self.target[from..to]);

        token.str = text;
        token.file_info.column_number_to = self.column_number;
        token.file_info.line_number_to = self.line_number;
        token.file_info.index_to = self.current_index;

        self.tokens.push(token);
    }

    fn new_token(&mut self, tp: TokenType) -> Token {
        let token = Token {
            str: String::from(""),
            tp: tp,
            file_info: CodePoint {
                index_from: self.current_index-1,
                index_to: self.current_index-1,

                line_number_from: self.line_number,
                column_number_from: self.column_number,

                line_number_to: self.line_number,
                column_number_to: self.column_number,
            }
        };
        return token;
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
                    let token = self.new_token(TokenType::Numeric);

                    loop {
                        match self.peek_char() {
                            Some(c) => match c {
                                '0'...'9' | '.' => {self.next_char();}
                                _ => {break;}
                            },
                            None => {
                                break;
                            }
                        }
                    }

                    self.save_token(token);
                }
                'a' ... 'z' | 'A' ... 'Z' | '_' => {
                    let token = self.new_token(TokenType::Alphanumeric);

                    loop {
                        match self.peek_char() {
                            Some(c) => match c {
                                'a' ... 'z' | 'A' ... 'Z' | '_' | '0'...'9' => {self.next_char();}
                                _ => {break;}
                            },
                            None => {
                                break;
                            }
                        }
                    }

                    self.save_token(token);
                }
                '"' => {
                    let token = self.new_token(TokenType::String);

                    loop {
                        match self.next_char() {
                            Some(c) => match c {
                                '\\' => { self.next_char(); }
                                '"' => { break; }
                                _ => {}
                            },
                            None => {
                                let msg = format!("Tokenizer: Invalid end of input for \"string\" {}", c);
                                return Err(msg);
                            }
                        }
                    }

                    self.save_token(token);
                }
                '(' | ')' => {
                    let token = self.new_token(TokenType::Parenthesis);
                    self.save_token(token);
                }
                ':' => {
                    let mut token = self.new_token(TokenType::Undefined);

                    match self.peek_char() {
                        Some(c) => match c {
                            ':' => {
                                self.next_char();
                                token.tp = TokenType::StaticAssignment;
                            }
                            '=' => {
                                self.next_char();
                                token.tp = TokenType::DynamicAssignment;
                            }
                            'a' ... 'z' | 'A' ... 'Z' => {
                                token.tp = TokenType::Token;
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

                    self.save_token(token);
                }
                ';' => {
                    let token = self.new_token(TokenType::EndOfStatement);
                    self.save_token(token);
                }
                ' ' | '\n' | '\r' | '\t' => {}
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
