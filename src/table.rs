use crate::file;
use crate::query::querier;
use async_trait::async_trait;
// use serde::{Deserialize, Deserializer, Serialize, Serializer};
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

  pub fn _set_name(&mut self, name: String) {
    self.name = Some(name);
  }

  pub fn _set_header(&mut self, header: Header) {
    self.header = header;
  }

  pub fn _set_rows(&mut self, rows: Rows) {
    self.rows = rows;
  }

  fn fmt_row(widths: Widths, row: Row) -> String {
    let zipper = widths.iter().zip(row.iter());
    zipper
      .map(|(width, entry)| format!("{}{}", entry, " ".repeat(width - entry.len())))
      .collect::<Row>()
      .join("|")
      .to_string()
  }
}

#[async_trait]
pub trait Purveyor {
  fn import(path: &path::Path) -> Result<Table>;
  fn export(&self, path: &path::Path, use_json: bool) -> Result<()>;
  async fn load<Q>(name: String, db_querier: Q) -> Result<Table>
  where
    Q: querier::QuerierTrait + std::marker::Sync + std::marker::Send;
  async fn store<Q>(&self, db_querier: Q) -> Result<()>
  where
    Q: querier::QuerierTrait + std::marker::Sync + std::marker::Send;
}

#[async_trait]
impl Purveyor for Table {
  // Imports a csv table from user file system given a path
  fn import(path: &path::Path) -> Result<Self> {
    Ok(file::import_csv(path)?)
  }

  // Exports a Table to a csv or json file in the user file system
  fn export(&self, path: &path::Path, use_json: bool) -> Result<()> {
    if use_json {
      Ok(file::export_json(path, self)?)
    } else {
      Ok(file::export_csv(path, self)?)
    }
  }

  // Loads a Table from the database
  async fn load<Q>(name: String, db_querier: Q) -> Result<Self>
  where
    Q: querier::QuerierTrait + std::marker::Sync + std::marker::Send,
  {
    Ok(db_querier.load(name.as_str()).await?)
  }

  // Stores a Table to the database
  async fn store<Q>(&self, db_querier: Q) -> Result<()>
  where
    Q: querier::QuerierTrait + std::marker::Sync + std::marker::Send,
  {
    match self.name.clone() {
      Some(name) => Ok(
        db_querier
          .store(name.as_str(), self.header.clone(), self.rows.clone())
          .await?,
      ),
      None => Err(Error::new(
        "No table name given.".to_string(),
        SubError::BaseError,
      )),
    }
  }
}

impl fmt::Display for Table {
  fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    // should probably throw an error if len of row/widths is 0
    let formatted_header = Table::fmt_row(
      self.widths.clone(),
      self
        .header
        .clone()
        .into_iter()
        .map(|(column_header, _)| column_header)
        .collect::<Vec<_>>(),
    );
    let widths_len = self.widths.len();
    let separator = "-".repeat(
      self
        .widths
        .clone()
        .into_iter()
        .fold(widths_len - 1, |acc, width| acc + width),
    );
    let formatted_rows = self
      .rows
      .clone()
      .into_iter()
      .map(|row| Table::fmt_row(self.widths.clone(), row))
      .collect::<Vec<_>>();
    write!(
      formatter,
      "{}\n{}\n{}\n",
      formatted_header,
      separator,
      formatted_rows.join("\n")
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
