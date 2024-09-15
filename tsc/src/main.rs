//! ToyScript Compiler

use std::{
    env::{self, args},
    fs::{read_to_string, File},
    io::{self, Write},
    process,
};
use toyscript::{self, ToyScript};

fn usage() -> ! {
    let mut args = env::args_os();
    let arg = args.next().unwrap();
    eprintln!(
        "ToyScript Compiler\nusage: {} [OPTIONS] INPUT",
        arg.to_str().unwrap()
    );
    process::exit(1);
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum OperationMode {
    Default,
    ShowAst,
    ShowTir,
    ToRun,
}

fn main() {
    let mut args = args();
    let _ = args.next().unwrap();

    let mut mode = OperationMode::Default;
    let mut path_input = None;
    let mut path_output = None;

    while let Some(arg) = args.next() {
        if arg.starts_with("-") {
            match arg.as_str() {
                "-ast" => {
                    if mode == OperationMode::Default {
                        mode = OperationMode::ShowAst;
                    } else {
                        panic!("too many option: {}", arg)
                    }
                }
                "-tir" => {
                    if mode == OperationMode::Default {
                        mode = OperationMode::ShowTir;
                    } else {
                        panic!("too many option: {}", arg)
                    }
                }
                "-run" => {
                    if mode == OperationMode::Default {
                        mode = OperationMode::ToRun;
                    } else {
                        panic!("too many option: {}", arg)
                    }
                }
                "-o" => match args.next() {
                    Some(v) => path_output = Some(v),
                    None => usage(),
                },
                "--" => {
                    path_input = args.next();
                    break;
                }
                _ => panic!("unknown option: {}", arg),
            }
        } else {
            path_input = Some(arg);
            break;
        }
    }

    let path_input = match path_input {
        Some(v) => v,
        None => usage(),
    };
    let src = read_to_string(path_input.as_str()).unwrap();

    match mode {
        OperationMode::Default => {
            let binary = match ToyScript::to_wasm(path_input.as_str(), src.into_bytes()) {
                Ok(v) => v,
                Err(err) => {
                    eprintln!("{}", err);
                    process::exit(1)
                }
            };

            if let Some(path_output) = path_output {
                let mut os = File::create(path_output).unwrap();
                os.write(&binary).unwrap();
            } else {
                io::stdout().write(binary.as_slice()).unwrap();
            }
        }
        OperationMode::ShowAst => {
            match ToyScript::explain_ast(path_input.as_str(), src.into_bytes()) {
                Ok(v) => println!("{}", v),
                Err(err) => {
                    eprintln!("{}", err);
                    process::exit(1)
                }
            }
        }
        OperationMode::ShowTir => {
            match ToyScript::explain_toyir(path_input.as_str(), src.into_bytes()) {
                Ok(v) => println!("{}", v),
                Err(err) => {
                    eprintln!("{}", err);
                    process::exit(1)
                }
            }
        }
        OperationMode::ToRun => todo!(),
    }
}
