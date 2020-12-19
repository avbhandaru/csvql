use crate::table;
use io::BufRead;
use serde::ser;
use serde::{Serialize, Serializer};
use std::io::Write;
use std::{fs, io, path};

#[derive(Debug)]
pub struct Error {
  pub path: path::PathBuf,
  pub message: String,
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
  // let lines = read_file(path)?
  //   .into_iter()
  //   .map(|line| line.split(",").map(String::from).collect())
  //   .collect::<Vec<Vec<String>>>();
  // // println!("csv file:\n{:#?}", lines);
  // // remove later
  let line = read_first_line(path)?;
  if line.len() == 0 {
    Err(Error::new(
      path.to_str().unwrap().to_string(),
      "Failed to parse Table from file. Read in Zero lines.".to_string(),
    ))
  } else {
    let header = line
      .split(",")
      // TODO let type be dynamic and grab type annotations from csv headers
      .map(|entry| (String::from(entry), "VARCHAR(512)".to_string()))
      .collect::<table::Header>();
    // let rows = lines[1..].to_vec(); // Only need the header, since we're using sql COPY
    Ok(table::Table::with_header(header))
  }
}

pub fn export_csv(path: &path::Path, table: &table::Table) -> Result<(), Error> {
  let header = table
    .header
    .clone()
    .into_iter()
    // TODO: Uncomment below when support for type annotations is added
    // .map(|(col_name, col_type)| format!("{}({})", &col_name, &col_type))
    .map(|(col_name, _)| format!("{}", &col_name))
    .collect::<Vec<String>>()
    .join(",");
  let rows = table
    .rows
    .clone()
    .into_iter()
    .map(|row| row.join(","))
    .collect::<Vec<String>>()
    .join("\n");
  write_file(path, format!("{}\n{}\n", header, rows))
}

pub fn export_json(path: &path::Path, table: &table::Table) -> Result<(), Error> {
  // let contents = ser::to_string(table)?;
  // write_file(path, contents)?
  todo!();
  Ok(())
}

fn read_first_line(path: &path::Path) -> Result<String, Error> {
  match fs::File::open(path) {
    Ok(file) => {
      let mut buffer = String::new();
      io::BufReader::new(file)
        .read_line(&mut buffer)
        .expect("Reading first line of file.");
      Ok(buffer)
    }
    Err(_) => Err(Error::new(
      path.to_str().unwrap().to_string(),
      "Failed to read first line of file.".to_string(),
    )),
  }
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
          Err(_) => {
            // io_error
            return Err(Error::new(
              path.to_str().unwrap().to_string(),
              "Failed to read line.".to_string(),
            ));
          }
        }
      }
      Ok(res)
    }
    Err(_) => Err(Error::new(
      path.to_str().unwrap().to_string(),
      "Failed to read file.".to_string(),
    )),
  }
}

fn write_file(path: &path::Path, contents: String) -> Result<(), Error> {
  // could use fs::write(path, contents) if I wanted to overwrite contents
  // let file = fs::File::with_options().append(true).create(true).open(path);
  let mut file = match fs::OpenOptions::new().append(true).create(true).open(path) {
    Ok(file) => file,
    Err(_) => {
      // io_error
      return Err(Error::new(
        path.to_str().unwrap().to_string(),
        "Failed open file at path.".to_string(),
      ));
    }
  };
  match file.write_all(contents.as_bytes()) {
    Err(_) => {
      // io_error
      return Err(Error::new(
        path.to_str().unwrap().to_string(),
        "Failed to write_all to file.".to_string(),
      ));
    }
    _ => (),
  };
  match file.sync_all() {
    Err(_) => {
      // io_error
      return Err(Error::new(
        path.to_str().unwrap().to_string(),
        "Failed to sync_all to file system.".to_string(),
      ));
    }
    _ => (),
  };

  Ok(())
}
