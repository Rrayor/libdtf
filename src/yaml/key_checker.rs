/// Collects differences between the keys of 2 data sets.
/// Stores `KeyDiff` values
///
/// 1. First we store all the keys in the `b` object into a set called `b_keys`
/// 2. Then we go through all the fields of object `a`
///     1. We construct a new key. If we have a key in our checker object, than we add the currently checked fields key to it after a '.'. That's how we handle the keys of nested objects.
///     2. If the key is in `b_keys`, we remove it from there
///         * If the field is an object, we recursively call the same comparison and go through the new object
///         * If the field is an array and the user defined the option that arrays have to be in the same order we iterate through the array and recursively repeat the checking process for each item. If we can't assume, that the arrays are in the same order, than this check is pointless.
///     3. If the key is not present in `b_keys`, we save it to the `diffs` vector
/// 3. After checking `a` we add all the remaining keys in `b_keys` to the diff vector, if they weren't removed, they aren't in a.
use std::collections::HashSet;

use serde_yaml::Value;

use crate::core::diff_types::{Checker, DiffCollection, KeyDiff};

use super::{diff_types::CheckingData, format_key};

impl<'a> Checker<KeyDiff> for CheckingData<'a, KeyDiff> {
    fn check(&mut self) {
        let mut b_keys = self.get_b_keys();
        self.check_a(&mut b_keys);
        self.check_b(&b_keys);
    }

    fn check_and_get(&mut self) -> &DiffCollection<KeyDiff> {
        self.check();
        &self.diffs
    }

    fn diffs(&self) -> &Vec<KeyDiff> {
        self.diffs.diffs()
    }
}

impl<'a> CheckingData<'a, KeyDiff> {
    fn find_key_diffs_in_values(&mut self, key_in: &str, a: &Value, b: &Value) {
        if a.is_mapping() && b.is_mapping() {
            self.find_key_diffs_in_objects(key_in, a, b);
        }

        if self.working_context.config.array_same_order
            && a.is_sequence()
            && b.is_sequence()
            && a.as_sequence().unwrap().len() == b.as_sequence().unwrap().len()
        {
            self.find_key_diffs_in_arrays(key_in, a, b);
        }
    }

    fn find_key_diffs_in_objects(&mut self, key_in: &str, a: &Value, b: &Value) {
        let mut key_checker = CheckingData::new(
            key_in,
            a.as_mapping().unwrap(),
            b.as_mapping().unwrap(),
            self.working_context,
        );

        key_checker.check();
        self.diffs.concatenate(&mut key_checker.diffs);
    }

    fn find_key_diffs_in_arrays(&mut self, key_in: &str, a: &Value, b: &Value) {
        a.as_mapping()
            .unwrap()
            .iter()
            .enumerate()
            .for_each(|(i, (a_key, _))| {
                self.find_key_diffs_in_values(
                    &format!("{}[{}]", key_in, i),
                    a_key,
                    &b.as_sequence().unwrap()[i],
                )
            });
    }

    fn get_b_keys(&self) -> HashSet<String> {
        self.b
            .into_iter()
            .map(|(key, _)| format_key(self.key, key.as_str().unwrap()))
            .collect()
    }

    fn check_a(&mut self, b_keys: &mut HashSet<String>) {
        for (a_key, a_value) in self.a.into_iter() {
            let key = format_key(self.key, a_key.as_str().unwrap());

            if let Some(b_value) = self.b.get(a_key) {
                b_keys.remove(&key);
                self.find_key_diffs_in_values(&key, a_value, b_value);
            } else {
                self.diffs.push(KeyDiff::new(
                    key,
                    self.working_context.file_a.name.clone(),
                    self.working_context.file_b.name.clone(),
                ));
            }
        }
    }

    fn check_b(&mut self, b_keys: &HashSet<String>) {
        let mut remainder = b_keys
            .iter()
            .map(|key| {
                KeyDiff::new(
                    key.to_owned(),
                    self.working_context.file_b.name.to_owned(),
                    self.working_context.file_a.name.to_owned(),
                )
            })
            .collect();

        self.diffs.append(&mut remainder);
    }
}

#[cfg(test)]
mod tests {
    use serde_yaml::{from_str, Mapping};

    use crate::{core::diff_types::{
        Checker, Config, KeyDiff, WorkingContext, WorkingFile,
    }, yaml::diff_types::CheckingData};

    const FILE_NAME_A: &str = "a.json";
    const FILE_NAME_B: &str = "b.json";

    #[test]
    fn test_key_checker() {
        // arrange
        let a: Mapping = from_str(
            r"
            'a_has': 'a_has'
            'both_have': 'both_have'
            'nested':
                'a_has': 'a_has'
                'both_have': 'both_have'
        ",
        )
        .unwrap();
        let b = from_str(
            r"
            'b_has': 'b_has'
            'both_have': 'both_have'
            'nested':
                'b_has': 'b_has'
                'both_have': 'both_have'
        ",
        )
        .unwrap();

        let expected = vec![
            KeyDiff::new(
                "a_has".to_owned(),
                FILE_NAME_A.to_owned(),
                FILE_NAME_B.to_owned(),
            ),
            KeyDiff::new(
                "nested.a_has".to_owned(),
                FILE_NAME_A.to_owned(),
                FILE_NAME_B.to_owned(),
            ),
            KeyDiff::new(
                "b_has".to_owned(),
                FILE_NAME_B.to_owned(),
                FILE_NAME_A.to_owned(),
            ),
            KeyDiff::new(
                "nested.b_has".to_owned(),
                FILE_NAME_B.to_owned(),
                FILE_NAME_A.to_owned(),
            ),
        ];

        let working_context = create_test_working_context(false);

        let mut key_checker = CheckingData::new("", &a, &b, &working_context);

        // act
        key_checker.check();

        // assert
        assert_array(&expected, key_checker.diffs());
    }

    // Test utils

    fn create_test_working_context(array_same_order: bool) -> WorkingContext {
        let config = Config::new(array_same_order);
        let working_file_a = WorkingFile::new(FILE_NAME_A.to_owned());
        let working_file_b = WorkingFile::new(FILE_NAME_B.to_owned());
        WorkingContext::new(working_file_a, working_file_b, config)
    }

    fn assert_array<T: PartialEq>(expected: &Vec<T>, result: &Vec<T>) {
        assert_eq!(expected.len(), result.len());
        assert!(expected.into_iter().all(|item| result.contains(&item)));
    }
}
