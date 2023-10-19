use std::fs::File;
use std::io::BufReader;

mod array_checker;
pub mod diff_types;
mod key_checker;
mod type_checker;
mod value_checker;

/// Reads in a json file
///
/// # Errors
/// Panics if the file cannot be opened.
pub fn read_json_file(
    file_path: &str,
) -> serde_json::Result<serde_json::Map<String, serde_json::Value>> {
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
