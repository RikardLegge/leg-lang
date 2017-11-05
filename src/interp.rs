use parser::{Ast, AstNodeType};
use std::collections::HashMap;
use std::mem;

use std::fmt;
use std::error::Error;
use std::fmt::Display;
use std::fmt::Formatter;

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
    fn new(desc: String) -> InterpError {
        return InterpError {
            desc: desc
        }
    }
}

impl Error for InterpError {
    fn description(&self) -> &str{
        "Interpreter error!!!"
    }

    fn cause(&self) -> Option<&Error> {
        None
    }
}

#[derive(Clone, Debug)]
pub enum InterpValue {
    Void,
    Number(f64),
    String(String),
    Function,
    Struct
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

                let mut last_result: InterpValue = InterpValue::Void;

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

                if function.name == "print" {
                    for arg in function.arguments {
                        let val = self.evaluate_next(arg)?;

                        let string = match val {
                            InterpValue::Void => {String::from("VOID")}
                            InterpValue::Number(num) => {num.to_string()}
                            InterpValue::String(val) => {val},
                            InterpValue::Function => {String::from("FUNCTION")}
                            InterpValue::Struct =>{String::from("STRUCT")}
                        };
                        println!("{}", string);
                    }
                }
            }
            AstNodeType::StringValue(boxed) => {
                let string = *boxed;
                let value = string.value.clone();

                return Ok(InterpValue::String(value));
            }
            AstNodeType::NumberValue(boxed) => {
                let number = *boxed;
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
        }

        return Ok(InterpValue::Void);
    }
}

pub fn interp(ast: Ast) -> Result<InterpValue, InterpError> {
    let base_stack_frame = StackFrame::new();

    let mut interp = Interp {
        stack: Vec::new(),
        current_frame: base_stack_frame
    };
    return interp.evaluate_next(ast.root);
}