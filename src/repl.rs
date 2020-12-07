// #[macro_use]
// use crate::util;
use crate::util::less;
use queues::{Buffer, IsQueue};
use regex::Regex;
use std::fmt::Display;
use std::io::{stdin, stdout, Write};

const QUERY_HISTORY_CAPACITY: usize = 5;

enum Repl {
  Quit,
  Continue,
}

#[derive(Debug, Clone)]
enum Command {
  Invalid(String),
  Quit,                                      // Quit the REPL
  Help,                                      // Get help info for REPL
  Usage,                                     // Get usage examples for the REPL
  Query(String),                             // Execute a SQL query
  Import(String),                            // Import a csv/json input file as a table in the db
  Export(Option<bool>, Option<i32>, String), // Export a table into a csv/json output file
  List(bool),                                // List all tables, views, seqs concisely or verbosely
  Info(bool, String),                        // Show concise or verbose information on a table
}

pub fn run() {
  let mut query_history: Buffer<Command> = Buffer::new(QUERY_HISTORY_CAPACITY);
  loop {
    print!("> ");
    flush_repl();

    let mut experienced_read_error = false;
    let mut lines: Vec<String> = Vec::new();
    let mut line: String = String::new();
    while line_not_terminal(&line) && !experienced_read_error {
      match stdin().read_line(&mut line) {
        Ok(_) => lines.push(line.trim().to_string()),
        Err(e) => {
          print_error(e);
          experienced_read_error = true;
        }
      }
      print!("  ");
      flush_repl();
    }
    println!("");
    if experienced_read_error {
      continue;
    }

    let user_input = lines.join(" ");
    let user_command = into_command(user_input);
    let result = execute_command(&mut query_history, user_command);
    match result {
      Repl::Continue => continue,
      Repl::Quit => break,
    }
  }
  println!("");
}

fn into_command(user_input: String) -> Command {
  if line_is_invalid(&user_input) {
    return Command::Invalid(user_input);
  }

  if user_input.ends_with(";") {
    return Command::Query(user_input.strip_suffix(";").unwrap().to_string());
  }

  let user_input_args = user_input.split("\\s").collect::<Vec<&str>>();
  match user_input_args.as_slice() {
    [] => return Command::Invalid("".to_string()),
    ["\\q"] => return Command::Quit,
    ["\\?"] => return Command::Help,
    ["\\h"] => return Command::Usage,
    [command, tail @ ..] => match *command {
      "\\i" | "\\import" => {
        match tail {
          [path] => unimplemented!(),
          [path, name] => unimplemented!(),
          _ => return Command::Invalid(user_input),
        }
        println!(
          "Command given: {:#?}, with args: {:#?}",
          *command,
          tail.to_vec()
        );
        unimplemented!();
      }
      "\\e" | "\\export" => {
        match tail {
          [path] => unimplemented!(),
          [j @ "true", path] | [j @ "false", path] => unimplemented!(),
          [n, path] => unimplemented!(),
          [j, n, path] => unimplemented!(),
          _ => return Command::Invalid(user_input),
        }
        println!(
          "Command given: {:#?}, with args: {:#?}",
          *command,
          tail.to_vec()
        );
        unimplemented!();
      }
      "\\d" => {
        match tail {
          [] => unimplemented!(),
          [name] => unimplemented!(),
          _ => return Command::Invalid(user_input),
        }
        println!(
          "Command given: {:#?}, with args: {:#?}",
          *command,
          tail.to_vec()
        );
        unimplemented!();
      }
      "\\d+" => {
        match tail {
          [] => unimplemented!(),
          [name] => unimplemented!(),
          _ => return Command::Invalid(user_input),
        }
        println!(
          "Command given: {:#?}, with args: {:#?}",
          *command,
          tail.to_vec()
        );
        unimplemented!();
      }
      _ => return Command::Invalid(user_input),
    },
  }

  unimplemented!();
}

