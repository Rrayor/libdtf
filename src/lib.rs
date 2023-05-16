use diff_types::{
    ArrayDiff, Checker, CheckingData, DiffCollection, KeyDiff, TypeDiff, ValueDiff, WorkingContext,
};
use serde_json::{Map, Result, Value};
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

// TODO: remove. For testing only
struct ApiConsumer<'a> {
    key_checker: &'a mut CheckingData<'a, KeyDiff>,
    type_checker: &'a CheckingData<'a, TypeDiff>,
    value_checker: &'a CheckingData<'a, ValueDiff>,
    array_checker: &'a CheckingData<'a, ArrayDiff>,
}

impl<'a> ApiConsumer<'a> {
    pub fn use_api(
        &mut self,
        a: &Map<String, Value>,
        b: &Map<String, Value>,
        working_context: &WorkingContext,
    ) -> Option<&[KeyDiff]> {
        Some(self.key_checker.check_and_get().diffs())
    }
}
