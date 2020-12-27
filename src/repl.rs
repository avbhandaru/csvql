use crate::query::{postgres, querier};
use crate::table::{Purveyor, Table};
use crate::util::{evict, less, validate};

use ansi_term::Color;
use ansi_term::Style;
use collections::VecDeque;
use evict::EvictingList;
use querier::QuerierTrait;
use regex;
use rustyline::completion::{Completer, FilenameCompleter};
use rustyline::config::{Configurer, OutputStreamType};
use rustyline::error::ReadlineError;
use rustyline::highlight::{Highlighter, MatchingBracketHighlighter};
use rustyline::hint::{Hinter, HistoryHinter};
use rustyline::validate::{ValidationContext, ValidationResult, Validator};
use rustyline::Editor;
use rustyline::{Cmd, CompletionType, Config, EditMode, KeyEvent, Modifiers, Movement};
use rustyline_derive::{Completer, Helper, Highlighter, Hinter};
use std::io::{stdin, stdout, Write};
use std::str::FromStr;
use std::{collections, env, path};
use validate::Validate;

const QUERY_TABLE_HISTORY_CAPACITY: usize = 20;
const MAX_PRINTABLE_ROWS: usize = 20;
const DATABASE_URL_KEY: &str = "DATABASE_URL";

enum Repl<'a> {
  Quit,
  Continue,
  ClearAndContinue,
  AlertThenContinue(&'a str),
}

#[derive(Debug, Clone)]
enum Command {
  Invalid(String),
  Quit,                                      // Quit the REPL
  Help,                                      // Get help info for REPL
  Usage,                                     // Get usage examples for the REPL
  Query(String),                             // Execute a SQL query
  Import(String, Option<String>),            // Import a csv/json input file as a table in the db
  Export(Option<bool>, bool, usize, String), // Export a table into a csv/json output file
  List(bool),                                // List all tables, views, seqs concisely or verbosely
  Info(bool, String),                        // Show concise or verbose information on a table
  Clear,                                     // Clears repl screen
}

#[derive(Completer, Helper, Highlighter, Hinter)]
struct InputValidator {
  highlighter: MatchingBracketHighlighter,
  hinter: HistoryHinter,
  completer: FilenameCompleter,
  // colored_prompt: String,
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
  }
}

