/// Collects differences between the arrays of 2 data sets.
/// Stores `ArrayDiff` values
///
/// 1. First we check if the user has specified the option that states, that arrays should be in the same order. If the option is turned on, we don't do anything. The array will be checked for value differences instead.
/// 2. We iterate through object `a` and if a field is present in `b` as well, only then do we take action
///     1. We construct a new key. If we have a key in our checker object, than we add the currently checked fields key to it after a '.'. That's how we handle the keys of nested objects.
///     2. If `a` and `b` are both objects we recursively start the process over for the nested objects.
///     3. If both fields are arrays, we collect the differences:
///         * `AHas` and `BMisses` type of `ArrayDiffDesc` vectors, for values, that are present in `a` but not in `b`
///         * `BHas` and `AMisses` type of `ArrayDiffDesc` vectors, for values, that are present in `b` but not in `a`
///     4. We iterate through all the collected vectors and create `ArrayDiff` objects for each of them, which we store in our `diffs` vector
use std::collections::HashMap;

use serde_yaml::Value;

use crate::core::diff_types::{ArrayDiff, ArrayDiffDesc, Checker, DiffCollection, Stringable};

use super::{diff_types::CheckingData, format_key};

impl<'a> Checker<ArrayDiff> for CheckingData<'a, ArrayDiff> {
    fn check(&mut self) {
        if !self.working_context.config.array_same_order {
            for (a_key, a_value) in self.a.into_iter() {
                if let Some(b_value) = self.b.get(a_key) {
                    self.find_array_diffs_in_values(
                        &format_key(self.key, a_key.as_str().unwrap()),
                        a_value,
                        b_value,
                    );
                }
            }
        }
    }

    fn check_and_get(&mut self) -> &DiffCollection<ArrayDiff> {
        self.check();
        &self.diffs
    }

    fn diffs(&self) -> &Vec<ArrayDiff> {
        self.diffs.diffs()
    }
}

impl<'a> CheckingData<'a, ArrayDiff> {
    fn find_array_diffs_in_values(&mut self, key_in: &str, a: &Value, b: &Value) {
        if a.is_mapping() && b.is_mapping() {
            self.find_array_diffs_in_objects(key_in, a, b);
        }

        if a.is_sequence() && b.is_sequence() {
            let (a_has, a_misses, b_has, b_misses) =
                self.count_occurrences(a.as_sequence().unwrap(), b.as_sequence().unwrap());

            let array_diff_iter = a_has
                .iter()
                .map(|v| (v, ArrayDiffDesc::AHas))
                .chain(a_misses.iter().map(|v| (v, ArrayDiffDesc::AMisses)))
                .chain(b_has.iter().map(|v| (v, ArrayDiffDesc::BHas)))
                .chain(b_misses.iter().map(|v| (v, ArrayDiffDesc::BMisses)))
                .map(|(value, desc)| {
                    ArrayDiff::new(
                        key_in.to_owned(),
                        desc,
                        value
                            .as_str()
                            .map_or_else(|| value.as_str().unwrap().to_string(), |v| v.to_owned()),
                    )
                });

            self.diffs.extend(array_diff_iter);
        }
    }

    fn count_occurrences<T: PartialEq + Stringable>(
        &mut self,
        a: &[T],
        b: &[T],
    ) -> (Vec<Value>, Vec<Value>, Vec<Value>, Vec<Value>) {
        let ocurrence_counts_a = self.count_items(a);
        let ocurrence_counts_b = self.count_items(b);

        let a_has = self.calculate_difference(&ocurrence_counts_a, &ocurrence_counts_b);
        let b_has = self.calculate_difference(&ocurrence_counts_b, &ocurrence_counts_a);

        let a_misses = b_has.clone();
        let b_misses = a_has.clone();

        (a_has, a_misses, b_has, b_misses)
    }

    fn count_items<T: PartialEq + Stringable>(&self, items: &[T]) -> HashMap<String, i32> {
        let mut occurrence_counts = HashMap::new();

        for item in items {
            *occurrence_counts.entry(item.to_string()).or_insert(0) += 1;
        }

        occurrence_counts
    }

    fn calculate_difference(
        &self,
        ocurrence_counts_a: &HashMap<String, i32>,
        ocurrence_counts_b: &HashMap<String, i32>,
    ) -> Vec<Value> {
        let mut difference = vec![];

        for (key, count) in ocurrence_counts_a.iter() {
            let count_b = ocurrence_counts_b.get(key).copied().unwrap_or(0);
            let diff = count - count_b;

            for _ in 0..diff {
                difference.push(Value::String(key.to_owned()));
            }
        }

        difference
    }

