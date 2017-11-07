use parser::{Ast, AstNodeType, AstFunctionDeclaration, AstStructDeclaration, AstBlock};
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
    InterpStruct(usize),
    InterpFunction(usize)
}

struct InterpStruct {
    pub fields: Vec<String>,
    pub types: Vec<String>
}

struct StackFrame {
    scope: HashMap<String, InterpValue>,
}

struct Closure<'a> {
    parent_closure: Option<&'a Closure<'a>>,
    variables: HashMap<String, InterpValue>
}

impl <'a>Closure<'a> {
    fn new() -> Closure<'a> {
        return Closure {
            parent_closure: None,
            variables: HashMap::new()
        };
    }

    fn get_variable(&self, name: &String) -> Result<&InterpValue, InterpError> {
        if let Some(interpValue) = self.variables.get(name) {
            return Ok(interpValue);
        } else if let Some(parent_closure) = self.parent_closure {
            return parent_closure.get_variable(name);
        } else {
            let msg = format!("Unable to find variable {}", name);
            return Err(InterpError::new(msg));
        }
    }
}

impl StackFrame {
    fn new() -> StackFrame {
        return StackFrame {
            scope: HashMap::new()
        };
    }
}

struct Interp<'a> {
    structs: Vec<&'a AstStructDeclaration>,
    functions: Vec<&'a AstFunctionDeclaration>,
    stack: Vec<StackFrame>,
    current_frame: StackFrame,
    current_closure: Closure<'a>
}

impl <'a>Interp<'a> {

    fn get_variable(&self, name: &String) -> Result<&InterpValue, InterpError> {
        if let Some(interpValue) = self.current_frame.scope.get(name) {
            return Ok(interpValue);
        } else {
            let msg = format!("Unable to find variable {}", name);
            return Err(InterpError::new(msg));
        }
    }

    fn set_variable(&mut self, name: String, value: InterpValue) -> Result<InterpValue, InterpError> {
        self.current_frame.scope.insert(name, value);
        return Ok(InterpValue::InterpVoid);
    }

    fn push_frame(&mut self, new_frame: StackFrame) {
        let frame = mem::replace(&mut self.current_frame, new_frame);
        self.stack.push(frame);
    }

    fn pop_frame(&mut self) -> Result<StackFrame, InterpError> {
        if let Some(frame) = self.stack.pop() {
            let old_frame = mem::replace(&mut self.current_frame, frame);
            return Ok(old_frame);
        } else {
            let msg = format!("Unable to pop from stack");
            return Err(InterpError::new(msg));
        }
    }

    fn evaluate_block(&mut self, block: &'a AstBlock) -> Result<InterpValue, InterpError> {
        let mut last_result: InterpValue = InterpValue::InterpVoid;

        for statement in &block.statements {
            last_result = self.evaluate_next(&statement)?;
        }

        return Ok(last_result);
    }

    fn evaluate_next(&mut self, node: &'a AstNodeType) -> Result<InterpValue, InterpError> {
        match node {
            &AstNodeType::Block(ref boxed) => {
                let block = &**boxed;

                self.push_frame(StackFrame::new());
                let res = self.evaluate_block(block);
                self.pop_frame();
                return res;
            }
            &AstNodeType::FunctionCall(ref boxed) => {
                let function = &**boxed;

                let mut args: Vec<InterpValue> = Vec::with_capacity(function.arguments.len());
                for arg in &function.arguments {
                    let val = self.evaluate_next(&arg)?;
                    args.push(val);
                }

                let name = &function.name;
                if name == "print" {
                    leg_sdl::print(args);
                    return Ok(InterpValue::InterpVoid);
                } else {
                    let mut maybe_index = {
                        let interp_value = self.get_variable(name)?;
                        match *interp_value {
                            InterpValue::InterpFunction(function_index) => {
                                Some(function_index)
                            }
                            _ => {
                                None
                            }
                        }
                    };

                    if let Some(index) = maybe_index {
                        let func :&AstFunctionDeclaration = self.functions.get(index).unwrap();

                        let mut argument_names :Vec<&str> = Vec::new();
                        for arg in &func.arguments {
                            match arg {
                                &AstNodeType::Variable(ref boxed) => {
                                    let variable = &**boxed;
                                    let name = &variable.name[..];
                                    argument_names.push(name);
                                }
                                _ => {
                                    let msg = format!("Invalid argument expression");
                                    return Err(InterpError::new(msg));
                                }
                            }
                        }

                        assert_eq!(argument_names.len(), args.len());

                        let mut frame = StackFrame::new();
                        for name_value in argument_names.iter().zip(&args) {
                            let name = String::from(*name_value.0);
                            let value: InterpValue = name_value.1.clone();
                            frame.scope.insert(name, value);
                        }

                        self.push_frame(frame);
                        let res = self.evaluate_block(&func.body);
                        self.pop_frame();

                        return res;
                    }
                }

                let msg = format!("Unable to find function {}", name);
                return Err(InterpError::new(msg));
            }
            &AstNodeType::StringValue(ref boxed) => {
                let string = &**boxed;
                let value = string.value.clone();

                return Ok(InterpValue::InterpString(value));
            }
            &AstNodeType::NumberValue(ref boxed) => {
                let number = &**boxed;

                return Ok(InterpValue::InterpNumber(number.value));
            }
            &AstNodeType::Variable(ref boxed) => {
                let variable = &**boxed;
                let name = &variable.name;

                if let Some(value) = self.current_frame.scope.get(name) {
                    return Ok(value.clone());
                } else {
                    let msg = format!("Unable to find variable {}", name);
                    return Err(InterpError::new(msg));
                }
            }
            &AstNodeType::Assignment(ref boxed) => {
                let assignment = &**boxed;
                let name = assignment.to.name.clone();
                let value = self.evaluate_next(&assignment.from)?;

                self.set_variable(name, value);
            }
            &AstNodeType::OperatorCall(ref boxed) => {
                let operation = &**boxed;
                let operator = operation.operator;
                let lhs = self.evaluate_next(&operation.lhs)?;
                let rhs = self.evaluate_next(&operation.rhs)?;

                return operators::apply_operation(lhs, rhs, operator);
            }
            &AstNodeType::Alias(ref boxed) => {
                let alias = &**boxed;
                let name = alias.to.name.clone();
                let value = self.evaluate_next(&alias.from)?;

                self.set_variable(name, value);
            }
            &AstNodeType::StructDeclaration(ref boxed) => {
                let dec = &**boxed;

                let index = self.structs.len();
                self.structs.push(&dec);

                return Ok(InterpValue::InterpStruct(index));
            }
            &AstNodeType::FunctionDeclaration(ref boxed) => {
                let dec = &**boxed;

                let index = self.functions.len();
                self.functions.push(dec);

                return Ok(InterpValue::InterpFunction(index));
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
    let base_closure = Closure::new();

    let mut interp = Interp {
        functions: Vec::new(),
        structs: Vec::new(),
        stack: Vec::with_capacity(100),
        current_frame: base_stack_frame,
        current_closure: base_closure
    };
    return interp.evaluate_next(&ast.root);
}