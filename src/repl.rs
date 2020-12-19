use crate::query::{postgres, querier};
use crate::resolve;
use crate::table::{Purveyor, Table};
use crate::util::{evict, less};

use collections::HashSet;
use collections::VecDeque;
use evict::EvictingList;
use querier::QuerierTrait;
use rustyline::completion::Completer;
use rustyline::config::{Configurer, OutputStreamType};
use rustyline::error::ReadlineError;
use rustyline::highlight::{Highlighter, MatchingBracketHighlighter};
use rustyline::hint::{Hinter, HistoryHinter};
use rustyline::validate::{ValidationContext, ValidationResult, Validator};
use rustyline::Editor;
use rustyline::{Cmd, CompletionType, Config, EditMode, KeyCode, KeyEvent, Modifiers, Movement};
use rustyline_derive::{Completer, Helper, Highlighter, Hinter};
use std::io::{stdin, stdout, Write};
use std::str::FromStr;
use std::{collections, env, path};

const QUERY_HISTORY_CAPACITY: usize = 5;
const MAX_PRINTABLE_ROWS: usize = 20;
const DATABASE_URL_KEY: &str = "DATABASE_URL";

enum Repl<'a> {
  Quit,
  Continue,
  AlertThenContinue(&'a str),
}

#[derive(Debug, Clone)]
enum Command {
  Invalid(String),
  Quit,                           // Quit the REPL
  Help,                           // Get help info for REPL
  Usage,                          // Get usage examples for the REPL
  Query(String),                  // Execute a SQL query
  Import(String, Option<String>), // Import a csv/json input file as a table in the db
  Export(bool, usize, String),    // Export a table into a csv/json output file
  List(bool),                     // List all tables, views, seqs concisely or verbosely
  Info(bool, String),             // Show concise or verbose information on a table
}

#[derive(Completer, Helper, Highlighter, Hinter)]
struct InputValidator {
  highlighter: MatchingBracketHighlighter,
  hinter: HistoryHinter,
}

impl Validator for InputValidator {
  fn validate(&self, ctx: &mut ValidationContext) -> Result<ValidationResult, ReadlineError> {
    use ValidationResult::{Incomplete, Valid};
    let input = ctx.input();
    let result = if line_not_terminal(input) {
      Incomplete
    } else {
      Valid(None)
    };
    return Ok(result);
    // let input = ctx.input();
    // let result = if !input.starts_with("SELECT") {
    //     Invalid(Some(" --< Expect: SELECT stmt".to_owned()))
    // } else if !input.ends_with(';') {
    //     Incomplete
    // } else {
    //     Valid(None)
    // };
    // Ok(result)
  }
}

