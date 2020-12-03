// use futures::Future;
// use futures_state_stream::StateStream;
// use tokio_core::reactor::Core;

use tokio_postgres::{Error, NoTls};

use clap::{crate_authors, crate_description, crate_version, App, Arg};
// use diesel::dsl;
// use diesel::pg::PgConnection;
// use diesel::prelude::*;
// use diesel::sql_query;
// use diesel::sql_types::*;
use dotenv::dotenv;
use std::env;
use std::path::PathBuf;

mod query;
mod repl;
mod resolve;
// #[macro_use]
// mod util;
use query::{postgres, querier};

#[tokio::main]
async fn main() {
  let options = App::new("csvql")
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

  // Test running repl
  repl::run();
}
