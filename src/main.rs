//TODO: remove for production build
#![cfg_attr(debug_assertions, allow(unused))]

use lexer::Lexer;
use parser::Parser;

mod common;
mod tables;

mod lexer;
mod parser;

#[cfg(test)]
mod tests;

fn main() {

}