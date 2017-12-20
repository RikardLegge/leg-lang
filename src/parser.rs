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
    FunctionDeclaration(Box<AstFunctionDeclaration>),
    StructDeclaration(Box<AstStructDeclaration>),
    Variable(Box<AstVariable>),
    Assignment(Box<AstAssignment>),
    Alias(Box<AstAlias>),
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
    pub body: Option<AstBlock>,
    pub next: Option<AstFunctionCall>
}

#[derive(Debug)]
pub struct AstOperatorCall {
    pub rhs: AstNodeType,
    pub lhs: AstNodeType,
    pub operator: AstOperator
}

#[derive(Debug)]
pub struct AstFunctionDeclaration {
    pub arguments: Vec<AstNodeType>,
    pub body: AstBlock
}

#[derive(Debug)]
pub struct AstStructDeclaration {
    pub fields: Vec<String>,
    pub types: Vec<String>
}

#[derive(Debug)]
pub struct AstStringValue {
    pub value: String
}

#[derive(Debug)]
pub struct AstNumberValue {
    pub value: f64
}

#[derive(Debug)]
pub struct AstVariable {
    pub name: String
}

#[derive(Debug)]
pub struct AstAssignment {
    pub to: AstVariable,
    pub from: AstNodeType
}

#[derive(Debug)]
pub struct AstAlias {
    pub to: AstVariable,
    pub from: AstNodeType
}

