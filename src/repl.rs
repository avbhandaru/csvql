// #[macro_use]
// use crate::util;
use regex::Regex;
use std::io;
use std::io::Write;

enum LoopStep {
  Break,
  Continue,
}

enum Command {
  Help,                         // Get help info for REPL
  Quit,                         // Quit the REPL
  Query(String),                // Execute a SQL query
  Store(String),                // Store a csv/json input file as a table in the database
  Load(Option<String>, String), // Load a table into a csv/json output file
}

pub fn run() -> () {
  loop {
    print!("> ");
    io::stdout().flush().ok().expect("Could not flush stdout");
    let mut literals: Vec<String> = Vec::new();
    let mut line = String::new();
    while !line.trim().ends_with(";") {
      match io::stdin().read_line(&mut line) {
        Ok(_) => {
          println!("User Input is '{}'", line.trim());
          literals.push(line.clone());
        }
        Err(e) => println!("Resolved to error '{}'", e),
      }
    }
    let user_input = literals.join(" ");
    let loop_step = execute_command(into_command(&user_input));
    match loop_step {
      LoopStep::Break => break,
      LoopStep::Continue => continue,
    }
  }
}

fn into_command(literal: &String) -> Command {
  Command::Help
}

fn execute_command(command: Command) -> LoopStep {
  match command {
    Command::Help => {
      println!("HELP");
    }
    Command::Quit => {
      println!("QUIT");
      return LoopStep::Break;
    }
    Command::Query(query_statement) => {
      println!("QUERY: database for => {}", query_statement);
    }
    Command::Store(input_file_path) => {
      println!("STORE: input file {} into database", input_file_path);
    }
    Command::Load(output_file_path, table_name) => {
      println!(
        "LOAD: {} into output file {}",
        table_name,
        if output_file_path.is_some() {
          output_file_path.unwrap()
        } else {
          String::new()
        }
      );
    }
  }
  LoopStep::Continue
}
