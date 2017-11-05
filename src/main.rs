mod tokenizer;
mod file_info;
mod parser;
mod interp;
mod operators;
mod leg_sdl;

use tokenizer::tokenize;
use parser::parse;
use interp::interp;

use std::fs::File;
use std::io::BufReader;
use std::io::prelude::*;

use std::io;
use std::fs;
use std::path::PathBuf;

fn read_script_from_file() -> Result<String, io::Error> {
    let srcdir = PathBuf::from("./hello_world.leg");
    println!("{:?}", fs::canonicalize(&srcdir));

    let file = File::open(srcdir)?;
    let mut buf_reader = BufReader::new(file);
    let mut contents = String::new();
    buf_reader.read_to_string(&mut contents)?;

    Ok(contents)
}

fn main() {
    match read_script_from_file() {
        Ok(contents) => {
            let script = & contents[..];

            match tokenize(script) {
                Ok(tokens) => {
                    println !("{:?}", tokens);

                    match parse(&tokens) {
                        Ok(ast) => {
                            println!("{:?}", ast);

                            println!("Output:\n");

                            match interp(ast) {
                                Ok(res) => {
                                    println!("Result: {:?}", res);
                                }
                                Err(error) => {
                                    println!("{}", error);
                                }
                            }
                        }
                        Err(error) => {
                            println!("{}", error);
                        }
                    }
                }
                Err(error) => {
                    println!("{}", error);
                }

            }
        }
        Err(error) => {
            println!("Failed to read script: {}", error);
        }
    }

//    let script =   "\
//    {\
//        message :String :: \"Hello \\\" World\";\
//        print(message);\
//    }";
    // func :: (a :int, b :String){ }
    // [] dot []
    // 10.dot ()
    // dot()
    // struct :: {}
    // array :: []


}