pub struct Parser<'a> {
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
        let evaluatable = self.parse_partial_expression();

        if let Some(token) = self.peek_token() {
            if token.get_type() == Operator {
                self.next_token();
                return self.parse_operator(evaluatable?);
            }
        }

        return evaluatable;
    }

    fn parse_function_declaration(&mut self) -> Result<AstNodeType, ParsingError> {
        assert_eq!(self.current_token.get_type(), OpenParenthesis);

        let mut arguments: Vec<AstNodeType> = Vec::new();
        while let Some(token) = self.next_token() {
            if token.get_type() == CloseParenthesis {
                break;
            }

            if token.get_type() != Alphanumeric {
                let msg = format!("Unexpected character when parsing function declaration");
                return Err(ParsingError::new(self.current_token, msg));
            }
            let argument_name = token.get_text();
            let argument = AstVariable {
                name: argument_name,
            };
            let node = AstNodeType::Variable(Box::new(argument));
            arguments.push(node);

            if let Some(next) = self.peek_token() {
               if next.get_type() == ParameterDivider {
                   self.next_token();
                   continue;
               }
                if next.get_type() == CloseParenthesis {
                    continue
                }
            }

            let msg = format!("Unexpected character when parsing function declaration arguments");
            return Err(ParsingError::new(self.current_token, msg));
        }

        self.next_token();
        let body = self.parse_block_raw()?;

        let function = AstFunctionDeclaration {
            arguments: arguments,
            body: body,
        };
        let node = AstNodeType::FunctionDeclaration(Box::new(function));
        return Ok(node);
    }

    fn parse_struct_declaration(&mut self)  -> Result<AstNodeType, ParsingError> {
        assert_eq!(self.current_token.get_type(), OpenBlock);

        let mut fields: Vec<String> = Vec::new();
        let mut types: Vec<String> = Vec::new();
        while let Some(token) = self.next_token() {
            if token.get_type() == CloseBlock {
                break;
            }

            let field_name_token = token;
            if field_name_token.get_type() != Alphanumeric {
                let msg = format!("Unexpected character when parsing struct declaratoin, Alphanumeric expected");
                return Err(ParsingError::new(self.current_token, msg));
            }
            let field_name = field_name_token.get_text();
            fields.push(field_name);

            if self.next_token().unwrap().get_type() != Symbol {
                let msg = format!("Unexpected character when parsing struct declaration, Symbol expected");
                return Err(ParsingError::new(self.current_token, msg));
            }

            let field_type_token = self.next_token().unwrap();
            if field_type_token.get_type() != Alphanumeric {
                let msg = format!("Unexpected character when parsing struct declaration, Alphanumeric expected");
                return Err(ParsingError::new(self.current_token, msg));
            }
            let field_type = field_type_token.get_text();
            fields.push(field_type);
        }
        let structure = AstStructDeclaration {
            fields: fields,
            types: types,
        };
        let node = AstNodeType::StructDeclaration(Box::new(structure));
        return Ok(node);
    }

    fn parse_static_expression(&mut self) -> Result<AstNodeType, ParsingError> {
        let token = self.current_token;
        return match token.get_type() {
            OpenParenthesis => {
                self.parse_function_declaration()
            }
            OpenBlock => {
                self.parse_struct_declaration()
            }
            Alphanumeric | Numeric | StaticString => {
                self.parse_expression()
            }
            _ => {
                let msg = format!("Invalid token in expression");
                Err(ParsingError::new(self.current_token, msg))
            }
        };
    }

    fn parse_assignment(&mut self) -> Result<AstNodeType, ParsingError> {
        assert_eq!(self.current_token.get_type(), Alphanumeric);

        let variable_name = self.current_token.get_text();
        let mut variable_type: Option<String> = None;

        let maybe_type_token = self.peek_token().unwrap();
        if maybe_type_token.get_type() == Symbol {
            variable_type = Some(maybe_type_token.get_text());
            self.next_token();
        }

        let assignment_type_token = self.next_token().unwrap();
        return match assignment_type_token.get_type() {
            StaticAssignment => {
                // Struct or function
                let variable = AstVariable {
                    name: variable_name
                };

                self.next_token();
                let expression = self.parse_static_expression()?;
                let alias = AstAlias {
                    to: variable,
                    from: expression,
                };

                let node = AstNodeType::Alias(Box::new(alias));
                Ok(node)
            }
            VariableAssignment => {
                // Variable or expression
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
                Ok(node)
            }
            _ => {
                let msg = format!("Unexpected character when parsing an assignment");
                Err(ParsingError::new(self.current_token, msg))
            }
        }
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

                if let Some(next) = self.peek_token() {
                    if next.get_type() == ParameterDivider {
                        self.next_token();
                        continue
                    }
                    if next.get_type() == CloseParenthesis {
                        continue;
                    }
                }

                let msg = format!("Unexpected character when parsing function call arguments");
                return Err(ParsingError::new(self.current_token, msg));
            }

            let body = match self.peek_token() {
                Some(token) => {
                    match token.get_type() {
                        OpenBlock => {
                            self.next_token();
                            let block = self.parse_block_raw()?;
                            Some(block)
                        },
                        _ => {None}
                    }
                }
                None => {None}
            };

            let call = AstFunctionCall {
                name: function_name,
                arguments: arguments,
                body: body,
                next: None
            };

            let node = AstNodeType::FunctionCall(Box::new(call));
            return Ok(node);
        }

        let msg = format!("Unexpected character when parsing function call");
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
                Symbol | VariableAssignment | StaticAssignment => {
                    self.parse_assignment()
                }
                OpenParenthesis => {
                    self.parse_function_call()
                }
                _ => {
                    self.parse_expression()
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
                    return self.parse_block();
                }
                EndOfStatement => {
                    let null = AstNullValue {};
                    return Ok(AstNodeType::NullValue(Box::new(null)));
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

    fn parse_block_raw(&mut self) -> Result<AstBlock, ParsingError> {
        if !self.current_token.is_null() {
            assert_eq!(self.current_token.get_type(), OpenBlock);
        }
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
        return Ok(block);
    }

    fn parse_block(&mut self) -> Result<AstNodeType, ParsingError> {
        let block = self.parse_block_raw()?;
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
    let null_token = Token::null();

    let mut iter = tokens.iter().peekable();

    let mut parser = Parser {
        token_buffer: Vec::new(),
        token_stream: iter,
        current_token: &null_token
    };
    return parser.parse();
}