fn execute_command(query_history: &mut Buffer<Command>, command: Command) -> Repl {
  match command {
    Command::Invalid(user_input) => print_invalid(user_input),
    Command::Quit => return Repl::Quit,
    Command::Help => less_help(),
    Command::Usage => less_usage(),
    Command::Query(query) => {
      let res = query_history.add(Command::Query(query.clone()));
      match res {
        Ok(Some(query_command)) => {
          println!(
            "
            Successfully added query to query history {:#?}
            Removed query {:#?} in the process
            ",
            query_history, query_command
          );
        }
        Ok(None) => {
          println!(
            "
            Successfully added query to query history {:#?}
            ",
            query_history
          );
        }
        Err(e) => {
          println!(
            "
            Failed to add query to query history.
            Received following error {:#?}
            ",
            e
          );
          return Repl::Quit;
        }
      }
      unimplemented!()
    }
    Command::Import(path) => {
      unimplemented!()
    }
    Command::Export(to_json, query_index, path) => {
      unimplemented!()
    }
    Command::List(is_verbose) => {
      unimplemented!()
    }
    Command::Info(is_verbose, name) => {
      unimplemented!()
    }
  }

  Repl::Continue
}

fn flush_repl() {
  stdout()
    .flush()
    .ok()
    .expect("Failed to flush output to repl.")
}

fn line_not_terminal(line: &str) -> bool {
  let trimmed_line = line.trim();
  !(trimmed_line.starts_with("\\") || trimmed_line.ends_with(";"))
}

fn line_is_invalid(line: &str) -> bool {
  let trimmed_line = line.trim();
  trimmed_line.starts_with("\\") && trimmed_line.ends_with(";")
}

fn print_error(err: std::io::Error) {
  println!(
    "
    Experienced an error:
      Could not read in user input in repl.

    {:#?}
    ",
    err
  )
}

fn print_invalid(user_input: String) {
  println!(
    "
    Invalid Command:
    {:#?}

    Try \\? for help.
    ",
    user_input
  )
}

// TODO put help in a public/static/docs/file
fn less_help() {
  let help = format!(
    "
    Terminology:
      PATH              - an absolute or relative path to a csv (imports) or json file (exports can be csv or json)

    General:
      \\q               - Quit repl
      \\?               - Show help on backslash commands (this page)
      \\h               - Show usage examples for (csvql)
      \\print bool      - If bool is false then no resulting query rows will be printed to repl, vice versa

    Import:
      \\i path          - Imports a csv table into the database given a PATH
      \\i path name     - Imports a csv table into the database given a PATH and aliases the table with given name
      \\import          - Equivalent long form of above, same usages

    Export:
      \\e path          - Exports last query result into csv file given a PATH, equivalent to (\\e 1 path)
      \\e n path        - Exports n(th) last query (1 being most recent, max 5 query history size) into csv file
      \\e j path        - Equivalent to (e path), but exports as json
      \\e j n path      - Equivalent to (e n path), but exports as json
      \\export          - Equivalent long form of above, same usages

    Informational:
      \\d[+]            - list tables, views and sequences, with additional information if (+) is used
      \\d[+] name       - describe table, view, sequence, or index, with additional information if (+) is used
    "
  );
  less::string(help);
}

// TODO put usage in a public/static/docs/file
fn less_usage() {
  let usage = format!(
    "
    Querying the Database:
      Regular sql code followed by a semi-colon.
      Following are some examples.

      > SELECT table.a, table.b
        FROM table
        JOIN other_table ON table.a = other_table.a;
        --------------------------------------------
          a    | b | c
        --------+---+----
        ribbit | 1 | a
        woof   | 2 | an
        meow   | 7 | ask
               .
               .
               .

      > CREATE TABLE other_other_table (
          a SERIAL,
          b VARCHAR(256),
        );
        --------------------------------
        Successfully created table with name 'other_other_table'

      Any resulting rows from a select statement will be printed to the repl.
      This stdout feature can be suppresed using a repl backslash command (\\print).

    Importing Tables:
      Use the (\\?) command to find out the exact syntax for import statements.
      Following are some examples.

      > \\i '~/home/csv_tables/other_table.csv'
        ---------------------------------------
        Successfully imported table from '~/home/csv_tables/other_table.csv' as 'other_table'

      > \\i '/Users/home/csv_tables/asdfghjkl.csv' random_letters_table
        ---------------------------------------------------------------
        Successfully imported table from '~/home/csv_tables/asdfghjkl.csv' as 'random_letters_table'

      The above will import the csv table from the given file.
      By default all column types will be set to VARCHAR with max size of 1 GB.
      Planning on supporting sql type annotations in csv column header.

    Exporting Tables:
        Use the (\\?) command to find out the exact syntax for export statements.

        > \\print false
        > SELECT * FROM random_letters_table;
        > \\e '~/home/csv_tables/exported_table.csv'
          ------------------------------------------
          Successfully exported last query:
            'SELECT * FROM random_letters_table'
          into csv file '~/home/csv_tables/exported_table.csv'

        This will export the resulting rows of the last query into the provided csv file.
        If the file does not already exist, it will be created for you.

        > \\e 5 '~/home/csv_tables/5th_exported_table.csv'

        In this case the 5th most recent (5th to last) query result will be exported.

        > \\e j 2 '/Users/home/json_tables/2nd_exported_table.json'

        In this case, the penultimate (2nd to last) query result will be formatted as json
        and exported to the provided json file.

    Getting Information:
      Use the (\\?) command to find out the exact syntax on informational statements.
      These work in the same way the psql command (\\d[S+] [name]) works.

      > \\d
        ---
        Schema |         Name         |   Type   | Owner
        --------+----------------------+----------+-------
        public | table                | table    | user
        public | table_id_seq         | sequence | user
        public | other_table          | table    | user
        public | other_other_table    | table    | user
        public | other_table_seq      | sequence | user

      > \\d other_other_table
        ---------------------
         Column |          Type          | Collation | Nullable |             Default
        --------+------------------------+-----------+----------+---------------------------------
         a      | integer                |           | not null | nextval('test_a_seq'::regclass)
         b      | character varying(256) |           |          |
         c      | character varying(128) |           |          |

    "
  );
  less::string(usage);
}
