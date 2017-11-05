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
        };
    }
}

impl Error for ParsingError {
    fn description(&self) -> &str {
        "Parsing error"
    }
}

#[derive(Debug, Copy, Clone)]
pub enum AstOperator {
    Add,
    Sub,
    Mult,
    Div,
    Pow,
    Mod
}

impl AstOperator {
    fn from_token(token: &Token) -> AstOperator {
        assert_eq!(token.get_type(), Operator);
        let c = &token.get_text()[0..];
        return match c {
            "+" => { AstOperator::Add }
            "-" => { AstOperator::Sub }
            "*" => { AstOperator::Mult }
            "/" => { AstOperator::Div }
            "^" => { AstOperator::Pow }
            "%" => { AstOperator::Mod }
            _ => { panic!("Can not interpret '{}' as an operator", c); }
        };
    }
}

#[derive(Debug)]
pub enum AstNodeType {
    Block(Box<AstBlock>),
    OperatorCall(Box<AstOperatorCall>),
    FunctionCall(Box<AstFunctionCall>),
    StringValue(Box<AstStringValue>),
    NumberValue(Box<AstNumberValue>),
    Variable(Box<AstVariable>),
    Assignment(Box<AstAssignment>),
    NullValue(Box<AstNullValue>)
}

#[derive(Debug)]
pub struct Ast {
    pub root: AstNodeType
}

pub struct AstBlock {
    pub statements: Vec<AstNodeType>
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
pub struct AstNullValue {}

impl AstBlock {
    fn new() -> AstBlock {
        return AstBlock {
            statements: Vec::new()
        };
    }
}


#[derive(Debug)]
pub struct AstFunctionCall {
    pub name: String,
    pub arguments: Vec<AstNodeType>,
    pub body: Option<AstBlock>
}

#[derive(Debug)]
pub struct AstOperatorCall {
    pub rhs: AstNodeType,
    pub lhs: AstNodeType,
    pub operator: AstOperator
}

#[derive(Debug)]
struct AstStringValue {
    pub value: String
}

#[derive(Debug)]
pub struct AstNumberValue {
    pub value: f64
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
        assert_eq!(self.current_token.get_type(), Numeric);

        let text = self.current_token.get_text();
        let maybe_number = match text.parse::<f64>() {
            Ok(number) => { Ok(number) }
            Err(err) => {
                let msg = format!("Failed to parse number: {}", text);
                Err(ParsingError::new(self.current_token, msg))
            }
        };
        let value = AstNumberValue {
            value: maybe_number?
        };

        let node = AstNodeType::NumberValue(Box::new(value));
        return Ok(node);
    }

    fn parse_string(&mut self) -> Result<AstNodeType, ParsingError> {
        assert_eq!(self.current_token.get_type(), StaticString);

        let text = self.current_token.get_text();
        let text_without_quotes = &text[1..text.len() - 1];
        let value = AstStringValue {
            value: String::from(text_without_quotes)
        };

        let node = AstNodeType::StringValue(Box::new(value));
        return Ok(node);
    }

    fn parse_partial_expression(&mut self) -> Result<AstNodeType, ParsingError> {
        let token = self.current_token;
        return match token.get_type() {
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
                self.next_token();
                let expr = self.parse_expression();
                self.next_token();

                if self.current_token.get_type() == CloseParenthesis {
                    expr
                } else {
                    let msg = format!("Missing closing parenthesis");
                    Err(ParsingError::new(token, msg))
                }
            }
            _ => {
                let msg = format!("Invalid token in expression");
                Err(ParsingError::new(self.current_token, msg))
            }
        };
    }

    fn parse_expression(&mut self) -> Result<AstNodeType, ParsingError> {
        let token = self.current_token;

        let evaluatable = self.parse_partial_expression();

        if let Some(token) = self.peek_token() {
            if token.get_type() == Operator {
                self.next_token();
                return self.parse_operator(evaluatable?);
            }
        }

        return evaluatable;
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
            return Ok(node);
        }

        let msg = format!("Unexpected character when parsing a typed assignment");
        return Err(ParsingError::new(self.current_token, msg));
    }

    fn parse_variable(&mut self) -> Result<AstNodeType, ParsingError> {
        assert_eq!(self.current_token.get_type(), Alphanumeric);

        let name = self.current_token.get_text();
        let variable = AstVariable {
            name: name
        };

        let node = AstNodeType::Variable(Box::new(variable));
        return Ok(node);
    }

    fn parse_named(&mut self) -> Result<AstNodeType, ParsingError> {
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
                    self.parse_variable()
                }
            };
        } else {
            let msg = format!("Unexpected end of stream when parsing a variable");
            return Err(ParsingError::new(self.current_token, msg));
        }
    }

    fn get_operator_precedence(&self, token: &Token) -> Result<usize, ParsingError> {
        assert_eq!(token.get_type(), Operator);

        let precedence = match token.get_type() {
            Operator => {
                match token.get_text().as_ref() {
                    "+" | "-" => {
                        1
                    }
                    "*" | "/" | "%" => {
                        2
                    }
                    "^" => {
                        3
                    }
                    _ => {
                        let msg = format!("Invalid operator: {}", token.get_text());
                        return Err(ParsingError::new(token, msg));
                    }
                }
            }
            OpenParenthesis => {
                4
            }
            _ => {
                let msg = format!("Invalid token after operator: ");
                return Err(ParsingError::new(token, msg));
            }
        };

        return Ok(precedence);
    }

    fn parse_operator(&mut self, lhs: AstNodeType) -> Result<AstNodeType, ParsingError> {
        let lhs_operator = self.current_token;
        assert_eq!(lhs_operator.get_type(), Operator);

        if let Some(rhs_token) = self.next_token() {

            let mut rhs = self.parse_partial_expression()?;
            if let Some(rhs_operator) = self.peek_token() {
                if rhs_operator.get_type() == Operator {
                    if self.get_operator_precedence(lhs_operator)? < self.get_operator_precedence(rhs_operator)? {
                        self.next_token();
                        rhs = self.parse_operator(rhs)?;
                    }
                }
            }

            let operator = AstOperator::from_token(lhs_operator);
            let call = AstOperatorCall {
                lhs: lhs,
                rhs: rhs,
                operator: operator
            };
            let node = AstNodeType::OperatorCall(Box::new(call));
            return Ok(node);
        }

        let msg = format!("Missing rhs operand");
        return Err(ParsingError::new(self.current_token, msg));
    }

    fn parse_statement(&mut self) -> Result<AstNodeType, ParsingError> {
        if let Some(token) = self.next_token() {
            let mut evaluatable = match token.get_type() {
                Alphanumeric => {
                    self.parse_named()
                }
                OpenBlock => {
                    return self.parse_block()
                }
                _ => {
                    self.parse_expression()
                }
            }?;

            if let Some(token) = self.next_token() {
                if token.get_type() == Operator {
                    evaluatable = self.parse_operator(evaluatable)?;
                }

                match token.get_type() {
                    EndOfStatement => {
                        return Ok(evaluatable);
                    }
                    _ => {
                        let msg = format!("Statements must end with a ; token");
                        return Err(ParsingError::new(self.current_token, msg));
                    }
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

            if token.get_type() == Comment {
                self.next_token();
                continue;
            }

            let evaluatable = self.parse_statement()?;
            block.statements.push(evaluatable);
        }

        let node = AstNodeType::Block(Box::new(block));
        return Ok(node);
    }

    fn parse(&mut self) -> Result<Ast, ParsingError> {
        let root = self.parse_block()?;
        let ast = Ast {
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

