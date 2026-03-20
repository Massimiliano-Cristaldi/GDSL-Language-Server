//TODO: remove for production build
#![cfg_attr(debug_assertions, allow(unused))]

use tokenizer::Tokenizer;
use parser::Parser;

mod common;
mod tables;

mod tokenizer;
mod parser;

#[cfg(test)]
mod tests;

fn main() {

}