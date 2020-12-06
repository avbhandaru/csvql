use crate::file;
use serde::{Deserialize, Serialize};
use std::clone;
use std::fmt;
use std::path::PathBuf;

pub type EntryType = String; // make into tokio_postgres::types::Type
pub type Header = Vec<(String, EntryType)>;
pub type Rows = Vec<Vec<String>>;
type Row = Vec<String>;
type Widths = Vec<usize>;

#[derive(Deserialize, Serialize, Debug)]
pub struct Table {
  pub header: Header,
  pub rows: Rows,
  widths: Widths,
}

impl Table {
  pub fn new(header: Header, rows: Rows) -> Self {
    let widths = get_widths(&header, &rows);
    Self {
      header: header,
      rows: rows,
      widths: widths,
    }
  }

  pub fn import(path: PathBuf) -> Result<Self, file::Error> {
    file::import_csv(path)
  }

  pub fn export(&self, path: PathBuf, use_json: bool) -> Result<(), file::Error> {
    if use_json {
      file::export_json(path, self)
    } else {
      file::export_csv(path, self)
    }
  }

  pub fn fmt_row(widths: Widths, row: Row) -> String {
    let zipper = widths.iter().zip(row.iter());
    zipper
      .map(|(width, entry)| format!("{}{}", entry, " ".repeat(width - entry.len())))
      .collect::<Row>()
      .join("|")
      .to_string()
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
