use criterion::{criterion_group, criterion_main, Criterion};
use libdtf::{
    array_checker::ArrayChecker,
    diff_types::{Checker, Config, WorkingContext, WorkingFile},
    key_checker::KeyChecker,
    type_checker::TypeChecker,
    value_checker::ValueChecker,
};
use serde_json::json;

const FILE_NAME_A: &str = "a.json";
const FILE_NAME_B: &str = "b.json";

fn benchmark_find_key_diffs(c: &mut Criterion) {
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

    let working_context = create_test_working_context(false);

    // act
    c.bench_function("Find Key Diffs", |bencher| {
        bencher.iter(|| {
            let mut key_checker = KeyChecker::new(
                "",
                &a.as_object().unwrap(),
                &b.as_object().unwrap(),
                &working_context,
            );
            key_checker.check();
        })
    });
}

fn benchmark_find_type_diffs_no_array_same_order(c: &mut Criterion) {
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

    let working_context = create_test_working_context(false);

    // act
    c.bench_function("Find Type Diffs No Array Same Order", |bencher| {
        bencher.iter(|| {
            let mut type_checker = TypeChecker::new(
                "",
                &a.as_object().unwrap(),
                &b.as_object().unwrap(),
                &working_context,
            );
            type_checker.check();
        })
    });
}

fn benchmark_find_type_diffs_array_same_order(c: &mut Criterion) {
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

    let working_context = create_test_working_context(true);

    // act
    c.bench_function("Find Type Diffs Array Same Order", |bencher| {
        bencher.iter(|| {
            let mut type_checker = TypeChecker::new(
                "",
                &a.as_object().unwrap(),
                &b.as_object().unwrap(),
                &working_context,
            );
            type_checker.check();
        })
    });
}

fn benchmark_find_value_diffs_no_array_same_order(c: &mut Criterion) {
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

    let working_context = create_test_working_context(false);

    // act
    c.bench_function("Find Value Diffs No Array Same Order", |bencher| {
        bencher.iter(|| {
            let mut value_checker = ValueChecker::new(
                "",
                &a.as_object().unwrap(),
                &b.as_object().unwrap(),
                &working_context,
            );
            value_checker.check();
        })
    });
}

fn benchmark_find_value_diffs_array_same_order(c: &mut Criterion) {
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

    let working_context = create_test_working_context(true);

    // act
    c.bench_function("Find Value Diffs Array Same Order", |bencher| {
        bencher.iter(|| {
            let mut value_checker = ValueChecker::new(
                "",
                &a.as_object().unwrap(),
                &b.as_object().unwrap(),
                &working_context,
            );
            value_checker.check();
        })
    });
}

fn benchmark_find_array_diffs(c: &mut Criterion) {
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

    let working_context = create_test_working_context(false);

    // act
    c.bench_function("Find Array Diffs", |bencher| {
        bencher.iter(|| {
            let mut array_checker = ArrayChecker::new(
                "",
                &a.as_object().unwrap(),
                &b.as_object().unwrap(),
                &working_context,
            );
            array_checker.check();
        })
    });
}

// Benchmark utils

fn create_test_working_context(array_same_order: bool) -> WorkingContext {
    let config = Config::new(array_same_order);
    let working_file_a = WorkingFile::new(FILE_NAME_A.to_owned());
    let working_file_b = WorkingFile::new(FILE_NAME_B.to_owned());
    WorkingContext::new(working_file_a, working_file_b, config)
}

criterion_group!(
    benches,
    benchmark_find_key_diffs,
    benchmark_find_type_diffs_no_array_same_order,
    benchmark_find_type_diffs_array_same_order,
    benchmark_find_value_diffs_no_array_same_order,
    benchmark_find_value_diffs_array_same_order,
    benchmark_find_array_diffs
);
criterion_main!(benches);
