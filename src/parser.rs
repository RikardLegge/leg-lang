use tokenizer::Token;
use tokenizer::TokenType::*;
use std::slice::Iter;
use std::iter::Peekable;
use std::fmt;
use std::ops::Deref;

use std::error::Error;
use std::fmt::Display;
use std::fmt::Formatter;

#[derive(Debug)]
pub struct ParsingError {
    token: Token,
    desc: String
}

impl Display for ParsingError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        writeln!(f, "ParsingError: \n{}\n\n{:?}", self.desc, self.token)
    }
}

impl ParsingError {
    fn new(token: &Token, desc: String) -> ParsingError {
        return ParsingError {
            token: token.clone(),
            desc: desc
        }
    }
}

impl Error for ParsingError {
    fn description(&self) -> &str {
        "Parsing error"
    }
}

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

    fn parse_number(&mut self) -> Result<AstNodeType, ParsingError> {
        unimplemented!()
    }

    fn parse_string(&mut self) -> Result<AstNodeType, ParsingError> {
        assert_eq!(self.current_token.get_type(), StaticString);

        let text = self.current_token.get_text();
        let text_without_quotes = &text[1..text.len()-1];
        let value = AstStringValue {
            value: String::from(text_without_quotes)
        };

        let node = AstNodeType::StringValue(Box::new(value));
        return Ok(node);
    }

    fn parse_expression(&mut self) -> Result<AstNodeType, ParsingError> {
        match self.current_token.get_type() {
            StaticString => {
                return self.parse_string();
            }
            Alphanumeric => {
                let name = self.current_token.get_text();
                let variable = AstVariable {
                    name: name
                };

                let node = AstNodeType::Variable(Box::new(variable));
                return Ok(node);
            }
            _ => {
                let msg = format!("Unexpected character when parsing an expression");
                return Err(ParsingError::new(self.current_token, msg));
            }
        }
    }

    fn parse_assignment(&mut self) -> Result<AstNodeType, ParsingError> {
        unimplemented!()
    }

    fn parse_typed_assignment(&mut self) -> Result<AstNodeType, ParsingError> {
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
                    let expression = self.parse_expression()?;
                    let assignment = AstAssignment {
                        to: variable,
                        from: expression
                    };
                    let node = AstNodeType::Assignment(Box::new(assignment));
                    return Ok(node);
                }
            }
        }

        let msg = format!("Unexpected character when parsing a typed assignment");
        return Err(ParsingError::new(self.current_token, msg));

    }

    fn parse_function_call(&mut self) -> Result<AstNodeType, ParsingError> {
        assert_eq!(self.current_token.get_type(), Alphanumeric);
        let function_name = self.current_token.get_text();
        if let Some(function_args_start) = self.next_token() {
            assert_eq!(self.current_token.get_type(), OpenParenthesis);

            let mut arguments: Vec<AstNodeType> = Vec::new();
            while let Some(token) = self.next_token() {
                if token.get_type() == CloseParenthesis {
                    break;
                }

                let expression = self.parse_expression()?;
                arguments.push(expression);
            }
            let call = AstFunctionCall {
                name: function_name,
                arguments: arguments,
                body: None
            };
            let node = AstNodeType::FunctionCall(Box::new(call));
            return  Ok(node);
        }

        let msg = format!("Unexpected character when parsing a typed assignment");
        return Err(ParsingError::new(self.current_token, msg));
    }

    fn parse_variable(&mut self) -> Result<AstNodeType, ParsingError> {
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
                    let msg = format!( "Unexpected token when after variable/function identifier");
                    Err(ParsingError::new(token, msg))
                }
            };
        } else {
            let msg = format!("Unexpected end of stream when parsing a variable");
            return Err(ParsingError::new(self.current_token, msg));
        }
    }

    fn parse_statement(&mut self) -> Result<AstNodeType, ParsingError> {
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
                    let msg = format!("Invalid token in block");
                    Err(ParsingError::new(self.current_token, msg))
                }
            };

            if let Some(token) = self.next_token() {
                if token.get_type() == EndOfStatement {
                    return evaluatable;
                }
            }
        }

        let msg = format!("Unexpected end of stream when parsing statement");
        return Err(ParsingError::new(self.current_token, msg));
    }

    fn parse_block(&mut self) -> Result<AstNodeType, ParsingError> {
        let mut block = AstBlock::new();

        while let Some(token) = self.peek_token() {
            if token.get_type() == CloseBlock {
                self.next_token();
                break;
            }

            let evaluatable = self.parse_statement()?;
            block.statements.push(evaluatable);
        }

        let node = AstNodeType::Block(Box::new(block));
        return Ok(node);
    }

    fn parse(&mut self) -> Result<Ast, ParsingError> {
        let root = self.parse_block()?;
        let ast =  Ast {
            root: root
        };

        return Ok(ast);
    }
}

pub fn parse(tokens: &Vec<Token>) -> Result<Ast, ParsingError> {
    let null_token = Token::new();
    let mut iter = tokens.iter().peekable();

    let mut parser = Parser {
        token_buffer: Vec::new(),
        token_stream: iter,
        current_token: &null_token
    };
    return parser.parse();
}

