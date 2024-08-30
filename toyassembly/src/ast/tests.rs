use super::{keyword::Keyword, AstModule};
use crate::token::{TokenPosition, Tokenizer};

#[test]
fn minimal() {
    let src = "(module)";
    let tokens = Tokenizer::with_slice(src.as_bytes(), Keyword::from_str).unwrap();
    let mut tokens = tokens.stream();
    AstModule::from_tokens(&mut tokens).unwrap();

    let src = "(module\n)";
    let tokens = Tokenizer::with_slice(src.as_bytes(), Keyword::from_str).unwrap();
    let mut tokens = tokens.stream();
    AstModule::from_tokens(&mut tokens).unwrap();

    let src = "(module (;;))";
    let tokens = Tokenizer::with_slice(src.as_bytes(), Keyword::from_str).unwrap();
    let mut tokens = tokens.stream();
    AstModule::from_tokens(&mut tokens).unwrap();

    let src = "(module";
    let tokens = Tokenizer::with_slice(src.as_bytes(), Keyword::from_str).unwrap();
    let mut tokens = tokens.stream();
    assert!(AstModule::from_tokens(&mut tokens)
        .unwrap_err()
        .explanation()
        .unwrap()
        .contains("EndOfFile"));

    let src = "(module (;;)";
    let tokens = Tokenizer::with_slice(src.as_bytes(), Keyword::from_str).unwrap();
    let mut tokens = tokens.stream();
    assert!(AstModule::from_tokens(&mut tokens)
        .unwrap_err()
        .explanation()
        .unwrap()
        .contains("EndOfFile"));

    let src = "(module])";
    let tokens = Tokenizer::with_slice(src.as_bytes(), Keyword::from_str).unwrap();
    let mut tokens = tokens.stream();
    let error = AstModule::from_tokens(&mut tokens).unwrap_err();
    assert_eq!(
        error.position(),
        &crate::ErrorPosition::Range(TokenPosition((7, 8)))
    );

    let src = "(module/)";
    let tokens = Tokenizer::with_slice(src.as_bytes(), Keyword::from_str).unwrap();
    let mut tokens = tokens.stream();
    let error = AstModule::from_tokens(&mut tokens).unwrap_err();
    assert_eq!(
        error.position(),
        &crate::ErrorPosition::Range(TokenPosition((1, 8)))
    );

    let src = "(module (foo))";
    let tokens = Tokenizer::with_slice(src.as_bytes(), Keyword::from_str).unwrap();
    let mut tokens = tokens.stream();
    let error = AstModule::from_tokens(&mut tokens).unwrap_err();
    assert_eq!(
        error.position(),
        &crate::ErrorPosition::Range(TokenPosition((9, 12)))
    );
}
