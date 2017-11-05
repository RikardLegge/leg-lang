use parser::{Ast, AstNodeType};
use std::collections::HashMap;
use std::mem;

#[derive(Clone)]
enum InterpValue {
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
    fn evaluate_next(&mut self, node: AstNodeType) -> InterpValue {
        match node {
            AstNodeType::Block(boxed) => {
                let block = *boxed;

                let frame = mem::replace(&mut self.current_frame, StackFrame::new());
                self.stack.push(frame);

                for statement in block.statements {
                    self.evaluate_next(statement);
                }

                if let Some(frame) = self.stack.pop() {
                    self.current_frame = frame;
                } else {
                    panic!("Unable to pop stack");
                }
            }
            AstNodeType::FunctionCall(boxed) => {
                let function = *boxed;

                if function.name == "print" {
                    for arg in function.arguments {
                        let val = self.evaluate_next(arg);

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

                return InterpValue::String(value);
            }
            AstNodeType::NumberValue(boxed) => {
                let number = *boxed;
            }
            AstNodeType::Variable(boxed) => {
                let variable = *boxed;
                let name = variable.name;

                if let Some(value) = self.current_frame.scope.get(&name) {
                    return value.clone();
                } else {
                    panic!("Unable to find variable {}", name);
                }
            }
            AstNodeType::Assignment(boxed) => {
                let assignment = *boxed;
                let name = assignment.to.name;
                let value = self.evaluate_next(assignment.from);

                self.current_frame.scope.insert(name, value);
            }
        }

        return InterpValue::Void;
    }
}

pub fn interp(ast: Ast) {
    let base_stack_frame = StackFrame::new();

    let mut interp = Interp {
        stack: Vec::new(),
        current_frame: base_stack_frame
    };
    interp.evaluate_next(ast.root);
}