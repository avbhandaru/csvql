use crate::table;
use std::process::{Command, Stdio};

// TODO use this functions for all tables with rows > 20 and for help and usage info
// TODO clean up this code/refactor to get rid of duplication.
pub fn file(path: &str) {
  Command::new("less")
    .arg(std::path::Path::new(path).as_os_str())
    .spawn()
    .expect(format!("Failed to spawn (less {}) process.", path).as_str())
    .wait()
    .expect(format!("Failed to wait on (less {})", path).as_str());
}

pub fn string(input: String) {
  // runs 'echo input | less'
  let echo = Command::new("echo")
    .arg(format!("{}", input).as_str())
    .stdout(Stdio::piped())
    .spawn()
    .expect("Failed to spawn (echo) process.");
  let echo_stdout = echo.stdout.expect("failed to open (echo.stdout).");
  Command::new("less")
    .stdin(Stdio::from(echo_stdout))
    .spawn()
    .expect("Failed to start (less) process.")
    .wait()
    .expect("Failed to wait on (less) process.");
}

pub fn table(table: &table::Table) {
  string(format!("{}", table));
}
