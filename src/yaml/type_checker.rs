/// Collects differences between the types of 2 data sets.
/// Stores `TypeDiff` values
///
/// 1. We iterate through object `a` and if a field is present in `b` as well, only then do we take action
///     1. We construct a new key. If we have a key in our checker object, than we add the currently checked fields key to it after a '.'. That's how we handle the keys of nested objects.
///     2. If `a` and `b` are both objects we recursively start the process over for the nested objects.
///     3. If both fields are arrays and the user has specified, that arrays should be in the same order, we iterate through the arrays and recursively repeat the checking for each item. If the user hasn't specified the option, this part is pointless.
///     4. If the types of the fields don't match, we add the difference to our `diffs` vector.
use serde_yaml::Value;

use crate::yaml::{
    diff_types::{Checker, CheckingData, DiffCollection, TypeDiff, ValueType},
    format_key,
};

impl<'a> Checker<TypeDiff> for CheckingData<'a, TypeDiff> {
    fn check(&mut self) {
        for (a_key, a_value) in self.a.into_iter() {
            if let Some(b_value) = self.b.get(a_key) {
                self.find_type_diffs_in_values(
                    &format_key(self.key, a_key.as_str().unwrap()),
                    a_value,
                    b_value,
                );
            }
        }
    }

    fn check_and_get(&mut self) -> &DiffCollection<TypeDiff> {
        self.check();
        &self.diffs
    }

    fn diffs(&self) -> &Vec<TypeDiff> {
        self.diffs.diffs()
    }
}

impl<'a> CheckingData<'a, TypeDiff> {
    fn find_type_diffs_in_values(&mut self, key_in: &str, a: &Value, b: &Value) {
        if a.is_mapping() && b.is_mapping() {
            self.find_type_diffs_in_objects(key_in, a, b);
        }

        if self.working_context.config.array_same_order
            && a.is_sequence()
            && b.is_sequence()
            && a.as_sequence().unwrap().len() == b.as_sequence().unwrap().len()
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
            a.as_mapping().unwrap(),
            b.as_mapping().unwrap(),
            self.working_context,
        );

        type_checker.check();
        self.diffs.concatenate(&mut type_checker.diffs);
    }

    fn find_type_diffs_in_arrays(&mut self, key_in: &str, a: &Value, b: &Value) {
        a.as_sequence()
            .unwrap()
            .iter()
            .enumerate()
            .for_each(|(i, a_item)| {
                self.find_type_diffs_in_values(
                    &format!("{}[{}]", key_in, i),
                    a_item,
                    &b.as_sequence().unwrap()[i],
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
        Value::Sequence(_) => ValueType::Array,
        Value::Mapping(_) => ValueType::Object,
        // TODO: may need a different type
        Value::Tagged(_) => ValueType::Number,
    }
}

#[cfg(test)]
mod tests {
    use serde_yaml::from_str;

    use crate::yaml::diff_types::{Checker, Config, TypeDiff, WorkingContext, WorkingFile};

    use super::CheckingData;

    const FILE_NAME_A: &str = "a.json";
    const FILE_NAME_B: &str = "b.json";

    #[test]
    fn test_find_type_diffs_no_array_same_order() {
        // arrange
        let a = from_str(
            r"
            'a_string_b_int': 'a_string_b_int'
            'both_string': 'both_string'
            'array_3_a_string_b_int':
                - 'string'
                - 'string2'
                - 'string3'
                - 'string4'
                - 8
                - true
            'nested':
                'a_bool_b_string': true
                'both_number': 4
                'array_3_a_int_b_bool':
                    - 'string'
                    - 'string2'
                    - 'string3'
                    - 6
                    - 8
                    - true
        ",
        )
        .unwrap();
        let b = from_str(
            r"
            'a_string_b_int': 2
            'both_string': 'both_string'
            'array_3_a_string_b_int':
                - 'other_string'
                - 'other_string2'
                - 'other_string3'
                - 5,
                - 1,
                - false
            'nested':
                'a_bool_b_string': 'a_bool_b_string'
                'both_number': 1
                'array_3_a_int_b_bool':
                    - 'other_string'
                    - 'other_string2'
                    - 'other_string3'
                    - false,
                    - 2,
                    - false
        ",
        )
        .unwrap();

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
        let mut type_checker = CheckingData::new("", &a, &b, &working_context);

        // act
        type_checker.check();

        // assert
        assert_array(&expected, type_checker.diffs());
    }

    #[test]
    fn test_find_type_diffs_array_same_order() {
        // arrange
        let a = from_str(
            r"
            'a_string_b_int': 'a_string_b_int'
            'both_string': 'both_string'
            'array_3_a_string_b_int':
                - 'string'
                - 'string2'
                - 'string3'
                - 'string4'
                - 8,
                - true
            'nested':
                'a_bool_b_string': true
                'both_number': 4
                'array_3_a_int_b_bool':
                    - 'string'
                    - 'string2'
                    - 'string3'
                    - 6
                    - 8
                    - true
        ",
        )
        .unwrap();
        let b = from_str(
            r"
            'a_string_b_int': 2
            'both_string': 'both_string'
            'array_3_a_string_b_int':
                - 'other_string'
                - 'other_string2'
                - 'other_string3'
                - 5,
                - 1,
                - false
            'nested':
                'a_bool_b_string': 'a_bool_b_string'
                'both_number': 1
                'array_3_a_int_b_bool':
                    - 'other_string'
                    - 'other_string2'
                    - 'other_string3'
                    - false,
                    - 2,
                    - false
        ",
        )
        .unwrap();

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
        let mut type_checker = CheckingData::new("", &a, &b, &working_context);

        // act
        type_checker.check();

        // assert
        assert_array(&expected, type_checker.diffs());
    }

    // Test utils

    fn create_test_working_context(array_same_order: bool) -> WorkingContext {
        let config = Config::new(array_same_order);
        let working_file_a = WorkingFile::new(FILE_NAME_A.to_owned());
        let working_file_b = WorkingFile::new(FILE_NAME_B.to_owned());
        WorkingContext::new(working_file_a, working_file_b, config)
    }

    println!("expected: {:?}", expected);
    println!("result: {:?}", result);

    fn assert_array<T: PartialEq>(expected: &Vec<T>, result: &Vec<T>) {
        assert_eq!(expected.len(), result.len());
        assert!(expected.into_iter().all(|item| result.contains(&item)));
    }
}
