#[macro_use]
extern crate lazy_static;

mod boxes;
mod category;
mod dimension;
mod dvi;
mod glue;
mod lexer;
mod list;
mod makro;
mod parser;
mod paths;
mod state;
mod tfm;
mod token;
mod variable;

#[cfg(test)]
mod testing;
#[cfg(test)]
mod tests;

use std::io;
use std::io::prelude::*;

use crate::parser::Parser;
use crate::state::TeXState;

fn main() {
    let mut lines: Vec<String> = Vec::new();

    // Read in every line of stdin. This currently doesn't let us do parsing as
    // we go along, but that's fine.
    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        lines.push(line.unwrap());
    }

    let state = TeXState::new();
    let mut parser = Parser::new(&lines[..], &state);

    // Parse a top-level horizontal list and print out the characters that we
    // got as a result.
    let result: String =
        parser.parse_vertical_box_to_chars().into_iter().collect();

    println!("{}", result);
}