pub async fn run() {
  // TODO: both of these should be Box's. I don't know how large they might get
  // so better to put them in heap. Although there is a chance they are auto
  // placed in heap?
  let mut tables_in_database: HashSet<String> = HashSet::new();
  let mut query_history: VecDeque<(Command, Table)> = VecDeque::evl_new(QUERY_HISTORY_CAPACITY);
  // Get the querier
  // Handle these Errors...
  // This should be at repl level?
  let db_url = env::var(DATABASE_URL_KEY).unwrap();
  let db_querier = postgres::Querier::new("postgres", db_url.as_str())
    .await
    .unwrap();

  // rustyline reader configuration
  let config = Config::builder()
    .history_ignore_space(true)
    .history_ignore_dups(true)
    .completion_type(CompletionType::List)
    .edit_mode(EditMode::Emacs)
    .output_stream(OutputStreamType::Stdout)
    .build();
  let helper = InputValidator {
    highlighter: MatchingBracketHighlighter::new(),
    hinter: HistoryHinter {},
  };
  let mut reader = Editor::with_config(config);
  reader.set_helper(Some(helper));
  reader.bind_sequence(KeyEvent::alt('N'), Cmd::HistorySearchForward);
  reader.bind_sequence(KeyEvent::alt('P'), Cmd::HistorySearchBackward);
  // indents the entire start of line... not just from location... todo! fix
  reader.bind_sequence(
    KeyEvent::new('\t', Modifiers::NONE),
    Cmd::Indent(Movement::ForwardChar(4)),
  );
  reader.bind_sequence(
    KeyEvent::new('\t', Modifiers::SHIFT),
    Cmd::Dedent(Movement::BackwardChar(4)),
  );
  reader.set_max_history_size(50);
  // reader.set_tab_stop(4);
  // reader.set_indent_size(4);
  if reader.load_history("history.txt").is_err() {
    // println!("No previous history.");
    () // do nothing, but auto create the history.txt file
  }

  // Read Eval Print Loop
  loop {
    // print!(">>> ");
    // flush_repl();

    // TODO: try out rustyline
    let user_input;
    let readline = reader.readline("[in]:\n");
    match readline {
      Ok(line) => {
        reader.add_history_entry(line.as_str());
        user_input = line;
      }
      Err(ReadlineError::Interrupted) => {
        println!("CTRL-C");
        break;
      }
      Err(ReadlineError::Eof) => {
        println!("CTRL-D");
        break;
      }
      Err(err) => {
        println!("Error: {:?}", err);
        break;
      }
    }
    // println!("userinput: {}", user_input);
    println!("\n[out]:");
    // Ok(())

    // let mut experienced_read_error = false;
    // let mut lines: Vec<String> = Vec::new();
    // while !experienced_read_error {

    //   let mut line = String::new();
    //   match stdin().read_line(&mut line) {
    //     Ok(_) => lines.push(line.trim().to_string()),
    //     Err(e) => {
    //       print_error(e);
    //       experienced_read_error = true;
    //     }
    //   }
    //   if line_not_terminal(&line) {
    //     print!("... ");
    //     flush_repl();
    //   } else {
    //     break;
    //   }
    // }
    // println!("");
    // if experienced_read_error {
    //   continue;
    // }

    // let user_input = lines.join(" ");
    // println!("User command: {}", user_input);
    let user_command = into_command(user_input);
    let result = execute_command(
      &mut tables_in_database,
      &mut query_history,
      &db_querier,
      user_command,
    )
    .await;
    println!("");
    match result {
      Repl::Continue => continue,
      Repl::AlertThenContinue(alert) => println!("{}", alert),
      Repl::Quit => break,
    }
  }
  // Clean up tables_in_database HashSet
  match reader.append_history("history.txt") {
    Err(e) => println!("Could not append to history. Error: {:#?}", e),
    _ => (),
  }
  clean_database(&mut tables_in_database, &db_querier).await;
  println!("");
}

