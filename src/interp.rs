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
    InterpBoolean(bool),
    InterpString(String),
    InterpStruct(usize),
    InterpFunction{id: usize, closure_id: usize }
}

impl InterpValue {
    fn evals_to_true(&self) ->bool {
        return match self {
            &InterpValue::InterpNumber(num) => {
                num != 0.0
            },
            _ => {false}
        }
    }
}

struct InterpStruct {
    pub fields: Vec<String>,
    pub types: Vec<String>
}

struct StackFrame<'a> {
    index: usize,
    creator: &'a AstNodeType,
    closure_id: usize,
    parent_frame: Option<Box<StackFrame<'a>>>
}

impl <'a>StackFrame<'a> {
    fn new(creator: &'a AstNodeType, closure: usize) -> StackFrame<'a> {
        return StackFrame {
            index: 0,
            creator: creator,
            closure_id: closure,
            parent_frame: None
        };
    }

    fn set_parent_frame(&mut self, parent_frame: StackFrame<'a>) {
        let parent_frame_index = parent_frame.index;
        let new_frame = Some(Box::new(parent_frame));
        mem::replace(&mut self.parent_frame, new_frame);

        self.index = parent_frame_index+1;
    }

    fn remove_parent_frame(&mut self) -> Option<StackFrame<'a>> {
        let parent_frame = mem::replace(&mut self.parent_frame, None);

        if let Some(frame) = parent_frame {
            let frame = Some(*frame);
            return frame;
        } else {
            return None;
        }
    }
}

struct Closure<'a> {
    creator: &'a AstNodeType,
    variables: HashMap<String, InterpValue>,
    parent_id: Option<usize>
}

impl <'a>Closure<'a> {
    fn new(creator: &'a AstNodeType, parent_closure: Option<usize>) -> Closure<'a> {
        return Closure {
            creator: creator,
            variables: HashMap::new(),
            parent_id: parent_closure
        };
    }

    fn set_variable(&mut self, name: String, value: InterpValue) {

    }
}



struct Interp<'a> {
    stack_size: usize,
    structs: Vec<&'a AstStructDeclaration>,
    functions: Vec<&'a AstFunctionDeclaration>,
    closures: Vec<Option<Closure<'a>>>,
    current_frame: StackFrame<'a>
}

