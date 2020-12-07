mod file;
mod query;
mod repl;
mod resolve;
mod table;
mod util;

use clap::{crate_authors, crate_description, crate_version, App, Arg};
use dotenv::dotenv;
use query::querier;
use std::env;
use std::path::PathBuf;

#[tokio::main]
async fn main() {
  let _options = App::new("csvql")
    .version(crate_version!())
    .author(crate_authors!())
    .about(crate_description!())
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
        .help("If present, then query output will be in JSON format versus csv (default)"),
    )
    .arg(
      Arg::with_name("queries")
        .short("q")
        .long("queries")
        .help("List of .sql query files to be executed. If not present, repl will be opened")
        .min_values(1),
    )
    .get_matches();
  // ## Test cli works?
  // test_cli_works(options);

  // ## Test running repl
  repl::run();

  // ## Test less command
  // util::less::table("/Users/akhil/csvql/data/test.csv");
}

// TODO remove later
fn _test_cli_works(options: clap::ArgMatches) {
  println!("Options {:#?}", options);
  if options.is_present("queries") {
    let query_vector = options
      .values_of("queries")
      .unwrap()
      .into_iter()
      .map(PathBuf::from)
      .collect::<Vec<_>>();
    println!("Queries: {:#?}", query_vector);
    println!(
      "Query contents\n{:#?}",
      query_vector
        .into_iter()
        .map(resolve::sql_table_paths)
        .collect::<Vec<_>>()
    );
  }
  dotenv().ok();
  for (key, value) in env::vars() {
    println!("==> {}: {}", key, value);
  }
}
