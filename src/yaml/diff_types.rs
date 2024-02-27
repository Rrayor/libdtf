use serde_yaml::Mapping;

use crate::core::diff_types::{Diff, DiffCollection, Stringable, WorkingContext};

impl Stringable for serde_yaml::Value {
    fn to_string(&self) -> String {
        match self {
            serde_yaml::Value::Null => "null".to_owned(),
            value => serde_yaml::to_string(value)
                .unwrap_or_default()
                .trim()
                .to_owned(),
        }
    }
}

/// Holds the data required to run a difference check
pub struct CheckingData<'a, T: Diff> {
    /// Holds the collected differences
    pub diffs: DiffCollection<T>,
    /// Holds the key of the field currently checked - empty if it's the outermost object
    pub key: &'a str,
    /// One of the 2 objects that should be checked
    pub a: &'a Mapping,
    /// One of the 2 objects that should be checked
    pub b: &'a Mapping,
    /// Holds relevant data for the current run, such as file names, and user configs
    pub working_context: &'a WorkingContext,
}

impl<'a, T: Diff> CheckingData<'a, T> {
    pub fn new(
        key: &'a str,
        a: &'a Mapping,
        b: &'a Mapping,
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