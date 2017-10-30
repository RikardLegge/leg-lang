mod tokenizer;
mod file_info;
mod parser;

use tokenizer::tokenize;

fn main() {
    let script =   "message :String :: \"Hello \\\" World\"; print(message);";
    // func :: (a :int, b :String){ }
    // struct :: {}
    // array :: []

    match tokenize(script) {
        Ok(tokens) => {

            println!("Tokenization complete: {:?}", tokens);
        }
        Err(msg) => {
            println!("Something went wrong: '{}'", msg);
        }
    }
}



