#[macro_use]
extern crate clap;

use clap::{App, Arg};
use std::path::PathBuf;

mod resolve;

fn main() {
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
        .help("List of .sql query files to be executed. If not present then will open repl")
        .min_values(1),
    )
    .get_matches();
  println!("Options {:#?}", options);
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
