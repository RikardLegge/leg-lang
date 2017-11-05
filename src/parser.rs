use tokenizer::Token;
use tokenizer::TokenType;
use tokenizer::TokenType::*;
use std::slice::Iter;
use std::iter::Peekable;
use std::fmt;
use std::ops::Deref;
use std::fmt::Debug;

#[derive(Debug)]
pub enum AstNodeType {
    Block(Box<AstBlock>),
    FunctionCall(Box<AstFunctionCall>),
    StringValue(Box<AstStringValue>),
    NumberValue(Box<AstNumberValue>),
    Variable(Box<AstVariable>),
    Assignment(Box<AstAssignment>)
}

#[derive(Debug)]
pub struct Ast {
    pub root: AstNodeType
}

pub struct AstBlock {
    pub statements: Vec<AstNodeType>
}

impl AstBlock {
    fn new() -> AstBlock {
        return AstBlock {
            statements: Vec::new()
        };
    }
}

impl fmt::Debug for AstBlock {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut statements_str = String::new();
        for wrapped_statement in &self.statements {
            let statement = wrapped_statement.deref();
            let statement_str = format!("{:?}", statement);
            statements_str.push_str(&statement_str);
        }
        write!(f, "AstBlock {{statements=[{}]}}", statements_str)
    }
}

#[derive(Debug)]
struct AstFunctionCall {
    pub name: String,
    pub arguments: Vec<AstNodeType>,
    pub body: Option<AstBlock>
}

#[derive(Debug)]
struct AstStringValue {
    pub value: String
}

#[derive(Debug)]
struct AstNumberValue {
    pub value: String
}

#[derive(Debug)]
struct AstVariable {
    pub name: String
}

#[derive(Debug)]
struct AstAssignment {
    pub to: AstVariable,
    pub from: AstNodeType
}

struct Parser<'a> {
    token_stream: Peekable<Iter<'a, Token>>,
    token_buffer: Vec<&'a Token>,
    current_token: &'a Token
}

impl<'a> Parser<'a> {
    fn report_error(&self, token: &Token, msg: &str) {
        let msg = format!("Parser Error!!!\n\n{}\nData: {:?}", msg, token);
        panic!(msg);
    }

