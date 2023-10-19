/// Collects differences between the values of 2 data sets.
/// Stores `ValueDiff` values
///
/// 1. We iterate through object `a` and if a field is present in `b` as well, only then do we take action
///     1. We construct a new key. If we have a key in our checker object, than we add the currently checked fields key to it after a '.'. That's how we handle the keys of nested objects.
///     2. If `a` and `b` are both objects we recursively start the process over for the nested objects.
///     3. If both fields are arrays and the user has specified, that arrays should be in the same order, we iterate through the arrays and recursively repeat the checking for each item. If the user hasn't specified the option, this part is pointless.
///     4. If the values of the fields aren't equal, we add the difference to our `diffs` vector.
use serde_yaml::Value;

use crate::yaml::{
    diff_types::{Checker, CheckingData, DiffCollection, Stringable, ValueDiff},
    format_key,
};

impl<'a> Checker<ValueDiff> for CheckingData<'a, ValueDiff> {
    fn check(&mut self) {
        for (a_key, a_value) in self.a.into_iter() {
            if let Some(b_value) = self.b.get(a_key) {
                self.find_value_diffs_in_values(
                    &format_key(self.key, a_key.as_str().unwrap()),
                    a_value,
                    b_value,
                );
            }
        }
    }

    fn check_and_get(&mut self) -> &DiffCollection<ValueDiff> {
        self.check();
        &self.diffs
    }

    fn diffs(&self) -> &Vec<ValueDiff> {
        self.diffs.diffs()
    }
}

impl<'a> CheckingData<'a, ValueDiff> {
    fn find_value_diffs_in_values(&mut self, key_in: &str, a: &Value, b: &Value) {
        if a.is_mapping() && b.is_mapping() {
            self.find_value_diffs_in_objects(key_in, a, b);
        } else if self.working_context.config.array_same_order
            && a.is_sequence()
            && b.is_sequence()
            && a.as_sequence().unwrap().len() == b.as_sequence().unwrap().len()
        {
            self.find_value_diffs_in_arrays(key_in, a, b);
        } else if a != b && !a.is_sequence() && !b.is_sequence() {
            self.diffs.push(ValueDiff::new(
                key_in.to_owned(),
                // String values are escaped by default if to_string() is called on them, so if it is a string, we call as_str() first.
                a.as_str().map_or_else(|| a.to_string(), |v| v.to_owned()),
                b.as_str().map_or_else(|| b.to_string(), |v| v.to_owned()),
            ));
        } else if a != b && a.is_sequence() && b.is_sequence() {
            self.diffs.push(ValueDiff::new(
                key_in.to_owned(),
                "Array differences present".to_owned(),
                "Array differences present".to_owned(),
            ))
        }
    }

    fn find_value_diffs_in_objects(&mut self, key_in: &str, a: &Value, b: &Value) {
        let mut value_checker = CheckingData::new(
            key_in,
            a.as_mapping().unwrap(),
            b.as_mapping().unwrap(),
            self.working_context,
        );

        value_checker.check();
        self.diffs.concatenate(&mut value_checker.diffs);
    }

