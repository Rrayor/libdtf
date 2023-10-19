use serde::{Deserialize, Serialize};
use serde_yaml::Mapping;
use std::fmt;

pub trait Stringable {
    fn to_string(&self) -> String;
}

impl Stringable for serde_yaml::Value {
    fn to_string(&self) -> String {
        match &self {
            serde_yaml::Value::Null => "null".to_owned(),
            serde_yaml::Value::Bool(value) => value.to_string(),
            serde_yaml::Value::Number(value) => value.to_string(),
            serde_yaml::Value::String(value) => value.to_string(),
            // TODO: proper to_string() for these
            serde_yaml::Value::Sequence(_) => "array".to_owned(),
            serde_yaml::Value::Mapping(_) => "mapping".to_owned(),
            serde_yaml::Value::Tagged(_) => "tagged".to_owned(),
        }
    }
}

/// Holds the data required to run a difference check
pub struct CheckingData<'a, T: Diff> {
    /// Holds the collected differences
    pub diffs: DiffCollection<T>,
    /// Holds the key of the field currently checked - empty if it's the outermost object
    pub key: &'a str,
    /// One of the 2 objects that should be checked
    pub a: &'a Mapping,
    /// One of the 2 objects that should be checked
    pub b: &'a Mapping,
    /// Holds relevant data for the current run, such as file names, and user configs
    pub working_context: &'a WorkingContext,
}

impl<'a, T: Diff> CheckingData<'a, T> {
    pub fn new(
        key: &'a str,
        a: &'a Mapping,
        b: &'a Mapping,
        working_context: &'a WorkingContext,
    ) -> CheckingData<'a, T> {
        let diff_collection: DiffCollection<T> = DiffCollection::new();
        CheckingData {
            diffs: diff_collection,
            key,
            a,
            b,
            working_context,
        }
    }
}

/// Defines a type for all the checker modules to use
pub trait Checker<T: Diff> {
    /// Does all the difference checks and store the results in the diffs vector
    fn check(&mut self);

    /// Does the difference checks, stores it in the object, but also returns it
    fn check_and_get(&mut self) -> &DiffCollection<T>;

    /// Returns the stored diffs
    fn diffs(&self) -> &Vec<T>;
}

/// Tag trait for the difference types
pub trait Diff {}

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

/// Holds diffs
pub struct DiffCollection<T: Diff> {
    diffs: Vec<T>,
}

impl<T: Diff> Default for DiffCollection<T> {
    fn default() -> Self {
        DiffCollection::new()
    }
}

impl<T: Diff> DiffCollection<T> {
    pub fn new() -> DiffCollection<T> {
        DiffCollection { diffs: vec![] }
    }

    pub fn diffs(&self) -> &Vec<T> {
        &self.diffs
    }

    pub fn concatenate(&mut self, diffs: &mut DiffCollection<T>) {
        self.diffs.append(&mut diffs.diffs);
    }

    pub fn extend<I: IntoIterator<Item = T>>(&mut self, diffs: I) {
        self.diffs.extend(diffs);
    }

    pub fn append(&mut self, diffs: &mut Vec<T>) {
        self.diffs.append(diffs);
    }

    pub fn push(&mut self, diff: T) {
        self.diffs.push(diff);
    }
}

/// Arrays have only one kind of difference if unordered. Either an array has an item present in the other or not.
/// We can describe this relation with 4 values:
/// 1. AHas/BMisses
/// 2. AMisses/BHas
#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Debug, Clone, Copy)]
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

impl Diff for KeyDiff {}

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

impl Diff for TypeDiff {}

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

impl Diff for ValueDiff {}

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

impl Diff for ArrayDiff {}

pub type ComparisionResult = (Vec<KeyDiff>, Vec<TypeDiff>, Vec<ValueDiff>, Vec<ArrayDiff>);
