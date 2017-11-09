use interp::InterpValue;

pub fn print(arguments: Vec<InterpValue>) {
    use interp::InterpValue::*;
    for val in arguments {
        let string = match val {
            InterpVoid => {String::from("VOID")}
            InterpBoolean(val) => {format!("BOOLEAN {{{}}}", val)}
            InterpNumber(num) => {num.to_string()}
            InterpString(val) => {val}
            InterpFunction{id, closure_id} => {format!("FUNCTION {}", id)}
            InterpStruct(i) =>{format!("STRUCT {}", i)}
        };
        println!("{}", string);
    }
}