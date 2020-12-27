use crate::file;
// use crate::query::querier;
use async_trait::async_trait;
// use serde::{Deserialize, Deserializer, Serialize, Serializer};
use ansi_term::Color;
use std::{clone, fmt, path};
use tokio_postgres::error as tokio_errors;

type Widths = Vec<usize>;
pub type EntryType = String; // make into tokio_postgres::types::Type
pub type Header = Vec<(String, EntryType)>;
pub type Row = Vec<String>;
pub type Rows = Vec<Row>;
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
enum SubError {
  BaseError,
  FileError(file::Error),
  PostgresError(tokio_postgres::error::Error),
}

#[derive(Debug)]
pub struct Error {
  description: String,
  sub_error: SubError,
}

impl Error {
  fn new(description: String, sub_error: SubError) -> Self {
    Self {
      description: description,
      sub_error: sub_error,
    }
  }
}

impl From<file::Error> for Error {
  fn from(error: file::Error) -> Self {
    Self {
      description: "File Error.".to_string(),
      sub_error: SubError::FileError(error),
    }
  }
}

impl From<tokio_errors::Error> for Error {
  fn from(error: tokio_errors::Error) -> Self {
    Self {
      description: "Postgres Error.".to_string(),
      sub_error: SubError::PostgresError(error),
    }
  }
}

impl std::fmt::Display for Error {
  fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(formatter, "Error: {:#?}", self)
  }
}

impl std::error::Error for Error {}

#[derive(Debug)]
pub struct Table {
  pub name: Option<String>,
  pub header: Header,
  pub rows: Rows,
  widths: Widths,
}

impl Table {
  pub fn new(header: Header, rows: Rows) -> Self {
    let widths = get_widths(&header, &rows);
    Self {
      name: None,
      header: header,
      rows: rows,
      widths: widths,
    }
  }

  pub fn with_header(header: Header) -> Self {
    // Fix this up
    Self {
      name: None,
      header: header,
      rows: Vec::new(),
      widths: Vec::new(),
    }
  }

  pub fn set_name(&mut self, name: String) {
    self.name = Some(name);
  }

  pub fn _set_header(&mut self, header: Header) {
    self.header = header;
  }

  pub fn _set_rows(&mut self, rows: Rows) {
    self.rows = rows;
  }

  fn _fmt_table_name(name: &Option<String>) -> String {
    match name {
      Some(table_name) => format!("{}", Color::Yellow.underline().bold().paint(table_name)),
      None => "".to_string(),
    }
  }

  fn fmt_row(widths: &Widths, row: &Row) -> String {
    let zipper = widths.iter().zip(row.iter());
    format!(
      "|{}|",
      zipper
        // Adding space of padding on both sides
        .map(|(width, entry)| format!(" {}{} ", entry, " ".repeat(width - entry.len())))
        .collect::<Row>()
        .join("|")
    )
  }

  fn fmt_header(widths: &Widths, header: &Header) -> String {
    let header_as_row = header
      .clone()
      .into_iter()
      .map(|(column_header, _)| column_header)
      .collect::<Vec<_>>();
    let zipper = widths.iter().zip(header_as_row.iter());
    format!(
      "|{}|",
      zipper
        // Adding space of padding on both sides
        .map(|(width, entry)| {
          let formatted_header_entry = format!("{}", Color::Purple.bold().paint(entry));
          format!(
            " {}{} ",
            formatted_header_entry,
            " ".repeat(width - entry.len())
          )
        })
        .collect::<Row>()
        .join("|")
    )
  }

  fn fmt_rows(widths: &Widths, rows: &Rows) -> Vec<String> {
    rows
      .into_iter()
      .map(|row| Self::fmt_row(widths, row))
      .collect()
  }

  fn fmt_row_separator(widths: &Widths) -> String {
    let widths_len = widths.len();
    format!(
      "\n+{}+\n",
      "-".repeat(
        widths
          .into_iter()
          // widths_len - 1 is the number of | entry separators
          // width + 2 is the length of the formatted entry with a space on both sides for padding
          .fold(widths_len - 1, |acc, width| acc + width + 2)
      )
    )
  }
}

#[async_trait]
pub trait Purveyor {
  fn import(path: &path::Path) -> Result<Table>;
  fn export(&self, path: &path::Path, use_json: Option<bool>, index: Option<usize>) -> Result<()>;

  // async fn load<Q>(name: String, db_querier: Q) -> Result<Table>
  // where
  //   Q: querier::QuerierTrait + std::marker::Sync + std::marker::Send;

  // async fn store<Q>(&self, db_querier: Q) -> Result<()>
  // where
  //   Q: querier::QuerierTrait + std::marker::Sync + std::marker::Send;
}

#[async_trait]
impl Purveyor for Table {
  // Imports a csv table from user file system given a path
  fn import(path: &path::Path) -> Result<Self> {
    Ok(file::import_csv(path)?)
  }

  // Exports a Table to a csv or json file in the user file system
  fn export(&self, path: &path::Path, use_json: Option<bool>, index: Option<usize>) -> Result<()> {
    // If no boolean is given then resolve using path file extensions
    if use_json == None {
      return Ok(file::export(index, path, self)?);
    }

    // If explicit boolean given then can just skip file extension resolution
    // TODO: remove this and replace with just file::export, since that has parent path validation
    if use_json.unwrap() {
      Ok(file::export_json(path, self)?)
    } else {
      Ok(file::export_csv(path, self)?)
    }
  }
}

impl fmt::Display for Table {
  fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    let formatted_row_separator = Table::fmt_row_separator(&self.widths);
    let formatted_header = Table::fmt_header(&self.widths, &self.header);
    let formatted_rows = Table::fmt_rows(&self.widths, &self.rows);
    // name
    // ----
    // head
    // ----
    // rows
    // ----
    write!(
      formatter,
      "{}{}{}{}{}",
      formatted_row_separator.strip_prefix("\n").unwrap(),
      formatted_header,
      formatted_row_separator,
      formatted_rows.join(formatted_row_separator.as_str()),
      formatted_row_separator
    )
  }
}

impl clone::Clone for Table {
  fn clone(&self) -> Self {
    Self {
      name: self.name.clone(),
      header: self.header.clone(),
      rows: self.rows.clone(),
      widths: self.widths.clone(),
    }
  }
}

pub fn _vec_to_table_string(column_name: &str, vector: &Vec<String>) -> String {
  // TODO include a type
  let header = vec![(String::from(column_name), String::from("VARCHAR(256)"))];
  let rows = vector
    .into_iter()
    .map(|entry| vec![String::from(entry)])
    .collect::<Vec<_>>();
  if rows.len() == 0 {
    "There are no tables in this database.".to_string()
  } else {
    format!("{}", Table::new(header, rows))
  }
}

fn get_widths(header: &Header, rows: &Rows) -> Widths {
  let column_headers = header
    .into_iter()
    .map(|(column_header, _type)| column_header.len())
    .collect::<Vec<_>>();
  rows
    .into_iter()
    .map(|row| {
      row
        .into_iter()
        .map(|column_entry| column_entry.len())
        .collect::<Vec<usize>>()
    })
    .fold(column_headers, |acc, row| {
      let zipper = acc.iter().zip(row.iter());
      let mut res = Vec::new();
      for (_, (acc_col, row_col)) in zipper.enumerate() {
        res.push(std::cmp::max(*acc_col, *row_col));
      }
      res
    })
}
