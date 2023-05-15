use serde_json::Value;

use crate::{
    diff_types::{Checker, CheckingData, ValueDiff},
    format_key,
};

impl<'a> Checker<ValueDiff> for CheckingData<'a, ValueDiff> {
    fn check(&mut self) {
        for (a_key, a_value) in self.a.into_iter() {
            if let Some(b_value) = self.b.get(a_key) {
                self.find_value_diffs_in_values(&format_key(self.key, a_key), a_value, b_value);
            }
        }
    }

    fn diffs(&self) -> &Vec<ValueDiff> {
        &self.diffs
    }
}

impl<'a> CheckingData<'a, ValueDiff> {
    fn find_value_diffs_in_values(&mut self, key_in: &str, a: &Value, b: &Value) {
        if a.is_object() && b.is_object() {
            self.find_value_diffs_in_objects(key_in, a, b);
        } else if self.working_context.config.array_same_order
            && a.is_array()
            && b.is_array()
            && a.as_array().unwrap().len() == b.as_array().unwrap().len()
        {
            self.find_value_diffs_in_arrays(key_in, a, b);
        } else if a != b {
            self.diffs.push(ValueDiff::new(
                key_in.to_owned(),
                // String values are escaped by default if to_string() is called on them, so if it is a string, we call as_str() first.
                a.as_str().map_or_else(|| a.to_string(), |v| v.to_owned()),
                b.as_str().map_or_else(|| b.to_string(), |v| v.to_owned()),
            ));
        }
    }

    fn find_value_diffs_in_objects(&mut self, key_in: &str, a: &Value, b: &Value) {
        let mut value_checker = CheckingData::new(
            key_in,
            a.as_object().unwrap(),
            b.as_object().unwrap(),
            self.working_context,
        );

        value_checker.check();
        self.diffs.append(&mut value_checker.diffs);
    }

