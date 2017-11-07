use interp::InterpValue;

pub fn print(arguments: Vec<InterpValue>) {
    use interp::InterpValue::*;
    for val in arguments {
        let string = match val {
            InterpVoid => {String::from("VOID")}
            InterpNumber(num) => {num.to_string()}
            InterpString(val) => {val}
            InterpFunction(i) => {format!("FUNCTION {}", i)}
            InterpStruct(i) =>{format!("STRUCT {}", i)}
        };
        println!("{}", string);
    }
}