    fn find_array_diffs_in_objects(&mut self, key_in: &str, a: &Value, b: &Value) {
        let mut array_checker = CheckingData::new(
            key_in,
            a.as_mapping().unwrap(),
            b.as_mapping().unwrap(),
            self.working_context,
        );

        array_checker.check();
        self.diffs.concatenate(&mut array_checker.diffs);
    }
}

#[cfg(test)]
mod tests {
    use serde_yaml::{from_str, Mapping};

    use crate::core::diff_types::{
        ArrayDiff, ArrayDiffDesc, Checker, Config, WorkingContext, WorkingFile,
    };

    use super::CheckingData;

    const FILE_NAME_A: &str = "a.json";
    const FILE_NAME_B: &str = "b.json";

    #[test]
    fn test_find_array_diffs() {
        // arrange
        let a = from_str(
            r"
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
            ArrayDiff::new("diff_array".to_owned(), ArrayDiffDesc::AHas, "3".to_owned()),
            ArrayDiff::new(
                "diff_array".to_owned(),
                ArrayDiffDesc::BMisses,
                "3".to_owned(),
            ),
            ArrayDiff::new("diff_array".to_owned(), ArrayDiffDesc::BHas, "8".to_owned()),
            ArrayDiff::new(
                "diff_array".to_owned(),
                ArrayDiffDesc::AMisses,
                "8".to_owned(),
            ),
            ArrayDiff::new(
                "nested.diff_array".to_owned(),
                ArrayDiffDesc::AHas,
                "3".to_owned(),
            ),
            ArrayDiff::new(
                "nested.diff_array".to_owned(),
                ArrayDiffDesc::BMisses,
                "3".to_owned(),
            ),
            ArrayDiff::new(
                "nested.diff_array".to_owned(),
                ArrayDiffDesc::BHas,
                "8".to_owned(),
            ),
            ArrayDiff::new(
                "nested.diff_array".to_owned(),
                ArrayDiffDesc::AMisses,
                "8".to_owned(),
            ),
        ];

        let working_context = create_test_working_context(false);
        let mut array_checker = CheckingData::new("", &a, &b, &working_context);

        // act
        array_checker.check();

        // assert
        assert_array(&expected, array_checker.diffs());
    }

    #[test]
    fn test_find_array_diffs_multiple_entries_with_same_value() {
        // arrange
        let a: Mapping = from_str(
            r"
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
            "
            'no_diff_array':
                - 1
                - 2
                - 3
                - 4
            'diff_array':
                - 1
                - 1
                - 2
                - 3
                - 3
                - 3
                - 4
            'nested':
                'no_diff_array':
                    - 1
                    - 2
                    - 3
                    - 4
                'diff_array':
                    - 1
                    - 1
                    - 2
                    - 3
                    - 3
                    - 3
                    - 4
        ",
        )
        .unwrap();

        let expected = vec![
            ArrayDiff::new("diff_array".to_owned(), ArrayDiffDesc::BHas, "1".to_owned()),
            ArrayDiff::new("diff_array".to_owned(), ArrayDiffDesc::BHas, "3".to_owned()),
            ArrayDiff::new("diff_array".to_owned(), ArrayDiffDesc::BHas, "3".to_owned()),
            ArrayDiff::new(
                "diff_array".to_owned(),
                ArrayDiffDesc::AMisses,
                "1".to_owned(),
            ),
            ArrayDiff::new(
                "diff_array".to_owned(),
                ArrayDiffDesc::AMisses,
                "3".to_owned(),
            ),
            ArrayDiff::new(
                "diff_array".to_owned(),
                ArrayDiffDesc::AMisses,
                "3".to_owned(),
            ),
            ArrayDiff::new(
                "nested.diff_array".to_owned(),
                ArrayDiffDesc::BHas,
                "1".to_owned(),
            ),
            ArrayDiff::new(
                "nested.diff_array".to_owned(),
                ArrayDiffDesc::BHas,
                "3".to_owned(),
            ),
            ArrayDiff::new(
                "nested.diff_array".to_owned(),
                ArrayDiffDesc::BHas,
                "3".to_owned(),
            ),
            ArrayDiff::new(
                "nested.diff_array".to_owned(),
                ArrayDiffDesc::AMisses,
                "1".to_owned(),
            ),
            ArrayDiff::new(
                "nested.diff_array".to_owned(),
                ArrayDiffDesc::AMisses,
                "3".to_owned(),
            ),
            ArrayDiff::new(
                "nested.diff_array".to_owned(),
                ArrayDiffDesc::AMisses,
                "3".to_owned(),
            ),
        ];

        let working_context = create_test_working_context(false);
        let mut array_checker = CheckingData::new("", &a, &b, &working_context);

        // act
        array_checker.check();

        // assert
        assert_array(&expected, array_checker.diffs());
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