pub async fn run() {
  let mut query_history: Vec<(usize, Command)> = Vec::new();
  let mut query_table_history: VecDeque<(usize, Command, Table)> =
    VecDeque::evl_new(QUERY_TABLE_HISTORY_CAPACITY);

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
    // .color_mode(ColorMode::Enabled)
    .build();
  let helper = InputValidator {
    highlighter: MatchingBracketHighlighter::new(),
    hinter: HistoryHinter {},
    completer: FilenameCompleter::new(),
    // colored_prompt: "".to_owned(),
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
  reader.set_max_history_size(100);
  if reader.load_history("history.txt").is_err() {
    // println!("No previous history.");
    () // do nothing, but auto create the history.txt file
  }

  // Read Eval Print Loop
  let mut count: i128 = -1;
  loop {
    let user_input;
    count += 1;
    // let prompt_text = format!("in[{}]:\n", count as u128);
    let prompt_text = format!(
      "{}[{}]:\n",
      Color::Green.bold().paint("in"),
      Style::new()
        .dimmed()
        .paint((count as u128).to_string().as_str())
    );
    let readline = reader.readline(&prompt_text);
    match readline {
      Ok(line) => {
        if should_be_saved_to_history(&line) {
          reader.add_history_entry(line.as_str());
        }
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
    println!(
      "\n{}[{}]:",
      Color::Blue.bold().paint("out"),
      Style::new()
        .dimmed()
        .paint((count as u128).to_string().as_str())
    );

    let user_command = into_command(count as usize, user_input);
    let result = execute_command(
      count as usize,
      &mut query_history,
      &mut query_table_history,
      &db_querier,
      user_command,
    )
    .await;
    println!("");
    match result {
      Repl::Continue => continue,
      Repl::AlertThenContinue(alert) => println!("{}", alert),
      Repl::ClearAndContinue => {
        print!("\x1B[2J\x1B[1;1H"); // Escape characters that clear screen
        flush_repl();
      }
      Repl::Quit => {
        println!("Goodbye!");
        break;
      }
    }
  }
  // Clean up tables_in_database HashSet
  match reader.append_history("history.txt") {
    Err(e) => println!("Could not append to history. Error: {:#?}", e),
    _ => (),
  }
  clean_database(&db_querier).await;
  println!("");
}

// TODO factor this function out a bit
fn into_command(command_index: usize, user_input: String) -> Command {
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
    ["\\q"] | ["\\quit"] => return Command::Quit,
    ["\\h"] | ["\\help"] => return Command::Help,
    ["\\?"] | ["\\usage"] => return Command::Usage,
    ["\\c"] | ["\\clear"] => return Command::Clear,
    [command, tail @ ..] => match *command {
      "\\i" | "\\import" => match tail {
        [path] => {
          return Command::Import(path.to_string(), None);
        }
        [path, name] => {
          return Command::Import(path.to_string(), Some(name.to_string()));
        }
        _ => {
          return Command::Invalid(user_input);
        }
      },
      "\\e" | "\\export" => {
        // Export regex for extracting i from out[i]
        lazy_static! {
          static ref OUT_RE: regex::Regex = regex::Regex::new(r"out\[(\d+)\]").unwrap();
        }
        match tail {
          [path] => return Command::Export(None, false, 1, path.to_string()),
          [j @ "true", path] | [j @ "false", path] => {
            let use_json = if *j == "true" { true } else { false };
            return Command::Export(Some(use_json), false, 1, path.to_string());
          }
          [n, path] if !OUT_RE.is_match(n) => {
            let which_query;
            match usize::from_str(n) {
              Ok(num) => which_query = num,
              _ => return Command::Invalid("Could not parse query number n.".to_string()),
            };
            return Command::Export(None, false, which_query, path.to_string());
          }
          [out, path] if OUT_RE.is_match(out) => {
            let index: i32 = OUT_RE
              .captures(out)
              .unwrap()
              .get(1)
              .map_or(-1, |m| m.as_str().parse().unwrap());
            if index == -1 {
              return Command::Invalid(user_input);
            }
            return Command::Export(None, true, index as usize, path.to_string());
          }
          [j, n, path] => {
            let use_json = if *j == "true" { true } else { false };
            let which_query;
            match usize::from_str(n) {
              Ok(num) => which_query = num,
              _ => return Command::Invalid("Could not parse query number n.".to_string()),
            };
            return Command::Export(Some(use_json), false, which_query, path.to_string());
          }
          _ => return Command::Invalid(user_input),
        }
      }
      "\\d" => match tail {
        [] => return Command::List(false),
        [name] => return Command::Info(false, String::from(*name)),
        _ => return Command::Invalid(user_input),
      },
      "\\d+" => match tail {
        [] => return Command::List(true),
        [name] => return Command::Info(true, String::from(*name)),
        _ => return Command::Invalid(user_input),
      },
      _ => return Command::Invalid(user_input),
    },
  }
}

async fn execute_command<'a>(
  command_index: usize,
  query_history: &'a mut Vec<(usize, Command)>,
  query_table_history: &'a mut VecDeque<(usize, Command, Table)>,
  db_querier: &postgres::Querier,
  command: Command,
) -> Repl<'a> {
  // Execute the given command
  match command {
    Command::Invalid(user_input) => print_invalid(user_input),
    Command::Quit => return Repl::Quit,
    Command::Help => less_help(),
    Command::Usage => less_usage(),
    Command::Clear => return Repl::ClearAndContinue,
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
          print_table(&table);
          // TODO need to also add a table pointer as an element here...
          let query_statement_result = (
            command_index,
            Command::Query(query_statement.clone()),
            table,
          );
          query_history.push((command_index, Command::Query(query_statement.clone())));
          query_table_history.evl_add(query_statement_result);
        }
        Option::None => return Repl::AlertThenContinue("Success!"),
      }
    }
    Command::Import(path, optional_name) => {
      // Validate and Resolve the relative or absolute path
      let path = path::Path::new(path.as_str());
      let validator_result = path.validate();
      match validator_result {
        Err(_) => return Repl::AlertThenContinue("Invalid file path to import from. Path: {}"),
        _ => (),
      }

      // Yield the pathInfo associated with the relative path given
      let path_info = validator_result.unwrap();
      if path_info.path.is_dir() {
        return Repl::AlertThenContinue("Given path is to a directory. Must be a csv file.");
      }

      // Import table given the resolved absolute path
      let import_result = Table::import(path_info.path.as_path());
      match import_result {
        Err(_) => return Repl::AlertThenContinue("Failure. Table import error occurred."),
        _ => (),
      }
      let table_name = if optional_name == None {
        path_info.filename.unwrap()
      } else {
        optional_name.unwrap()
      };
      let mut table = import_result.unwrap();
      table.set_name(table_name.clone());

      // If this table is alrady in the database then throw
      // TODO: Find a cheaper way to keep track of imported and query created tables?
      let mut is_name_taken = false;
      let result_of_list = db_querier.list(false).await;
      match result_of_list {
        Err(_) => {
          return Repl::AlertThenContinue(
            "Failure. Internal Error. Unable to confirm if table name is taken.",
          )
        }
        Ok(Some(list)) => is_name_taken = list.rows.into_iter().any(|name| table_name == name[0]),
        Ok(None) => (),
      }
      if is_name_taken {
        return Repl::AlertThenContinue("Failure. Table name already taken.");
      } else {
        let absolute_path = path_info.path.as_os_str().to_str().unwrap();
        let result_of_store = db_querier
          .store(absolute_path, table.name.unwrap().as_str(), table.header)
          .await;
        match result_of_store {
          Ok(_) => {
            let result_of_load = db_querier.load(&table_name, Some(4)).await;
            match result_of_load {
              Ok(Some(table)) => {
                println!(
                  "Success! Loaded TABLE[{}] into database. Printing the first 4 rows.\n",
                  table_name
                );
                print_table(&table);
              }
              _ => (),
            }
            return Repl::Continue;
          }
          Err(e) => {
            return Repl::AlertThenContinue(
              "Failure. Error occurred while storing table in database",
            )
          }
        }
      }
    }
    Command::Export(to_json, use_out, query_index, path) => {
      let export_path = path::Path::new(path.as_str());

      println!(
        "Command: {:#?}, export_path: {:?}",
        Command::Export(to_json, use_out, query_index, path.clone()),
        export_path
      );

      if use_out {
        // let result_of_get = query_history.get(query_index);
        let result_of_get = query_history
          .into_iter()
          .filter_map(|(index, command)| {
            if query_index == *index {
              match command {
                Command::Query(query) => Some(query),
                _ => None,
              }
            } else {
              None
            }
          })
          .collect::<Vec<_>>();
        if result_of_get.len() == 0 {
          return Repl::AlertThenContinue("No queries in query history. Nothing to export.");
        }

        // There should only be one value for every command_index
        if result_of_get.len() > 1 {
          return Repl::AlertThenContinue("Query could not be found. Index out of bounds.");
        } else {
          let query = result_of_get[0].as_str();

          // requery to get exportable table
          let result_of_query = db_querier.query(query).await;
          match result_of_query {
            Err(_) => {
              return Repl::AlertThenContinue(
                "Query could not be reexecuted. Thus, query result could not be exported.",
              )
            }
            Ok(None) => {
              return Repl::AlertThenContinue("Nothing to export. Query result has no rows.")
            }
            _ => (),
          }

          // Retrieve exportable table and export it
          let table = result_of_query.unwrap().unwrap();
          match table.export(&export_path, None, Some(query_index)) {
            Ok(_) => (),
            Err(_) => return Repl::AlertThenContinue("Failed to export table using out syntax."),
          }
        }
      } else {
        // Get relative index
        let index = query_table_history.len() - query_index;
        if index >= query_table_history.len() {
          return Repl::AlertThenContinue("Query could not be found.");
        }

        // Retreive Command and Table
        let (_query_statement, queried_table): (&Command, &Table);
        match query_table_history.evl_get(index) {
          Some((_, q, t)) => {
            _query_statement = q;
            queried_table = t;
          }
          None => return Repl::Quit,
        }
        match queried_table.export(&export_path, to_json, None) {
          Ok(_) => (),
          Err(_) => return Repl::AlertThenContinue("Failed to export table."),
        }
      }
    }
    Command::List(is_verbose) => {
      // Verbose table listing
      let result = db_querier.list(is_verbose).await;
      match result {
        Ok(None) => {
          // TODO, remove this clone... I shouldn't have to clone this
          println!("There are no tables in this database.");
          return Repl::Continue;
        }
        Err(_) => return Repl::AlertThenContinue("Failure. Internal table list error."),
        _ => (),
      }
      let table = result.unwrap().unwrap();
      print_table(&table);
    }
    Command::Info(is_verbose, name) => {
      let result = db_querier.info(name.as_str(), is_verbose).await;
      match result {
        Ok(None) | Err(_) => return Repl::AlertThenContinue("Failure. Table not found."),
        _ => (),
      }
      let table = result.unwrap().unwrap();
      print_table(&table);
    }
  }

  Repl::Continue
}