    fn find_value_diffs_in_arrays(&mut self, key_in: &str, a: &Value, b: &Value) {
        for (index, a_item) in a.as_sequence().unwrap().iter().enumerate() {
            let array_key = format!("{}[{}]", key_in, index);
            self.find_value_diffs_in_values(&array_key, a_item, &b.as_sequence().unwrap()[index]);
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_yaml::from_str;

    use crate::yaml::diff_types::{Checker, Config, ValueDiff, WorkingContext, WorkingFile};

    use super::CheckingData;

    const FILE_NAME_A: &str = "a.json";
    const FILE_NAME_B: &str = "b.json";

    #[test]
    fn test_find_value_diffs_no_array_same_order() {
        // arrange
        let a = from_str(
            r"
            'no_diff_string': 'no_diff_string'
            'diff_string': 'a'
            'no_diff_number': 'no_diff_number'
            'diff_number': 1
            'no_diff_boolean': true
            'diff_boolean': true
            'no_diff_array':
                - 1
                - 2
                - 3
                - 4
            'diff_array':
                - 1
                - 2
                - 3
                - 4
            'nested':
                'no_diff_string': 'no_diff_string'
                'diff_string': 'a'
                'no_diff_number': 'no_diff_number'
                'diff_number': 1
                'no_diff_boolean': true
                'diff_boolean': true
                'no_diff_array':
                    - 1
                    - 2
                    - 3
                    - 4
                'diff_array':
                    - 1
                    - 2
                    - 3
                    - 4
        ",
        )
        .unwrap();

        let b = from_str(
            r"
            'no_diff_string': 'no_diff_string'
            'diff_string': 'b'
            'no_diff_number': 'no_diff_number'
            'diff_number': 2
            'no_diff_boolean': true
            'diff_boolean': false
            'no_diff_array':
                - 1
                - 2
                - 3
                - 4
            'diff_array':
                - 5
                - 6
                - 7
                - 8
            'nested':
                'no_diff_string': 'no_diff_string'
                'diff_string': 'b'
                'no_diff_number': 'no_diff_number'
                'diff_number': 2
                'no_diff_boolean': true
                'diff_boolean': false
                'no_diff_array':
                    - 1
                    - 2
                    - 3
                    - 4
                'diff_array':
                    - 5
                    - 6
                    - 7
                    - 8
        ",
        )
        .unwrap();

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
                "Array differences present".to_owned(),
                "Array differences present".to_owned(),
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
                "Array differences present".to_owned(),
                "Array differences present".to_owned(),
            ),
        ];

        let working_context = create_test_working_context(false);
        let mut value_checker = CheckingData::new("", &a, &b, &working_context);

        // act
        value_checker.check();

        // assert
        assert_array(&expected, value_checker.diffs());
    }

    #[test]
    fn test_find_value_diffs_array_same_order() {
        // arrange
        let a = from_str(
            r"
            'no_diff_string': 'no_diff_string'
            'diff_string': 'a'
            'no_diff_number': 'no_diff_number'
            'diff_number': 1
            'no_diff_boolean': true
            'diff_boolean': true
            'no_diff_array':
                - 1
                - 2
                - 3
                - 4
            'diff_array':
                - 1
                - 2
                - 3
                - 4
            'nested':
                'no_diff_string': 'no_diff_string'
                'diff_string': 'a'
                'no_diff_number': 'no_diff_number'
                'diff_number': 1
                'no_diff_boolean': true
                'diff_boolean': true
                'no_diff_array':
                    - 1
                    - 2
                    - 3
                    - 4
                'diff_array':
                    - 1
                    - 2
                    - 3
                    - 4
        ",
        )
        .unwrap();

        let b = from_str(
            r"
            'no_diff_string': 'no_diff_string'
            'diff_string': 'b'
            'no_diff_number': 'no_diff_number'
            'diff_number': 2
            'no_diff_boolean': true
            'diff_boolean': false
            'no_diff_array':
                - 1
                - 2
                - 3
                - 4
            'diff_array':
                - 1
                - 2
                - 8
                - 4
            'nested':
                'no_diff_string': 'no_diff_string'
                'diff_string': 'b'
                'no_diff_number': 'no_diff_number'
                'diff_number': 2
                'no_diff_boolean': true
                'diff_boolean': false
                'no_diff_array':
                    - 1
                    - 2
                    - 3
                    - 4
                'diff_array':
                    - 1
                    - 2
                    - 8
                    - 4
        ",
        )
        .unwrap();

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
        let mut value_checker = CheckingData::new("", &a, &b, &working_context);

        // act
        value_checker.check();

        // assert
        assert_array(&expected, value_checker.diffs());
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
