#[macro_use(lazy_static)]
extern crate lazy_static;

mod file;
mod query;
mod repl;
mod table;
mod types;
mod util;

use clap::{crate_authors, crate_description, crate_version, App, Arg};
use dotenv::dotenv;
use query::{postgres, querier};
use std::env;

#[tokio::main]
async fn main() {
  let options = App::new("csvql")
    .version(crate_version!())
    .author(crate_authors!())
    .about(crate_description!())
    .usage(
      "
      csvql [FLAGS]                             Opens REPL.
      csvql [FLAGS] [-i imports]                Imports csv tables and opens REPL.
      csvql [FLAGS] [OPTIONS]                   Execute queries, pre-importing tables and exporting query results to given csv output files."
    )
    .arg(
      Arg::with_name("imports")
        .short("i")
        .long("imports")
        .help("List of .csv files to import and resolve into tables")
        .min_values(1),
    )
    .arg(
      Arg::with_name("exports")
        .short("e")
        .long("exports")
        .help("List of export files to be output to as csv or json. Defaults to STDOUT")
        .min_values(1),
    )
    .arg(
      Arg::with_name("use_json")
        .short("j")
        .long("json")
        .help("If present, then query output will be in JSON format versus csv (default)")
    )
    .arg(
      Arg::with_name("infer_types")
        .long("infer")
        .help("If present, then csvql will infer the column types of any given csv, unless type annotations are already provided.")
    )
    .arg(
      Arg::with_name("queries")
        .short("q")
        .long("queries")
        .help("List of .sql query files to be executed. If not present, repl will be opened")
        .min_values(1),
    )
    .get_matches();

  // Load in environment variables
  dotenv().ok();

  // Run repl if no queries were provided in command
  if !options.is_present("queries") {
    repl::run().await;
  } else {
    println!("Raw query execution is not supported yet!");
    println!("Notice: Version 2 of [csvql] will be built ontop of dbcli or pgcli");
  }
}