async fn clean_database(db_querier: &postgres::Querier) {
  let result_of_list = db_querier.list(false).await;
  match result_of_list {
    Err(_) => {
      println!("Failure. Could not clean database/i.e. drop tables.");
      return;
    }
    Ok(None) => return, // no tables to drop, gracefully return
    _ => (),
  }
  let tables_in_database = result_of_list.unwrap().unwrap();
  for table_name in tables_in_database.rows.iter() {
    // must get index 0 since table_name is technically a vector of Strings
    match db_querier.drop(table_name[0].as_str()).await {
      Ok(_) => (),
      Err(_) => println!("Failure. Could not drop table with name: {}", table_name[0]),
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

fn should_be_saved_to_history(line: &str) -> bool {
  line.ends_with(";")
    || line.starts_with("\\i")
    || line.starts_with("\\import")
    || line.starts_with("\\e")
    || line.starts_with("\\export")
}

fn print_table(table: &Table) {
  if table.rows.len() > MAX_PRINTABLE_ROWS {
    less::table(&table);
  } else {
    println!("{}", table);
  }
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
  // FIX CLEAR ISSUE
  // \\c or \\clear     - Resets repl and sets the cursor at the top of the terminal window
  // This should be fixed by simply having the help and usage strings moved to a file to
  // be lessed normally

  let help = format!(
    "
    Terminology:
      PATH              - an absolute or relative path to a csv (imports) or json file (exports can be csv or json)

    General:
      \\q or \\quit     - Quit repl
      \\? or \\usage    - Show help on backslash commands (this page)
      \\h or \\help     - Show usage examples for (csvql)
      \\print bool      - If bool is false then no resulting query rows will be printed to repl, vice versa

    Import:
      \\i path          - Imports a csv table into the database given a PATH
      \\i path name     - Imports a csv table into the database given a PATH and aliases the table with given name
      \\import          - Equivalent long form of above, same usages

    Export:
      \\e path          - Exports last query result into csv file given a PATH, equivalent to (\\e 1 path)
      \\e n path        - Exports n(th) last query (1 being most recent, max 20 query history size) into csv file
      \\e j path        - Equivalent to (e path), but exports as json
      \\e j n path      - Equivalent to (e n path), but exports as json
      \\export          - Equivalent long form of above, same usages

    Informational:
      \\d[+]            - List all tables, views and sequences, with additional information if (+) is used
      \\d[+] name       - Describe a table, view, sequence, or index, with additional information if (+) is used
      \\dd

    Display:
      \\x               - Expanded display toggle. If toggled on, then each column appears in its own row.
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