    fn next_token(&mut self) -> Option<&'a Token> {
        return match self.token_stream.next() {
            Some(token) => {
                self.current_token = token;
                Some(token)
            }
            None => {
                None
            }
        };
    }

    fn peek_token(&mut self) -> Option<&'a Token> {
        return match self.token_stream.peek() {
            Some(token) => {
                Some(*token)
            }
            None => {
                None
            }
        };
    }

    fn parse_number(&mut self) -> AstNodeType {
        unimplemented!()
    }

    fn parse_string(&mut self) -> AstNodeType {
        unimplemented!()
    }

    fn parse_expression(&mut self) -> AstNodeType {
        match self.current_token.get_type() {
            StaticString => {
                let text = self.current_token.get_text();
                let text_without_quotes = &text[1..text.len()-1];
                let value = AstStringValue {
                    value: String::from(text_without_quotes)
                };

                return AstNodeType::StringValue(Box::new(value));
            }
            Alphanumeric => {
                let name = self.current_token.get_text();
                let variable = AstVariable {
                    name: name
                };

                return AstNodeType::Variable(Box::new(variable));
            }
            _ => {
                self.report_error(self.current_token, "Unexpected character when parsing an expression");
                unreachable!();
            }
        }

        self.report_error(self.current_token, "Unexpected character when parsing an expression");
        unreachable!();
    }

    fn parse_assignment(&mut self) -> AstNodeType {
        unimplemented!()
    }

    fn parse_typed_assignment(&mut self) -> AstNodeType {
        assert_eq!(self.current_token.get_type(), Alphanumeric);
        let variable_name = self.current_token.get_text();
        if let Some(symbol_start_token) = self.next_token() {
            assert_eq!(symbol_start_token.get_type(), Symbol);
            if let Some(symbol_token) = self.next_token() {
                assert_eq!(symbol_token.get_type(), Alphanumeric);
                if let Some(assignment_token) = self.next_token() {
                    let assignment_type = assignment_token.get_type();
                    let variable = AstVariable {
                        name: variable_name
                    };

                    self.next_token();
                    let expression = self.parse_expression();
                    let assignment = AstAssignment {
                        to: variable,
                        from: expression
                    };
                    return AstNodeType::Assignment(Box::new(assignment));
                }
            }
        }

        self.report_error(self.current_token, "Unexpected character when parsing a typed assignment");
        unreachable!();
    }

    fn parse_function_call(&mut self) -> AstNodeType {
        assert_eq!(self.current_token.get_type(), Alphanumeric);
        let function_name = self.current_token.get_text();
        if let Some(function_args_start) = self.next_token() {
            assert_eq!(self.current_token.get_type(), OpenParenthesis);

            let mut arguments: Vec<AstNodeType> = Vec::new();
            while let Some(token) = self.next_token() {
                if token.get_type() == CloseParenthesis {
                    break;
                }

                let expression = self.parse_expression();
                arguments.push(expression);
            }
            let call = AstFunctionCall {
                name: function_name,
                arguments: arguments,
                body: None
            };
            return AstNodeType::FunctionCall(Box::new(call));
        }

        self.report_error(self.current_token, "Unexpected character when parsing a typed assignment");
        unreachable!();
    }

    fn parse_variable(&mut self) -> AstNodeType {
        if let Some(token) = self.peek_token() {
            return match token.get_type() {
                Symbol => {
                    self.parse_typed_assignment()
                }
                DynamicAssignment | StaticAssignment => {
                    self.parse_assignment()
                }
                OpenParenthesis => {
                    self.parse_function_call()
                }
                _ => {
                    self.report_error(token, "Unexpected token when after variable/function identifier");
                    unreachable!();
                }
            };
        } else {
            self.report_error(self.current_token, "Unexpected end of stream when parsing a variable");
            unreachable!();
        }
    }

    fn parse_statement(&mut self) -> AstNodeType {
        if let Some(token) = self.next_token() {
            let evaluatable = match token.get_type() {
                Alphanumeric => {
                    self.parse_variable()
                }
                Numeric => {
                    self.parse_number()
                }
                StaticString => {
                    self.parse_string()
                }
                OpenParenthesis => {
                    self.parse_expression()
                }
                OpenBlock => {
                    self.parse_block()
                }
                _ => {
                    self.report_error(token, "Invalid token in block");
                    unreachable!();
                }
            };

            if let Some(token) = self.next_token() {
                if token.get_type() == EndOfStatement {
                    return evaluatable;
                }
            }
        }

        self.report_error(self.current_token, "Unexpected end of stream when parsing a variable");
        unreachable!();
    }

    fn parse_block(&mut self) -> AstNodeType {
        let mut block = AstBlock::new();

        while let Some(token) = self.peek_token() {
            if token.get_type() == CloseBlock {
                self.next_token();
                break;
            }

            let evaluatable = self.parse_statement();
            block.statements.push(evaluatable);
        }

        return AstNodeType::Block(Box::new(block));
    }

    fn parse_root_block(&mut self) -> AstNodeType {
        let mut block = AstBlock::new();

        while let Some(token) = self.next_token() {
            match token.get_type() {
                OpenBlock => {
                    let child_block = self.parse_block();
                    block.statements.push(child_block);
                }
                _ => { self.report_error(token, "Invalid end of input, file must start with a root block") }
            }
        }

        return AstNodeType::Block(Box::new(block));
    }

    fn parse(&mut self) -> Ast {
        return Ast {
            root: self.parse_root_block()
        };
    }
}

pub fn parse(tokens: &Vec<Token>) -> Ast {
    let null_token = Token::new();
    let mut iter = tokens.iter().peekable();

    let mut parser = Parser {
        token_buffer: Vec::new(),
        token_stream: iter,
        current_token: &null_token
    };
    return parser.parse();
}

