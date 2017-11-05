mod tokenizer;
mod file_info;
mod parser;

use tokenizer::tokenize;
use parser::parse;

fn main() {
    let script =   "\
    {\
        message :String :: \"Hello \\\" World\";\
        print(message);\
    }";
    // func :: (a :int, b :String){ }
    // [] dot []
    // 10.dot ()
    // dot()
    // struct :: {}
    // array :: []

    let tokens = tokenize(script);
    println!("{:?}", tokens);

    let ast = parse(tokens);
    println!("{:?}", ast);
    println!("Compilation complete!");
}