    fn find_value_diffs_in_arrays(&mut self, key_in: &str, a: &Value, b: &Value) {
        for (index, a_item) in a.as_array().unwrap().iter().enumerate() {
            let array_key = format!("{}[{}]", key_in, index);
            self.find_value_diffs_in_values(&array_key, a_item, &b.as_array().unwrap()[index]);
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::diff_types::{Checker, Config, ValueDiff, WorkingContext, WorkingFile};

    use super::CheckingData;

    const FILE_NAME_A: &str = "a.json";
    const FILE_NAME_B: &str = "b.json";

    #[test]
    fn test_find_value_diffs_no_array_same_order() {
        // arrange
        let a = json!({
            "no_diff_string": "no_diff_string",
            "diff_string": "a",
            "no_diff_number": "no_diff_number",
            "diff_number": 1,
            "no_diff_boolean": true,
            "diff_boolean": true,
            "no_diff_array": [
                1, 2, 3, 4
            ],
            "diff_array": [
                1, 2, 3, 4
            ],
            "nested": {
                "no_diff_string": "no_diff_string",
                "diff_string": "a",
                "no_diff_number": "no_diff_number",
                "diff_number": 1,
                "no_diff_boolean": true,
                "diff_boolean": true,
                "no_diff_array": [
                    1, 2, 3, 4
                ],
                "diff_array": [
                    1, 2, 3, 4
                ],
            },
        });

        let b = json!({
            "no_diff_string": "no_diff_string",
            "diff_string": "b",
            "no_diff_number": "no_diff_number",
            "diff_number": 2,
            "no_diff_boolean": true,
            "diff_boolean": false,
            "no_diff_array": [
                1, 2, 3, 4
            ],
            "diff_array": [
                5, 6, 7, 8
            ],
            "nested": {
                "no_diff_string": "no_diff_string",
                "diff_string": "b",
                "no_diff_number": "no_diff_number",
                "diff_number": 2,
                "no_diff_boolean": true,
                "diff_boolean": false,
                "no_diff_array": [
                    1, 2, 3, 4
                ],
                "diff_array": [
                    5, 6, 7, 8
                ],
            },
        });

        let expected = vec![
            ValueDiff::new("diff_string".to_owned(), "a".to_owned(), "b".to_owned()),
            ValueDiff::new("diff_number".to_owned(), "1".to_owned(), "2".to_owned()),
            ValueDiff::new(
                "diff_boolean".to_owned(),
                "true".to_owned(),
                "false".to_owned(),
            ),
            ValueDiff::new(
                "diff_array".to_owned(),
                "[1,2,3,4]".to_owned(),
                "[5,6,7,8]".to_owned(),
            ),
            ValueDiff::new(
                "nested.diff_string".to_owned(),
                "a".to_owned(),
                "b".to_owned(),
            ),
            ValueDiff::new(
                "nested.diff_number".to_owned(),
                "1".to_owned(),
                "2".to_owned(),
            ),
            ValueDiff::new(
                "nested.diff_boolean".to_owned(),
                "true".to_owned(),
                "false".to_owned(),
            ),
            ValueDiff::new(
                "nested.diff_array".to_owned(),
                "[1,2,3,4]".to_owned(),
                "[5,6,7,8]".to_owned(),
            ),
        ];

        let working_context = create_test_working_context(false);
        let mut value_checker = CheckingData::new(
            "",
            &a.as_object().unwrap(),
            &b.as_object().unwrap(),
            &working_context,
        );

        // act
        value_checker.check();

        // assert
        assert_array(&expected, &value_checker.diffs());
    }

    #[test]
    fn test_find_value_diffs_array_same_order() {
        // arrange
        let a = json!({
            "no_diff_string": "no_diff_string",
            "diff_string": "a",
            "no_diff_number": "no_diff_number",
            "diff_number": 1,
            "no_diff_boolean": true,
            "diff_boolean": true,
            "no_diff_array": [
                1, 2, 3, 4
            ],
            "diff_array": [
                1, 2, 3, 4
            ],
            "nested": {
                "no_diff_string": "no_diff_string",
                "diff_string": "a",
                "no_diff_number": "no_diff_number",
                "diff_number": 1,
                "no_diff_boolean": true,
                "diff_boolean": true,
                "no_diff_array": [
                    1, 2, 3, 4
                ],
                "diff_array": [
                    1, 2, 3, 4
                ],
            },
        });

        let b = json!({
            "no_diff_string": "no_diff_string",
            "diff_string": "b",
            "no_diff_number": "no_diff_number",
            "diff_number": 2,
            "no_diff_boolean": true,
            "diff_boolean": false,
            "no_diff_array": [
                1, 2, 3, 4
            ],
            "diff_array": [
                1, 2, 8, 4
            ],
            "nested": {
                "no_diff_string": "no_diff_string",
                "diff_string": "b",
                "no_diff_number": "no_diff_number",
                "diff_number": 2,
                "no_diff_boolean": true,
                "diff_boolean": false,
                "no_diff_array": [
                    1, 2, 3, 4
                ],
                "diff_array": [
                    1, 2, 8, 4
                ],
            },
        });

        let expected = vec![
            ValueDiff::new("diff_string".to_owned(), "a".to_owned(), "b".to_owned()),
            ValueDiff::new("diff_number".to_owned(), "1".to_owned(), "2".to_owned()),
            ValueDiff::new(
                "diff_boolean".to_owned(),
                "true".to_owned(),
                "false".to_owned(),
            ),
            ValueDiff::new("diff_array[2]".to_owned(), "3".to_owned(), "8".to_owned()),
            ValueDiff::new(
                "nested.diff_string".to_owned(),
                "a".to_owned(),
                "b".to_owned(),
            ),
            ValueDiff::new(
                "nested.diff_number".to_owned(),
                "1".to_owned(),
                "2".to_owned(),
            ),
            ValueDiff::new(
                "nested.diff_boolean".to_owned(),
                "true".to_owned(),
                "false".to_owned(),
            ),
            ValueDiff::new(
                "nested.diff_array[2]".to_owned(),
                "3".to_owned(),
                "8".to_owned(),
            ),
        ];

        let working_context = create_test_working_context(true);
        let mut value_checker = CheckingData::new(
            "",
            &a.as_object().unwrap(),
            &b.as_object().unwrap(),
            &working_context,
        );

        // act
        value_checker.check();

        // assert
        assert_array(&expected, &value_checker.diffs());
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
