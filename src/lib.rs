use serde_json::{Map, Result, Value};
use std::fs::File;
use std::io::BufReader;

pub mod array_checker;
pub mod diff_types;
pub mod key_checker;
pub mod type_checker;
pub mod value_checker;

/// Reads in a json file
///
/// # Errors
/// Panics if the file cannot be opened.
pub fn read_json_file(file_path: &str) -> Result<Map<String, Value>> {
    let file =
        File::open(file_path).unwrap_or_else(|_| panic!("Could not open file {}", file_path));
    let reader = BufReader::new(file);
    let result = serde_json::from_reader(reader)?;
    Ok(result)
}

fn format_key(key_in: &str, current_key: &str) -> String {
    if key_in.is_empty() {
        current_key.to_owned()
    } else {
        format!("{}.{}", key_in, current_key)
    }
}
