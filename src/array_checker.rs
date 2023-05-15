use std::fmt::Display;

use serde_json::Value;

use crate::{
    diff_types::{ArrayDiff, ArrayDiffDesc, Checker, CheckingData},
    format_key,
};

impl<'a> Checker<ArrayDiff> for CheckingData<'a, ArrayDiff> {
    fn check(&mut self) {
        if self.working_context.config.array_same_order {
            return;
        }

        for (a_key, a_value) in self.a.into_iter() {
            if let Some(b_value) = self.b.get(a_key) {
                self.find_array_diffs_in_values(&format_key(self.key, a_key), a_value, b_value);
            }
        }
    }

    fn diffs(&self) -> &Vec<ArrayDiff> {
        &self.diffs
    }
}

impl<'a> CheckingData<'a, ArrayDiff> {
    fn find_array_diffs_in_values(&mut self, key_in: &str, a: &Value, b: &Value) {
        if a.is_object() && b.is_object() {
            self.find_array_diffs_in_objects(key_in, a, b);
        }

        if a.is_array() && b.is_array() {
            let (a_has, a_misses, b_has, b_misses) =
                self.fill_diff_vectors(a.as_array().unwrap(), b.as_array().unwrap());

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
                            .map_or_else(|| value.to_string(), |v| v.to_owned()),
                    )
                });

            self.diffs.extend(array_diff_iter);
        }
    }

    fn fill_diff_vectors<T: PartialEq + Display>(
        &self,
        a: &'a [T],
        b: &'a [T],
    ) -> (Vec<&'a T>, Vec<&'a T>, Vec<&'a T>, Vec<&'a T>) {
        let a_has = a.iter().filter(|&x| !b.contains(x)).collect::<Vec<&T>>();
        let b_has = b.iter().filter(|&x| !a.contains(x)).collect::<Vec<&T>>();
        let a_misses = b.iter().filter(|&x| !a.contains(x)).collect::<Vec<&T>>();
        let b_misses = a.iter().filter(|&x| !b.contains(x)).collect::<Vec<&T>>();

        (a_has, a_misses, b_has, b_misses)
    }

    fn find_array_diffs_in_objects(&mut self, key_in: &str, a: &Value, b: &Value) {
        let mut array_checker = CheckingData::new(
            key_in,
            a.as_object().unwrap(),
            b.as_object().unwrap(),
            self.working_context,
        );

        array_checker.check();
        self.diffs.append(&mut array_checker.diffs);
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::diff_types::{
        ArrayDiff, ArrayDiffDesc, Checker, Config, WorkingContext, WorkingFile,
    };

    use super::CheckingData;

    const FILE_NAME_A: &str = "a.json";
    const FILE_NAME_B: &str = "b.json";

    #[test]
    fn test_find_array_diffs() {
        // arrange
        let a = json!({
            "no_diff_array": [
                1, 2, 3, 4
            ],
            "diff_array": [
                1, 2, 3, 4
            ],
            "nested": {
                "no_diff_array": [
                    1, 2, 3, 4
                ],
                "diff_array": [
                    1, 2, 3, 4
                ],
            },
        });

        let b = json!({
            "no_diff_array": [
                1, 2, 3, 4
            ],
            "diff_array": [
                1, 2, 8, 4
            ],
            "nested": {
                "no_diff_array": [
                    1, 2, 3, 4
                ],
                "diff_array": [
                    1, 2, 8, 4
                ],
            },
        });

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
        let mut array_checker = CheckingData::new(
            "",
            &a.as_object().unwrap(),
            &b.as_object().unwrap(),
            &working_context,
        );

        // act
        array_checker.check();

        // assert
        assert_array(&expected, &array_checker.diffs());
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
