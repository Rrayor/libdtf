use serde_json::{Map, Value};

use crate::core::diff_types::{Diff, DiffCollection, WorkingContext};

/// Holds the data required to run a difference check
pub struct CheckingData<'a, T: Diff> {
    /// Holds the collected differences
    pub diffs: DiffCollection<T>,
    /// Holds the key of the field currently checked - empty if it's the outermost object
    pub key: &'a str,
    /// One of the 2 objects that should be checked
    pub a: &'a Map<String, Value>,
    /// One of the 2 objects that should be checked
    pub b: &'a Map<String, Value>,
    /// Holds relevant data for the current run, such as file names, and user configs
    pub working_context: &'a WorkingContext,
}

impl<'a, T: Diff> CheckingData<'a, T> {
    pub fn new(
        key: &'a str,
        a: &'a Map<String, Value>,
        b: &'a Map<String, Value>,
        working_context: &'a WorkingContext,
    ) -> CheckingData<'a, T> {
        let diff_collection: DiffCollection<T> = DiffCollection::new();
        CheckingData {
            diffs: diff_collection,
            key,
            a,
            b,
            working_context,
        }
    }
}