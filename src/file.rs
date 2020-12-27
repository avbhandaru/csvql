use crate::table;
use crate::types;
use io::BufRead;
use regex;
// use serde::ser;
// use serde::{Serialize, Serializer};
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
  let line = read_first_line(path)?;
  if line.len() == 0 {
    Err(Error::new(
      path.to_str().unwrap().to_string(),
      "Failed to parse Table from file. Read in Zero lines.".to_string(),
    ))
  } else {
    let header = line
      .split(",")
      .map(|entry| {
        lazy_static! {
          // Match Key
          // C = <letter and or number strings of length at least 1>
          // S = <any number of space>
          // T = <any string of characters without commas>
          // Matches (C)S(T)S
          static ref RE: regex::Regex = regex::Regex::new(r"([a-zA-Z\d_\s]+)\(\s*([^,]+)\s*\)").unwrap();
        }
        if RE.is_match(entry) {
          let captures = RE.captures(entry).unwrap();
          let column_name = captures.get(1).map_or("null", |m| m.as_str()); // If column name is null then call it null
          let column_type = captures.get(2).map_or("TEXT", |m| m.as_str()); // Default is TEXT, else annotated type
          let is_valid_type = types::postgres::is_valid_type(&column_type);
          (
            column_name.to_string().replace(" ", "_"),
            if is_valid_type {
              column_type.to_string()
            } else {
              println!("Invalid SQL type annotation: {}. Defaulting column type to TEXT for column with name {}.", column_type, column_name);
              "TEXT".to_string()
            },
          )
        } else {
          // Default column type cast to SQL TEXT
          // Could make TEXT a constant
          // TODO: Add dynamic type guessing as well (or have that be enabled by user)
          let formatted_entry = entry.to_string().replace(" ", "_");
          (formatted_entry, "TEXT".to_string())
        }
      })
      .collect::<table::Header>();
    // let rows = lines[1..].to_vec(); // Only need the header, since we're using sql COPY
    Ok(table::Table::with_header(header))
  }
}

pub fn export(index: Option<usize>, path: &path::Path, table: &table::Table) -> Result<(), Error> {
  // Validate that file directory is real
  let mut absolute_path_buf;
  if !path.is_dir() {
    match path.parent() {
      None => (),
      Some(parent_path) => {
        if !parent_path.is_dir() {
          return Err(Error::new(
            path.to_str().unwrap().to_string(),
            "Failed to export query result. Invalid parent directory. Could not resolve."
              .to_string(),
          ));
        }
      }
    }
    // Check the file extension and decide how to export the file
    absolute_path_buf = path
      .parent()
      .map_or(path::Path::new("/"), |parent| parent)
      .canonicalize()
      .unwrap();
    absolute_path_buf.push(path.file_name().unwrap());
  } else {
    absolute_path_buf = path.canonicalize().unwrap();
    absolute_path_buf.push("temp");
    if index == None {
      return Err(Error::new(
        path.to_str().unwrap().to_string(),
        "Failed to export query result. Invalid path. No file name.".to_string(),
      ));
    }
    let out_index = index.unwrap();
    absolute_path_buf.set_file_name(format!("out_{}", out_index).as_str());
  }
  println!("PARENT PATH WITH TEMP: {:?}", absolute_path_buf);

  // Set file name and extensions
  match path.extension() {
    None => {
      let _ = absolute_path_buf.set_extension("csv");
    }
    Some(extension) => {
      let _ = absolute_path_buf.set_extension(extension);
    }
  }
  let absolute_path = absolute_path_buf.as_path();

  println!(
    "PATH {:?}, ABSOLUTE PATH!! {:?}, Extension {:?}",
    path,
    absolute_path,
    absolute_path_buf.extension()
  );

  match absolute_path.extension() {
    Some(os_str) => match os_str.to_str().unwrap() {
      "csv" => export_csv(absolute_path, table),
      "json" => export_json(absolute_path, table),
      _ => Err(Error::new(
        path.to_str().unwrap().to_string(),
        "Failed to export query result. Unsupported file extension. Must be .csv or .json."
          .to_string(),
      )),
    },
    None => {
      let e = Error::new(
        path.to_str().unwrap().to_string(),
        "Failed to export query result. Invalid path. File has no extension.".to_string(),
      );
      println!("ERROR :'( : {:?}", e);
      return Err(e);
    }
  }
}

pub fn export_csv(path: &path::Path, table: &table::Table) -> Result<(), Error> {
  let header = table
    .header
    .clone()
    .into_iter()
    // TODO: Uncomment below when support for type annotations is added
    // .map(|(col_name, col_type)| format!("{}({})", &col_name, &col_type))
    .map(|(col_name, col_type)| format!("{}({})", &col_name, &col_type.to_uppercase()))
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

fn _read_file(path: &path::Path) -> Result<Vec<String>, Error> {
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
