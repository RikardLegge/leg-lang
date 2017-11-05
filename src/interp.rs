use parser::{Ast, AstNodeType};
use std::collections::HashMap;
use std::mem;

use std::fmt;
use std::error::Error;
use std::fmt::Display;
use std::fmt::Formatter;

use leg_sdl;
use operators;

#[derive(Debug)]
pub struct InterpError {
    desc: String
}

impl Display for InterpError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        writeln!(f, "InterpError: \n{}", self.desc)
    }
}

impl InterpError {
    pub fn new(desc: String) -> InterpError {
        return InterpError {
            desc: desc
        };
    }
}

impl Error for InterpError {
    fn description(&self) -> &str {
        "Interpreter error!!!"
    }

    fn cause(&self) -> Option<&Error> {
        None
    }
}

#[derive(Clone, Debug)]
pub enum InterpValue {
    InterpVoid,
    InterpNumber(f64),
    InterpString(String),
    InterpFunction,
    InterpStruct
}

struct StackFrame {
    scope: HashMap<String, InterpValue>
}

impl StackFrame {
    fn new() -> StackFrame {
        return StackFrame {
            scope: HashMap::new()
        };
    }
}

struct Interp {
    stack: Vec<StackFrame>,
    current_frame: StackFrame
}

impl Interp {
    fn evaluate_next(&mut self, node: AstNodeType) -> Result<InterpValue, InterpError> {
        match node {
            AstNodeType::Block(boxed) => {
                let block = *boxed;

                let frame = mem::replace(&mut self.current_frame, StackFrame::new());
                self.stack.push(frame);

                let mut last_result: InterpValue = InterpValue::InterpVoid;

                for statement in block.statements {
                    last_result = self.evaluate_next(statement)?;
                }

                if let Some(frame) = self.stack.pop() {
                    self.current_frame = frame;
                    return Ok(last_result);
                } else {
                    let msg = format!("Unable to pop from stack");
                    return Err(InterpError::new(msg));
                }
            }
            AstNodeType::FunctionCall(boxed) => {
                let function = *boxed;

                let mut args: Vec<InterpValue> = Vec::with_capacity(function.arguments.len());
                for arg in function.arguments {
                    let val = self.evaluate_next(arg)?;
                    args.push(val);
                }

                if function.name == "print" {
                    leg_sdl::print(args);
                }
            }
            AstNodeType::StringValue(boxed) => {
                let string = *boxed;
                let value = string.value.clone();

                return Ok(InterpValue::InterpString(value));
            }
            AstNodeType::NumberValue(boxed) => {
                let number = *boxed;

                return Ok(InterpValue::InterpNumber(number.value));
            }
            AstNodeType::Variable(boxed) => {
                let variable = *boxed;
                let name = variable.name;

                if let Some(value) = self.current_frame.scope.get(&name) {
                    return Ok(value.clone());
                } else {
                    let msg = format!("Unable to find variable {}", name);
                    return Err(InterpError::new(msg));
                }
            }
            AstNodeType::Assignment(boxed) => {
                let assignment = *boxed;
                let name = assignment.to.name;
                let value = self.evaluate_next(assignment.from)?;

                self.current_frame.scope.insert(name, value);
            }
            AstNodeType::OperatorCall(boxed) => {
                let operation = *boxed;
                let operator = operation.operator;
                let lhs = self.evaluate_next(operation.lhs)?;
                let rhs = self.evaluate_next(operation.rhs)?;

                return operators::apply_operation(lhs, rhs, operator);
            }
            _ => {
                let msg = format!("Unable to intepret AstNode: {:?}", node);
                return Err(InterpError::new(msg));
            }
        }

        return Ok(InterpValue::InterpVoid);
    }
}

pub fn interp(ast: Ast) -> Result<InterpValue, InterpError> {
    let base_stack_frame = StackFrame::new();

    let mut interp = Interp {
        stack: Vec::with_capacity(100),
        current_frame: base_stack_frame
    };
    return interp.evaluate_next(ast.root);
}