use crate::table;
use io::BufRead;
use serde::{Deserialize, Serialize};
use std::{convert, fs, io, path};

#[derive(Debug)]
pub struct Error {
  path: path::PathBuf,
  message: String,
}

impl Error {
  fn new(path: String, message: String) -> Self {
    Self {
      path: path::Path::new(path.as_str()).to_path_buf(),
      message: message,
    }
  }
}

impl std::fmt::Display for Error {
  fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(formatter, "")
  }
}

impl std::error::Error for Error {}

// ## I need the below if I want to auto convert errors to my Error
// impl convert::From<io::Error> for Error {
//   // fn from(error: io::Error) -> Self {
//     ///         CliError::IoError(error)
//     ///     }
//   fn from(error: io::Error) -> Self {
//     Self {
//       path:
//     }
//   }
// }

pub fn import_csv(path: &path::Path) -> Result<table::Table, Error> {
  let lines = read_file(path)?
    .into_iter()
    .map(|line| line.split(",").map(String::from).collect())
    .collect::<Vec<Vec<String>>>();
  // println!("csv file:\n{:#?}", lines);
  // remove later
  if lines.len() == 0 {
    Err(Error::new(
      path.to_str().unwrap().to_string(),
      "Failed to parse Table from file. Read in Zero lines.".to_string(),
    ))
  } else {
    let header = lines
      .get(0)
      .unwrap()
      .into_iter()
      .map(|entry| (entry.clone(), "VARCHAR".to_string()))
      .collect::<table::Header>();
    let rows = lines[1..].to_vec();
    Ok(table::Table::new(header, rows))
  }
}

pub fn export_csv(path: &path::Path, table: &table::Table) -> Result<(), Error> {
  Ok(())
}

pub fn export_json(path: &path::Path, table: &table::Table) -> Result<(), Error> {
  Ok(())
}

fn read_file(path: &path::Path) -> Result<Vec<String>, Error> {
  // io::Result<io::Lines<io::BufReader<fs::File>>> <-- original return type
  // possible new return type? --> Result<io::Lines<io::BufReader<fs::File>>, Error>
  // Understand rust Error's and write a better Error type?
  match fs::File::open(path) {
    Ok(file) => {
      let mut res = Vec::<String>::new();
      let lines = io::BufReader::new(file).lines();
      for (_, line_result) in lines.enumerate() {
        match line_result {
          Ok(line) => res.push(line),
          Err(io_error) => {
            return Err(Error::new(
              path.to_str().unwrap().to_string(),
              "Failed to read line.".to_string(),
            ))
          }
        }
      }
      Ok(res)
    }
    Err(io_error) => Err(Error::new(
      path.to_str().unwrap().to_string(),
      "Failed to read file.".to_string(),
    )),
  }
}

fn write_file(path: &path::Path, contents: String) -> Result<(), Error> {
  unimplemented!()
}

fn fmt_header_csv(header: &table::Header) -> String {
  unimplemented!();
}

fn fmt_rows_csv(rows: &table::Rows) -> String {
  unimplemented!();
}