impl <'a>Interp<'a> {
    fn get_closure_by_id(&self, id: usize) -> Result<&Closure<'a>, InterpError> {
        return self.closures[id].as_ref().ok_or({
            let msg = format!("The closure with id {} no longer exists", id);
            InterpError::new(msg)
        });
    }

    fn get_mut_closure_by_id(&mut self, id: usize) -> Result<&mut Closure<'a>, InterpError> {
        return self.closures[id].as_mut().ok_or({
            let msg = format!("The closure with id {} no longer exists", id);
            InterpError::new(msg)
        });
    }

    fn get_current_closure(&self) -> Result<&Closure<'a>, InterpError>{
        let id = self.current_frame.closure_id;
        return self.get_closure_by_id(id);
    }

    fn get_current_mut_closure(&mut self) -> Result<&mut Closure<'a>, InterpError> {
        let id = self.current_frame.closure_id;
        return self.get_mut_closure_by_id(id);
    }

    fn get_variable_of_closure(&self, name: &String, closure: &'a Closure) -> Result<&InterpValue, InterpError> {
        if let Some(interpValue) = closure.variables.get(name) {
            return Ok(interpValue);
        } else if let Some(parent_id) = closure.parent_id {
            let parent_closure = self.get_closure_by_id(parent_id)?;
            return self.get_variable_of_closure(name, parent_closure);
        } else {
            let msg = format!("Unable to find variable {}", name);
            return Err(InterpError::new(msg));
        }
    }

    fn get_variable(&self, name: &String) -> Result<&InterpValue, InterpError> {
        let closure = self.get_current_closure()?;
        return self.get_variable_of_closure(name, closure);
    }

    fn set_variable(&mut self, name: String, value: InterpValue) -> Result<InterpValue, InterpError> {
        let closure = self.get_current_mut_closure()?;
        closure.variables.insert(name, value);

        return Ok(InterpValue::InterpVoid);
    }

    fn push_frame(&mut self, creator: &'a AstNodeType, closure_id: usize) -> Result<InterpValue, InterpError> {
        if self.current_frame.index > self.stack_size {
            let msg = format!("Stack overflow!");
            return Err(InterpError::new(msg));
        }

        let mut new_frame = StackFrame::new(creator, closure_id);
        let frame = mem::replace(&mut self.current_frame, new_frame);
        self.current_frame.set_parent_frame(frame);

        return Ok(InterpValue::InterpVoid);
    }

    fn pop_frame(&mut self) -> Result<StackFrame<'a>, InterpError> {
        let parent_frame = self.current_frame.remove_parent_frame();

        if let Some(frame) = parent_frame {
            let old_frame = mem::replace(&mut self.current_frame, frame);
            return Ok(old_frame);
        } else {
            let msg = format!("Unable to pop from stack");
            return Err(InterpError::new(msg));
        }
    }

    fn add_closure(&mut self, creator: &'a AstNodeType, parent_closure_id: usize) -> usize {
        let closure = Closure::new(creator, Some(parent_closure_id));
        let id = self.closures.len();
        self.closures.push(Some(closure));
        return id;
    }

    fn evaluate_block(&mut self, creator: &'a AstNodeType, block: &'a AstBlock) -> Result<InterpValue, InterpError> {
        let name = "A block";
        let parent_closure_id = self.current_frame.closure_id;
        let closure_id = self.add_closure(creator, parent_closure_id);

        self.push_frame(creator, closure_id)?;
        let res = {
            let mut last_result: InterpValue = InterpValue::InterpVoid;

            for statement in &block.statements {
                last_result = self.evaluate_next(&statement)?;
            }

            last_result
        };
        self.pop_frame()?;

        return Ok(res);
    }

    fn evaluate_next(&mut self, node: &'a AstNodeType) -> Result<InterpValue, InterpError> {
        match node {
            &AstNodeType::Block(ref boxed) => {
                let block = &**boxed;
                return self.evaluate_block(node, block);
            }
            &AstNodeType::FunctionCall(ref boxed) => {
                let function = &**boxed;

                let mut args: Vec<InterpValue> = Vec::with_capacity(function.arguments.len());
                for arg in &function.arguments {
                    let val = self.evaluate_next(&arg)?;
                    args.push(val);
                }

                let name = &function.name;
                if name == "while" {
                    
                } else if name == "if" {
                    if args.len() != 1 {
                        let msg = format!("if statements can only have one parameter");
                        return Err(InterpError::new(msg));
                    }

                    if let Some(ref body) = function.body {
                        let is_true = args[0].evals_to_true();

                        if is_true {
                            return self.evaluate_block(node, body);
                        } else {
                            return Ok(InterpValue::InterpVoid);
                        }
                    } else {
                        let msg = format!("If statement must have a body");
                        return Err(InterpError::new(msg));
                    }
                } else if name == "print" {
                    leg_sdl::print(args);
                    return Ok(InterpValue::InterpVoid);
                } else {
                    let mut maybe_index = {
                        let interp_value = self.get_variable(name)?;
                        match *interp_value {
                            InterpValue::InterpFunction{id, closure_id} => {
                                Some((id, closure_id))
                            }
                            _ => {
                                None
                            }
                        }
                    };

                    if let Some((function_id, closure_id)) = maybe_index {
                        let func :&AstFunctionDeclaration = self.functions.get(function_id).unwrap();

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

                        self.push_frame(node, closure_id)?;
                        for name_value in argument_names.iter().zip(&args) {
                            let name = String::from(*name_value.0);
                            let value: InterpValue = name_value.1.clone();
                            self.set_variable(name, value);
                        }

                        let res = self.evaluate_block(node,&func.body);
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

                let val = self.get_variable(name)?;
                return Ok(val.clone());
            }
            &AstNodeType::Assignment(ref boxed) => {
                let assignment = &**boxed;
                let name = assignment.to.name.clone();
                let value = self.evaluate_next(&assignment.from)?;

                self.set_variable(name, value);
                return Ok(InterpValue::InterpVoid);
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
                return Ok(InterpValue::InterpVoid);
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

                let parent_closure_id = self.current_frame.closure_id;
                let closure_id = self.add_closure(node, parent_closure_id);

                return Ok(InterpValue::InterpFunction{id: index, closure_id: closure_id});
            }
            &AstNodeType::NullValue(ref boxed) => {
                return Ok(InterpValue::InterpVoid);
            }
            _ => {
                let msg = format!("Unable to intepret AstNode: {:?}", node);
                return Err(InterpError::new(msg));
            }
        }
    }
}

pub fn interp(ast: Ast) -> Result<InterpValue, InterpError> {
    let root_expr = &ast.root;

    let mut closures: Vec<Option<Closure>> = Vec::new();
    let base_closure = Closure::new(root_expr, None);
    let base_closure_id = closures.len();
    closures.push(Some(base_closure));

    let base_stack_frame = StackFrame::new(root_expr, base_closure_id);

    let mut interp = Interp {
        stack_size: 10,
        functions: Vec::new(),
        structs: Vec::new(),
        closures: closures,
        current_frame: base_stack_frame
    };
    return interp.evaluate_next(root_expr);
}