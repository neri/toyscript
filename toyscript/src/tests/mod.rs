use crate::{
    ast::{statement::Statement, Ast},
    keyword::Keyword,
};
use token::Tokenizer;
// use std::fs::read_to_string;

#[test]
fn function_minimal() {
    let str = "function min1(){}function min2(){}".as_bytes();
    let tokens = Tokenizer::with_slice(str, Keyword::from_str).unwrap();
    let mut tokens = tokens.stream();
    let ast = Ast::from_tokens(&mut tokens).unwrap();

    let func_decl = &ast.module()[0];
    let Statement::Function(function) = func_decl else {
        panic!("Expected FunctionDeclaration, But {:?}", func_decl);
    };
    assert_eq!(function.identifier().as_str(), "min1");
    assert_eq!(function.parameters(), &[]);

    let func_decl = &ast.module()[1];
    let Statement::Function(function) = func_decl else {
        panic!("Expected FunctionDeclaration, But {:?}", func_decl);
    };
    assert_eq!(function.identifier().as_str(), "min2");
    assert_eq!(function.parameters(), &[]);
}

// #[test]
// fn infer() {
//     let str = read_to_string("../test/infer.ts").unwrap();
//     let str = str.as_bytes();
//     ToyScript::generate("infer.ts", str).unwrap();
// }

// #[test]
// fn app_add() {
//     let str = read_to_string("../test/add.ts").unwrap();
//     let str = str.as_bytes();
//     let (tokens, _) = Tokenizer::tokenize(str).unwrap();
//     let _ast = Ast::parse(str, tokens.as_slice()).unwrap();
// }