// TODO factor this function out a bit
fn into_command(user_input: String) -> Command {
  if line_is_invalid(&user_input) {
    return Command::Invalid(user_input);
  }

  if user_input.ends_with(";") {
    return Command::Query(user_input.strip_suffix(";").unwrap().to_string());
  }

  let user_input_args = user_input
    .split(" ")
    .filter(|element| element != &"")
    .map(|s| s.trim())
    .collect::<Vec<&str>>();
  // println!("user input args: {:#?}", user_input_args);
  match user_input_args.as_slice() {
    [] => return Command::Invalid("".to_string()),
    ["\\q"] => return Command::Quit,
    ["\\?"] => return Command::Help,
    ["\\h"] => return Command::Usage,
    [command, tail @ ..] => match *command {
      "\\i" | "\\import" => match tail {
        [path] => {
          println!("import path: {}", path);
          return Command::Import(path.to_string(), None);
        }
        [path, name] => {
          println!("import path: {}, name: {}", path, name);
          return Command::Import(path.to_string(), Some(name.to_string()));
        }
        _ => {
          println!(
            "import tail: {}",
            if tail.len() > 0 { tail[0] } else { "[]" }
          );
          return Command::Invalid(user_input);
        }
      },
      "\\e" | "\\export" => {
        match tail {
          [path] => return Command::Export(false, 1, path.to_string()),
          [j @ "true", path] | [j @ "false", path] => {
            let use_json = if *j == "true" { true } else { false };
            return Command::Export(use_json, 1, path.to_string());
          }
          [n, path] => {
            let which_query;
            match usize::from_str(n) {
              Ok(num) => which_query = num,
              _ => return Command::Invalid("Could not parse query number n.".to_string()),
            };
            return Command::Export(false, which_query, path.to_string());
          }
          [j, n, path] => {
            let use_json = if *j == "true" { true } else { false };
            let which_query;
            match usize::from_str(n) {
              Ok(num) => which_query = num,
              _ => return Command::Invalid("Could not parse query number n.".to_string()),
            };
            return Command::Export(use_json, which_query, path.to_string());
          }
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

async fn execute_command<'a>(
  tables_in_database: &'a mut HashSet<String>,
  query_history: &'a mut VecDeque<(Command, Table)>,
  db_querier: &postgres::Querier,
  command: Command,
) -> Repl<'a> {
  // Execute the given command
  match command {
    Command::Invalid(user_input) => print_invalid(user_input),
    Command::Quit => return Repl::Quit,
    Command::Help => less_help(),
    Command::Usage => less_usage(),
    Command::Query(query_statement) => {
      // handle this error.
      let result = db_querier.query(query_statement.as_str()).await;
      match result {
        Err(_) => return Repl::AlertThenContinue("Failure. Query syntax error."),
        _ => (),
      }
      let result = result.unwrap();
      match result {
        Some(table) => {
          if table.rows.len() > MAX_PRINTABLE_ROWS {
            less::table(&table);
          } else {
            println!("{}", table);
          }
          // TODO need to also add a table pointer as an element here...
          let query_statement_result_pair = (Command::Query(query_statement.clone()), table);
          query_history.evl_add(query_statement_result_pair);
        }
        Option::None => return Repl::AlertThenContinue("Success!"),
      }
    }
    Command::Import(path, optional_name) => {
      // TODO: Perform path validation, make sure its real
      // handle error
      let mut table = Table::import(path::Path::new(path.as_str())).unwrap();
      let table_name;
      match optional_name {
        Some(name) => {
          table_name = name.clone();
          table.set_name(name);
        }
        None => {
          table_name = resolve::get_table_name_from_path(path.as_str());
          table.set_name(table_name.clone())
        }
      };
      // ABOVE: seems to work
      // println!("Importing table!");
      // less::table(&table);
      let result = db_querier
        .store(path.as_str(), table.name.unwrap().as_str(), table.header)
        .await;
      match result {
        Ok(_) => {
          let insert_result = tables_in_database.insert(table_name);
          if !insert_result {
            // This shouldn't happen, if there is naming issue in the database
            // We would be in the Err(_) branch of the match
            return Repl::AlertThenContinue(
              "Failure. Unable to import table. Perhaps choose a different name.",
            );
          }
        }
        Err(_) => return Repl::AlertThenContinue("Failure. Unable to import table."),
      }
    }
    Command::Export(to_json, query_index, path) => {
      // TODO: Perform path validation, make sure its real/or create it
      // TODO Handle this error
      let index = query_history.len() - query_index;
      if index >= query_history.len() {
        // TODO handle this error, don't just quit the repl
        // return Repl::Quit;
        return Repl::AlertThenContinue("Query could not be found.");
      }
      let (_query_statement, queried_table): (&Command, &Table);
      match query_history.evl_get(index) {
        Some((q, t)) => {
          _query_statement = q;
          queried_table = t;
        }
        None => return Repl::Quit,
      }
      let export_path = path::Path::new(path.as_str());
      queried_table.export(&export_path, to_json);
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

async fn clean_database(tables_in_database: &mut HashSet<String>, db_querier: &postgres::Querier) {
  for table_name in tables_in_database.iter() {
    match db_querier.drop(table_name.as_str()).await {
      Ok(_) => (),
      Err(_e) => println!("Failure. Could not clean database/i.e. drop tables."),
    }
  }
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
