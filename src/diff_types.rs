use serde::{Deserialize, Serialize};
use std::fmt;

/// Used for tracking the types of fields in the read-in data
/// It has a Display implementation for ease-of-use in dependent applications
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ValueType {
    Null,
    Boolean,
    Number,
    String,
    Array,
    Object,
}

impl fmt::Display for ValueType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value_type_str = match self {
            ValueType::Null => "null",
            ValueType::Boolean => "bool",
            ValueType::Number => "number",
            ValueType::String => "string",
            ValueType::Array => "array",
            ValueType::Object => "object",
        };
        write!(f, "{}", value_type_str)
    }
}

/// Arrays have only one kind of difference if unordered. Either an array has an item present in the other or not.
/// We can describe this relation with 4 values:
/// 1. AHas/BMisses
/// 2. AMisses/BHas
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone, Copy)]
pub enum ArrayDiffDesc {
    AHas,
    AMisses,
    BHas,
    BMisses,
}

/// Contains configuration options
#[derive(Serialize, Deserialize, Clone, Copy)]
pub struct Config {
    /// Used for switching between one-by-one value comparison for arrays or has/misses kind of comparison
    pub array_same_order: bool,
}

impl Config {
    pub fn new(array_same_order: bool) -> Config {
        Config { array_same_order }
    }
}

/// Contains data about the file we're currently working with
#[derive(Serialize, Deserialize, Clone)]
pub struct WorkingFile {
    pub name: String,
}

impl WorkingFile {
    pub fn new(name: String) -> WorkingFile {
        WorkingFile { name }
    }
}

/// Contains data of the current run
#[derive(Serialize, Deserialize, Clone)]
pub struct WorkingContext {
    pub file_a: WorkingFile,
    pub file_b: WorkingFile,
    pub config: Config,
}

impl WorkingContext {
    pub fn new(file_a: WorkingFile, file_b: WorkingFile, config: Config) -> WorkingContext {
        WorkingContext {
            file_a,
            file_b,
            config,
        }
    }
}

/// Stores differences in keys. Either a data-structure has a key present in the other or not.
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct KeyDiff {
    pub key: String,
    pub has: String,
    pub misses: String,
}

impl KeyDiff {
    pub fn new(key: String, has: String, misses: String) -> KeyDiff {
        KeyDiff { key, has, misses }
    }
}

/// Stores differences in types. Used when a field with the same key has different types in the compared data.
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct TypeDiff {
    pub key: String,
    pub type1: String,
    pub type2: String,
}

impl TypeDiff {
    pub fn new(key: String, type1: String, type2: String) -> TypeDiff {
        TypeDiff { key, type1, type2 }
    }
}

/// Stores differences in values. Used when a field with the same key has different values in the compared data.
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct ValueDiff {
    pub key: String,
    pub value1: String,
    pub value2: String,
}

impl ValueDiff {
    pub fn new(key: String, value1: String, value2: String) -> ValueDiff {
        ValueDiff {
            key,
            value1,
            value2,
        }
    }
}

/// Stores differences in array contents. Used when two arrays with the same keys have different content in the compared data.
/// Only used when the user hasn't specified in the configs that the arrays should be in the same order.
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct ArrayDiff {
    pub key: String,
    pub descriptor: ArrayDiffDesc,
    pub value: String,
}

impl ArrayDiff {
    pub fn new(key: String, descriptor: ArrayDiffDesc, value: String) -> ArrayDiff {
        ArrayDiff {
            key,
            descriptor,
            value,
        }
    }
}

pub type ComparisionResult = (Vec<KeyDiff>, Vec<TypeDiff>, Vec<ValueDiff>, Vec<ArrayDiff>);
