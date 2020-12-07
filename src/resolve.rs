use regex::Regex;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::{Path, PathBuf};

// TODO figure out the cli for this? And how I want these queries to look?

pub fn sql_table_paths(query_file_path_buf: PathBuf) -> Vec<String> {
  let query_file_name = query_file_path_buf
    .clone()
    .into_os_string()
    .into_string()
    .unwrap();
  let query_file_path = query_file_path_buf.as_path();
  println!("resolving sql_table_paths of {}", query_file_name);
  let mut content: Vec<String> = Vec::new();
  match read_lines(query_file_path) {
    Ok(lines) => content = lines.map(Result::unwrap).collect::<Vec<_>>(),
    Err(msg) => println!("{}", msg),
  }
  return content;
}

// The output is wrapped in a Result to allow matching on errors
// Returns an Iterator to the Reader of the lines of the file.
fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
  P: AsRef<Path>,
{
  let file = File::open(filename)?;
  Ok(io::BufReader::new(file).lines())
}
