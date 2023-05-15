use serde_json::Value;

use crate::{
    diff_types::{Checker, CheckingData, TypeDiff, ValueType},
    format_key,
};

impl<'a> Checker<TypeDiff> for CheckingData<'a, TypeDiff> {
    fn check(&mut self) {
        for (a_key, a_value) in self.a.into_iter() {
            if let Some(b_value) = self.b.get(a_key) {
                self.find_type_diffs_in_values(&format_key(self.key, a_key), a_value, b_value);
            }
        }
    }

    fn diffs(&self) -> &Vec<TypeDiff> {
        &self.diffs
    }
}

impl<'a> CheckingData<'a, TypeDiff> {
    fn find_type_diffs_in_values(&mut self, key_in: &str, a: &Value, b: &Value) {
        if a.is_object() && b.is_object() {
            self.find_type_diffs_in_objects(key_in, a, b);
        }

        if self.working_context.config.array_same_order
            && a.is_array()
            && b.is_array()
            && a.as_array().unwrap().len() == b.as_array().unwrap().len()
        {
            self.find_type_diffs_in_arrays(key_in, a, b);
        }

        let a_type = get_type(a);
        let b_type = get_type(b);

        if a_type != b_type {
            self.diffs.push(TypeDiff::new(
                key_in.to_owned(),
                a_type.to_string(),
                b_type.to_string(),
            ));
        }
    }

    fn find_type_diffs_in_objects(&mut self, key_in: &str, a: &Value, b: &Value) {
        let mut type_checker = CheckingData::new(
            key_in,
            a.as_object().unwrap(),
            b.as_object().unwrap(),
            self.working_context,
        );

        type_checker.check();
        self.diffs.append(&mut type_checker.diffs);
    }

    fn find_type_diffs_in_arrays(&mut self, key_in: &str, a: &Value, b: &Value) {
        a.as_array()
            .unwrap()
            .iter()
            .enumerate()
            .for_each(|(i, a_item)| {
                self.find_type_diffs_in_values(
                    &format!("{}[{}]", key_in, i),
                    a_item,
                    &b.as_array().unwrap()[i],
                )
            });
    }
}

fn get_type(value: &Value) -> ValueType {
    match value {
        Value::Null => ValueType::Null,
        Value::Bool(_) => ValueType::Boolean,
        Value::Number(_) => ValueType::Number,
        Value::String(_) => ValueType::String,
        Value::Array(_) => ValueType::Array,
        Value::Object(_) => ValueType::Object,
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::diff_types::{Checker, Config, TypeDiff, WorkingContext, WorkingFile};

    use super::CheckingData;

    const FILE_NAME_A: &str = "a.json";
    const FILE_NAME_B: &str = "b.json";

    #[test]
    fn test_find_type_diffs_no_array_same_order() {
        // arrange
        let a = json!({
            "a_string_b_int": "a_string_b_int",
            "both_string": "both_string",
            "array_3_a_string_b_int": [
                "string",
                "string2",
                "string3",
                "string4",
                8,
                true
            ],
            "nested": {
                "a_bool_b_string": true,
                "both_number": 4,
                "array_3_a_int_b_bool": [
                    "string",
                    "string2",
                    "string3",
                    6,
                    8,
                    true
                ],
            }
        });
        let b = json!({
            "a_string_b_int": 2,
            "both_string": "both_string",
            "array_3_a_string_b_int": [
                "other_string",
                "other_string2",
                "other_string3",
                5,
                1,
                false
            ],
            "nested": {
                "a_bool_b_string": "a_bool_b_string",
                "both_number": 1,
                "array_3_a_int_b_bool": [
                "other_string",
                "other_string2",
                "other_string3",
                false,
                2,
                false
            ],
            }
        });

        let expected = vec![
            TypeDiff::new(
                "a_string_b_int".to_owned(),
                "string".to_owned(),
                "number".to_owned(),
            ),
            TypeDiff::new(
                "nested.a_bool_b_string".to_owned(),
                "bool".to_owned(),
                "string".to_owned(),
            ),
        ];

        let working_context = create_test_working_context(false);
        let mut type_checker = CheckingData::new(
            "",
            &a.as_object().unwrap(),
            &b.as_object().unwrap(),
            &working_context,
        );

        // act
        type_checker.check();

        // assert
        assert_array(&expected, &type_checker.diffs());
    }

    #[test]
    fn test_find_type_diffs_array_same_order() {
        // arrange
        let a = json!({
            "a_string_b_int": "a_string_b_int",
            "both_string": "both_string",
            "array_3_a_string_b_int": [
                "string",
                "string2",
                "string3",
                "string4",
                8,
                true
            ],
            "nested": {
                "a_bool_b_string": true,
                "both_number": 4,
                "array_3_a_int_b_bool": [
                    "string",
                    "string2",
                    "string3",
                    6,
                    8,
                    true
                ],
            }
        });
        let b = json!({
            "a_string_b_int": 2,
            "both_string": "both_string",
            "array_3_a_string_b_int": [
                "other_string",
                "other_string2",
                "other_string3",
                5,
                1,
                false
            ],
            "nested": {
                "a_bool_b_string": "a_bool_b_string",
                "both_number": 1,
                "array_3_a_int_b_bool": [
                "other_string",
                "other_string2",
                "other_string3",
                false,
                2,
                false
            ],
            }
        });

        let expected = vec![
            TypeDiff::new(
                "a_string_b_int".to_owned(),
                "string".to_owned(),
                "number".to_owned(),
            ),
            TypeDiff::new(
                "nested.a_bool_b_string".to_owned(),
                "bool".to_owned(),
                "string".to_owned(),
            ),
            TypeDiff::new(
                "array_3_a_string_b_int[3]".to_owned(),
                "string".to_owned(),
                "number".to_owned(),
            ),
            TypeDiff::new(
                "nested.array_3_a_int_b_bool[3]".to_owned(),
                "number".to_owned(),
                "bool".to_owned(),
            ),
        ];

        let working_context = create_test_working_context(true);
        let mut type_checker = CheckingData::new(
            "",
            &a.as_object().unwrap(),
            &b.as_object().unwrap(),
            &working_context,
        );

        // act
        type_checker.check();

        // assert
        assert_array(&expected, &type_checker.diffs());
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
