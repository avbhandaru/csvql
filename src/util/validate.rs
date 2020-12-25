use std::fs;
use std::path;

// ValidatorError struct
#[derive(Debug)]
pub struct ValidatorError {
  pub message: String,
}

impl std::fmt::Display for ValidatorError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "Validation Error: {}", self.message)
  }
}

impl std::error::Error for ValidatorError {}

// Validator trait for any Rust type
pub trait Validate {
  type Output;
  type Wrapper;
  fn validate(&self) -> Self::Wrapper;
  fn validate_bool(&self) -> bool;
}

// Implementations for common Rust types

// std::path::Path
pub struct PathInfo {
  pub path: path::PathBuf,
  pub filename: Option<String>,
  pub extension: Option<String>,
}

impl Validate for path::Path {
  type Output = PathInfo;
  type Wrapper = Result<Self::Output, ValidatorError>;

  fn validate(&self) -> Self::Wrapper {
    let canonical = fs::canonicalize(self);
    match canonical {
      Ok(abs_path) => {
        let extension = match abs_path.extension() {
          Some(os_str) => Some(os_str.to_str().unwrap().to_string()),
          None => None,
        };
        let filename = if abs_path.is_dir() {
          None
        } else {
          let extension_str = abs_path.extension().unwrap().to_str().unwrap();
          abs_path
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .strip_suffix(format!(".{}", extension_str).as_str())
            .map(String::from)
        };
        // return the resulting PathInfo object
        Ok(PathInfo {
          path: abs_path,
          filename: filename,
          extension: extension,
        })
      }
      Err(e) => Err(ValidatorError {
        message: "Invalid Path. Cannot resolve.".to_string(),
      }),
    }
  }

  fn validate_bool(&self) -> bool {
    match self.validate() {
      Ok(_) => true,
      Err(_) => false,
    }
  }
}
