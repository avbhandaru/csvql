// use regex::Regex;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::{Path, PathBuf, MAIN_SEPARATOR};

// TODO move this to utils?

pub fn get_table_name_from_path(path: &str) -> String {
  // TODO Handle all errors here
  // This is very hacky
  let extension = Path::new(path).extension().unwrap().to_str().unwrap();
  let path_without_extension = path.strip_suffix(extension).unwrap();
  let path_component_vectors = path_without_extension.split(MAIN_SEPARATOR);
  let file_name = path_component_vectors.last().unwrap();
  let table_name = file_name.replace('.', "_");
  table_name.to_string()
}

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
