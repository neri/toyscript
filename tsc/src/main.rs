//! ToyScript Compiler

use std::{
    env::{self, args},
    fs::read_to_string,
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

fn main() {
    let mut args = args();
    let _ = args.next().unwrap();

    let mut to_run = false;
    let mut path_input = None;
    let mut path_output = None;

    while let Some(arg) = args.next() {
        if arg.starts_with("-") {
            match arg.as_str() {
                "-run" => {
                    to_run = true;
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
    let _ = path_output;
    let _ = to_run;

    let src = read_to_string(path_input.as_str()).unwrap();
    let text = match ToyScript::debug_ir(path_input.as_str(), src.into_bytes()) {
        Ok(v) => v,
        Err(err) => {
            eprintln!("{}", err);
            process::exit(1)
        }
    };

    println!("{}", text);
}
