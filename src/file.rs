use crate::table;
use serde::{Deserialize, Serialize};
use std;
use std::path;

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

pub fn import_csv(path: std::path::PathBuf) -> Result<table::Table, Error> {
  Err(Error {
    path: path::Path::new("foo.txt").to_path_buf(),
    message: "".to_string(),
  })
}

pub fn export_csv(path: std::path::PathBuf, table: &table::Table) -> Result<(), Error> {
  Ok(())
}

pub fn export_json(path: std::path::PathBuf, table: &table::Table) -> Result<(), Error> {
  Ok(())
}

fn read_file(path: std::path::PathBuf) -> Result<String, Error> {
  unimplemented!()
}

fn write_file(path: std::path::PathBuf, contents: String) -> Result<(), Error> {
  unimplemented!()
}

fn fmt_header_csv(header: &table::Header) -> String {
  unimplemented!();
}

fn fmt_rows_csv(rows: &table::Rows) -> String {
  unimplemented!();
}
