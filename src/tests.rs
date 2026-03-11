use std::fs;
use std::path::Path;
use std::process;

use super::*;
use crate::parser::Diagnostic;

// #[test]
fn tokens() {
    let source = match fs::read_to_string(Path::new("test_material/test2.gdshader")) {
        Ok(txt) => txt,
        Err(e) => {
            eprintln!("Failed to read test file: {e}");
            process::exit(1);
        }
    };

    let tokenizer = Tokenizer::new(&source);
    let tokens = tokenizer.tokenize();
    println!("{tokens:#?}");

    let result: Vec<&str> = tokens.iter().map(|t| t.value).collect();
    let expected = ["shader_type", "canvas_item", ";", "uniform", "float", "fade_factor", "=", "1.0", ";", "//This is a single line comment/*", "/*", "* This is a multiline comment", "*/", "void", "fragment", "(", ")", "{", "const", "float", "radius", "=", "4.0", ";", "float", "screen_width", "=", "1.0", "/", "SCREEN_PIXEL_SIZE", ".", "x", ";", "float", "screen_height", "=", "1.0", "/", "SCREEN_PIXEL_SIZE", ".", "y", ";", "float", "screen_ratio", "=", "screen_width", "/", "screen_height", ";", "float", "x_pos", "=", "abs", "(", "SCREEN_UV", ".", "x", "-", "0.5", ")", "*", "radius", "*", "screen_ratio", ";", "float", "y_pos", "=", "abs", "(", "SCREEN_UV", ".", "y", "-", "0.5", ")", "*", "radius", ";", "float", "alpha", "=", "sqrt", "(", "pow", "(", "x_pos", ",", "2", ")", "+", "pow", "(", "y_pos", ",", "2", ")", "+", "0.1", ")", ";", "COLOR", ".", "a", "=", "alpha", "+", "fade_factor", ";", "}"];

    assert_eq!(result, expected);
}

#[test]
fn test1() {
    let source = match fs::read_to_string(Path::new("test_material/test1.gdshader")) {
        Ok(txt) => txt,
        Err(e) => {
            eprintln!("Failed to read test file: {e}");
            process::exit(1);
        }
    };

    let tokenizer = Tokenizer::new(&source);
    let tokens = tokenizer.tokenize();

    let parser = Parser::new(tokens);
    let result = parser.get_diagnostics();

    let expected = [
        Diagnostic::new(
            String::from("Unexpected token kind: expected 'float literal | float (highp)', found 'int literal'"),
            7,
            25,
            26,
        ),
        Diagnostic::new(
            String::from("Unexpected token kind: expected 'float literal | float (highp)', found 'keyword'"),
            9,
            18,
            24,
        ),
    ];
    assert_eq!(result, expected);

    println!("{:#?}", result);
}

#[test]
fn test2() {
    let source = match fs::read_to_string(Path::new("test_material/test2.gdshader")) {
        Ok(txt) => txt,
        Err(e) => {
            eprintln!("Failed to read test file: {e}");
            process::exit(1);
        }
    };

    let tokenizer = Tokenizer::new(&source);
    let tokens = tokenizer.tokenize();

    let parser = Parser::new(tokens);
    let result = parser.get_diagnostics();

    let expected = [
        Diagnostic::new(
            String::from("Unexpected token kind: expected 'float literal | float (highp)', found 'int literal'"),
            7,
            25,
            26,
        ),
        Diagnostic::new(
            String::from("Unexpected token kind: expected 'float literal | float (highp)', found 'keyword'"),
            9,
            18,
            24,
        ),
        Diagnostic::new(
            String::from("Unexpected token value: expected ';', found 'return'"),
            9,
            25,
            31,
        ),
    ];
    assert_eq!(result, expected);

    println!("{:#?}", result);
}
