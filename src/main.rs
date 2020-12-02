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

#[derive(Debug)]
struct Entry {
  a: String,
  b: String,
  c: String,
}

// async fn query() -> Result<(), Error> {
//   let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
//   let (client, connection) = tokio_postgres::connect(&database_url.as_str(), NoTls).await?;

//   // The connection object performs the actual communication with the database,
//   // so spawn it off to run on its own.
//   tokio::spawn(async move {
//     if let Err(e) = connection.await {
//       eprintln!("connection error: {}", e);
//     }
//   });

//   // Now we can execute a simple statement that just returns its parameter.
//   let rows = client.query("select * from test2", &[]).await?;

//   // And then check that we got back the same string we sent over.
//   // println!("results\n {:#?}", rows);
//   let value: &str = rows[0].get(0);
//   // assert_eq!(value, "hello world");
//   println!("value\n {:#?}", value);

//   for (row_index, row) in rows.iter().enumerate() {
//     let mut entry = Entry {
//       a: String::from("0"),
//       b: String::from("0"),
//       c: String::from("0"),
//     };
//     for (col_index, column) in row.columns().iter().enumerate() {
//       // let col_type: String = column.type_().to_string();
//       let col_name: &str = column.name();
//       let value: String = row.get(col_index);
//       // println!("ROW {:#?}, ROW.GET {:#?}", row, value);
//       if col_name == "a" {
//         entry.a = value;
//       } else if col_name == "b" {
//         entry.b = value;
//       } else if col_name == "c" {
//         entry.c = value;
//       }
//     }
//     println!("Row: {}", row_index);
//     println!("{:#?}", entry);
//   }

//   Ok(())
// }

// #[derive(Queryable)]
// struct Post {
//   pub id: i32,
//   pub title: String,
//   pub body: String,
//   pub published: bool,
// }

// fn establish_connection() -> PgConnection {
//   let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
//   PgConnection::establish(&database_url).expect(&format!("Error connecting to {}", database_url))
// }

// #[derive(QueryableByName, Debug)]
// struct Entry {
//   #[sql_type = "Serial"]
//   a: i32,
//   #[sql_type = "Varchar"]
//   b: String,
//   #[sql_type = "Varchar"]
//   c: String,
// }

// fn test_query_connection(conn: &PgConnection) {
//   let results = sql_query("SELECT id, item as name FROM tempdb_table1").load::<Entry>(conn);
//   println!("results\n {:#?}", results);
// }
