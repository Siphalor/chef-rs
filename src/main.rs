extern crate pest_derive;

use std::io::Read;

use clap::Parser as ArgParser;
use pest::Parser;

use crate::ast::recipe::recipes_from;
use crate::grammar::{ChefParser, Rule};
use crate::interpreter::interpreter::Interpreter;

mod ast;
mod grammar;
mod interpreter;

#[derive(ArgParser)]
#[clap(
    version = "0.1.0",
    author = "Siphalor <info@siphalor.de>",
    rename_all = "kebab",
    about = "Chef interpreter/compiler in Rust",
)]
struct Opts {
    /// An input file
    #[clap(required=true)]
    input: String,
}

fn main() {
    let opts: Opts = Opts::parse();

    println!("Hello, kitchen!");

    match std::fs::File::open(opts.input) {
        Ok(mut file) => {
            let mut code = String::new();
            if let Err(err) = file.read_to_string(&mut code) {
                eprintln!("file read error: {}", err);
            }

            let parsed = ChefParser::parse(Rule::recipes, code.as_str());
            match parsed {
                Ok(mut parsed) => {
                    match recipes_from(parsed.next().unwrap()) {
                        Ok(recipes) => {
                            if let Err(err) = Interpreter::new(recipes).run_main() {
                                println!("error: {:?}", err);
                            }
                        }
                        Err(err) => {
                            println!("transform error:\n{:?}", err);
                        }
                    }
                },
                Err(err) => {
                    println!("parse error:\n{:?}", err);
                }
            }
        }
        Err(err) => {
            eprintln!("failed to open file: {}", err);
        }
    }
}
