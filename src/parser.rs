use tokenizer::Token;
use tokenizer::TokenType;
use tokenizer::TokenType::*;
use std::slice::Iter;
use std::iter::Peekable;
use std::fmt;
use std::ops::Deref;
use std::fmt::Debug;

trait Evaluatable : Debug {
    fn evaluate(&self) -> i32;
}

#[derive(Debug)]
pub struct Ast {
    root: BlockNode
}

struct BlockNode {
    statements: Vec<Box<Evaluatable>>
}

impl Evaluatable for BlockNode {
    fn evaluate(&self) -> i32 {
        unimplemented!()
    }
}

impl fmt::Debug for BlockNode {
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

impl BlockNode {
    fn new() -> BlockNode {
        return BlockNode {
            statements: Vec::new()
        };
    }
}

#[derive(Debug)]
struct FunctionCallNode {
    name: String,
    arguments: Vec<Box<Evaluatable>>,
    body: Option<BlockNode>
}

impl Evaluatable for FunctionCallNode {
    fn evaluate(&self) -> i32 {
        unimplemented!()
    }
}

#[derive(Debug)]
struct StringValueNode {
    value: String
}

impl Evaluatable for StringValueNode {
    fn evaluate(&self) -> i32 {
        unimplemented!()
    }
}

#[derive(Debug)]
struct NumberValueNode {
    value: String
}

impl Evaluatable for NumberValueNode {
    fn evaluate(&self) -> i32 {
        unimplemented!()
    }
}

#[derive(Debug)]
struct VariableNode {
    name: String
}

impl Evaluatable for VariableNode {
    fn evaluate(&self) -> i32 {
        unimplemented!()
    }
}

#[derive(Debug)]
struct AssignmentNode {
    to: VariableNode,
    from: Box<Evaluatable>
}

impl Evaluatable for AssignmentNode {
    fn evaluate(&self) -> i32 {
        unimplemented!()
    }
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
        }
    }

    fn peek_token(&mut self) -> Option<&'a Token> {
        return match self.token_stream.peek() {
            Some(token) => {
                Some(*token)
            }
            None => {
                None
            }
        }
    }

    fn parse_number(&mut self) -> NumberValueNode {
        unimplemented!()
    }

    fn parse_string(&mut self) -> StringValueNode {
        unimplemented!()
    }

    fn parse_expression(&mut self) -> Box<Evaluatable> {
        match self.current_token.get_type() {
            StaticString => {
                let value = self.current_token.get_text();
                return Box::new(StringValueNode {
                    value: value
                });
            }
            Alphanumeric => {
                let name = self.current_token.get_text();
                return Box::new(VariableNode {
                    name: name
                });
            }
            _ => {
                self.report_error(self.current_token, "Unexpected character when parsing an expression");
                unreachable!();
            }
        }

        self.report_error(self.current_token, "Unexpected character when parsing an expression");
        unreachable!();
    }

    fn parse_assignment(&mut self) -> AssignmentNode {
        unimplemented!()
    }

    fn parse_typed_assignment(&mut self) -> AssignmentNode {
        assert_eq!(self.current_token.get_type(), Alphanumeric);
        let variable_name = self.current_token.get_text();
        if let Some(symbol_start_token) = self.next_token() {
            assert_eq!(symbol_start_token.get_type(), Symbol);
            if let Some(symbol_token) = self.next_token() {
                assert_eq!(symbol_token.get_type(), Alphanumeric);
                if let Some(assignment_token) = self.next_token() {
                    let assignment_type = assignment_token.get_type();
                    let variable = VariableNode {
                        name: variable_name
                    };

                    self.next_token();
                    let expression = self.parse_expression();
                    return AssignmentNode {
                        to: variable,
                        from: expression
                    }
                }
            }

        }

        self.report_error(self.current_token, "Unexpected character when parsing a typed assignment");
        unreachable!();
    }

    fn parse_function_call(&mut self) -> FunctionCallNode {
        assert_eq!(self.current_token.get_type(), Alphanumeric);
        let function_name = self.current_token.get_text();
        if let Some(function_args_start) = self.next_token() {
            assert_eq!(self.current_token.get_type(), OpenParenthesis);

            let mut arguments: Vec<Box<Evaluatable>> = Vec::new();
            while let Some(token) = self.next_token() {
                if token.get_type() == CloseParenthesis {
                    break;
                }

                let expression = self.parse_expression();
                arguments.push(expression);
            }

            return FunctionCallNode {
                name: function_name,
                arguments: arguments,
                body: None
            }
        }

        self.report_error(self.current_token, "Unexpected character when parsing a typed assignment");
        unreachable!();
    }

    fn parse_variable(&mut self) -> Box<Evaluatable> {
        if let Some(token) = self.peek_token() {
            return match token.get_type() {
                Symbol => {
                    Box::new(self.parse_typed_assignment())
                }
                DynamicAssignment | StaticAssignment => {
                    Box::new(self.parse_assignment())
                }
                OpenParenthesis => {
                    Box::new(self.parse_function_call())
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

    fn parse_statement(&mut self) -> Box<Evaluatable> {
        if let Some(token) = self.next_token() {
            let evaluatable = match token.get_type() {
                Alphanumeric => {
                    self.parse_variable()
                }
                Numeric => {
                    Box::new(self.parse_number())
                }
                StaticString => {
                    Box::new(self.parse_string())
                }
                OpenParenthesis => {
                    self.parse_expression()
                }
                OpenBlock => {
                    Box::new(self.parse_block())
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

    fn parse_block(&mut self) -> BlockNode {
        let mut block = BlockNode::new();

        while let Some(token) = self.peek_token() {
            if token.get_type() == CloseBlock {
                self.next_token();
                break;
            }

            let evaluatable = self.parse_statement();
            block.statements.push(evaluatable);
        }

        return block;
    }

    fn parse_root_block(&mut self) -> BlockNode {
        let mut block = BlockNode::new();

        while let Some(token) = self.next_token() {
            match token.get_type() {
                OpenBlock => {
                    let child_block = self.parse_block();
                    block.statements.push(Box::new(child_block));
                }
                _ => {self.report_error(token, "Invalid end of input, file must start with a root block")}
            }
        }

        return block;
    }

    fn parse(&mut self) -> Ast {
        return Ast {
            root: self.parse_root_block()
        };
    }
}

pub fn parse(tokens: Vec<Token>) -> Ast {
    let null_token = Token::new();
    let mut iter = tokens.iter().peekable();

    let mut parser = Parser {
        token_buffer: Vec::new(),
        token_stream: iter,
        current_token: &null_token
    };
    return parser.parse();
}

