#![feature(thread_local)]

#[macro_use]
mod heap;
mod term;
mod symbol;
mod parser;
mod eval;
mod compile;
mod task;

use parser::*;

extern crate lazy_static;
extern crate regex;

use std::process;
use std::fs;

use rustyline::{Editor, Result};
use rustyline::error::ReadlineError;
use rustyline::validate::{
    MatchingBracketValidator, ValidationContext, ValidationResult, Validator,
};
use rustyline_derive::{Completer, Helper, Highlighter, Hinter};

#[derive(Completer, Helper, Highlighter, Hinter)]
struct InputValidator {
    brackets: MatchingBracketValidator,
}

impl Validator for InputValidator {
    fn validate(&self, ctx: &mut ValidationContext) -> Result<ValidationResult> {
        self.brackets.validate(ctx)
    }
}

fn main() {
    task::thread_init();

    let h = InputValidator {
        brackets: MatchingBracketValidator::new(),
    };
    let mut rl = Editor::new();
    rl.set_helper(Some(h));
    if rl.load_history("history.txt").is_err() {
        println!("No previous history.");
    }
    command_line(":load test.nrm".to_string());
    loop {
        let readline = rl.readline("> ");
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str());
                let input = String::from(line);
                let input = input.to_string();
                command_line(input);
            },
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                process::exit(1);
            },
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break;
            },
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }
    rl.save_history("history.txt").unwrap();
    
    task::thread_exit();
}

fn command_line(input: String) {
    let input = input.trim().to_string();
    if input == "" { return }
    //println!("cmd {}",input);
    let mut par = parser::Parser::new(input);
    if let Some(command) = parser::read_command(&mut par) {
        match command {
            Command::Quit => {
                process::exit(1);
            }
            Command::Dict => {
                symbol::show_dict();
            }
            Command::Define(symb,term) => {
                symbol::define(symb,term);
            }
            Command::Update(symb,term) => {
                symbol::update(symb,term);
            }
            Command::Delete(symb) => {
                symbol::delete(symb);
            }
            Command::Load(path) => {
                let text = fs::read_to_string(&path).unwrap();
                for command in text.split(";;") {
                    command_line(command.to_string());  
                }
                println!("load:{} finished.", &path);
            }
            Command::Repl(term) => {
                println!("Parsed: {:?}", *term);
                let compiled = compile::compile_ski(term);
                println!("Compiled: {:?}", *compiled);
                let optimized = compile::optimize(compiled);
                println!("Optimized: {:?}", *optimized);
                let mut task = eval::Task::new(optimized);
                println!("Task: {:?}", task);
                if let Some(ret) = task.eval(256) {
                    println!("{:?}", *ret);
                } else {
                    task::send_task(task);
                }
            }
        }
    } else {
        println!("Can't parse command!");
    }
}



