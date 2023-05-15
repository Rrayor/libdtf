use std::collections::HashSet;

use serde_json::Value;

use crate::{
    diff_types::{Checker, CheckingData, KeyDiff},
    format_key,
};

impl<'a> Checker<KeyDiff> for CheckingData<'a, KeyDiff> {
    fn check(&mut self) {
        let mut b_keys = self.get_b_keys();
        self.check_a(&mut b_keys);
        self.check_b(&b_keys);
    }

    fn diffs(&self) -> &Vec<KeyDiff> {
        &self.diffs
    }
}

impl<'a> CheckingData<'a, KeyDiff> {
    fn find_key_diffs_in_values(&mut self, key_in: &str, a: &Value, b: &Value) {
        if a.is_object() && b.is_object() {
            self.find_key_diffs_in_objects(key_in, a, b);
        }

        if self.working_context.config.array_same_order
            && a.is_array()
            && b.is_array()
            && a.as_array().unwrap().len() == b.as_array().unwrap().len()
        {
            self.find_key_diffs_in_arrays(key_in, a, b);
        }
    }

    fn find_key_diffs_in_objects(&mut self, key_in: &str, a: &Value, b: &Value) {
        let mut key_checker = CheckingData::new(
            key_in,
            a.as_object().unwrap(),
            b.as_object().unwrap(),
            self.working_context,
        );

        key_checker.check();
        self.diffs.append(&mut key_checker.diffs);
    }

    fn find_key_diffs_in_arrays(&mut self, key_in: &str, a: &Value, b: &Value) {
        a.as_array()
            .unwrap()
            .iter()
            .enumerate()
            .for_each(|(i, a_item)| {
                self.find_key_diffs_in_values(
                    &format!("{}[{}]", key_in, i),
                    a_item,
                    &b.as_array().unwrap()[i],
                )
            });
    }

    fn get_b_keys(&self) -> HashSet<String> {
        self.b
            .into_iter()
            .map(|(key, _)| format_key(self.key, key))
            .collect()
    }

    fn check_a(&mut self, b_keys: &mut HashSet<String>) {
        for (a_key, a_value) in self.a.into_iter() {
            let key = format_key(self.key, a_key);

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
    use serde_json::json;

    use crate::diff_types::{Checker, CheckingData, Config, KeyDiff, WorkingContext, WorkingFile};

    const FILE_NAME_A: &str = "a.json";
    const FILE_NAME_B: &str = "b.json";

    #[test]
    fn test_key_checker() {
        // arrange
        let a = json!({
            "a_has": "a_has",
            "both_have": "both_have",
            "nested": {
                "a_has": "a_has",
                "both_have": "both_have"
            }
        });
        let b = json!({
            "b_has": "b_has",
            "both_have": "both_have",
            "nested": {
                "b_has": "b_has",
                "both_have": "both_have"
            }
        });

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

        let mut key_checker = CheckingData::new(
            "",
            &a.as_object().unwrap(),
            &b.as_object().unwrap(),
            &working_context,
        );

        // act
        key_checker.check();

        // assert
        assert_array(&expected, &key_checker.diffs);
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
