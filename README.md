<h1 align="center">Libdtf</h1>

[![Rust](https://github.com/Rrayor/libdtf/actions/workflows/rust.yml/badge.svg)](https://github.com/Rrayor/libdtf/actions/workflows/rust.yml)

A utility lib for my datadiffer projects. It finds the differences in data between 2 datasets in the supported formats.

- [Supported formats](#supported-formats)
- [Types of differences](#types-of-differences)
  - [Key difference](#key-difference)
  - [Type difference](#type-difference)
  - [Value difference](#value-difference)
  - [Array difference](#array-difference)
- [Configuration options](#configuration-options)
- [Usage](#usage)
- [Architecture](#architecture)
- [For Contributors](#for-contributors)
  - [Thank you for taking interest](#thank-you-for-taking-interest)
  - [The goal](#the-goal)
  - [How to contribute](#how-to-contribute)
  - [Technical guidelines](#technical-guidelines)
  - [Quality guidelines](#quality-guidelines)
  - [Be reasonable](#be-reasonable)
  - [Share your projects](#share-your-projects)
  - [Contact](#contact)


# Supported formats

Currently supported formats include:
* JSON (`.json`)
* YAML (`.yaml`, `.yml`)

**Important:**  Cross functionality is not yet(?) supported. You can only check JSON against JSON and YAML against YAML!

# Types of differences

The following are the types of differences the lib is built to spot.

## Key difference

Tells the user if there are keys that are present in one dataset, which are missing in the other.

## Type difference

Tells the user if the type of data belonging to a specific field in one dataset differs from the type of data belonging to the same field in the other.

## Value difference

Tells the user if the value of a field in one dataset differs from the value of the same field in the other.

## Array difference

Tells the user if an array like field has items in one dataset that are missing from the same array like field in the other.

# Configuration options

`array_same_order`: If set to true, it will check array like fields against their counterparts by index and return [value differences](#value-difference) instead of [array ones](#array-difference).

# Usage

Either you are dealing with a JSON file or a YAML one, there are some common types you should use from the `core` module:

```rust
core::diff_types::{ArrayDiff, Checker, KeyDiff, TypeDiff, ValueDiff}
```

`ArrayDiff`, `KeyDiff`, `TypeDiff` and `ValueDiff` represent the types of differences the lib can find. Each of these implement the `Diff` trait which doesn't add any functionality just helps with using generics elsewhere in the code.

`Checker` is a trait that let's you use the different modules' implementations of `CheckingData<ArrayDiff>`, `CheckingData<KeyDiff>`, `CheckingData<TypeDiff>` and `CheckingData<ValueDiff>`.

Each format specific module has their own `CheckingData` type which you should use:
For JSON

```rust
json::diff_types::CheckingData
```

For YAML

```rust
yaml::diff_types::CheckingData
```

In both cases `CheckingData` is a generic that can take one of the four `Diff` types as their type argument.

You can then acquire the differences like so:

```rust
checking_data: CheckingData<KeyDiff> = CheckingData::new("", data1, data2, &lib_working_context);
checking_data.check();
let diffs = checking_data.diffs()
```

Where
* For JSON:  `data1` and `data2` are of type `serde::json::Map<String, serde::json::Value>` and lib_working_context is of type `libdtf::core::diff_types::WorkingContext`.
* For YAML:  `data1` and `data2` are of type `serde::yaml::Mapping` and lib_working_context is of type `libdtf::core::diff_types::WorkingContext`.

`WorkingContext` acts as a "meta-information" storage for the lib. It contains information used across different functionalities, like information on the files that are checked and configuration options.



# Architecture

Currently each format sits in their on module. These each have their own files for types and difference checking specific to them. Common functionality resides in the `core` module. This structure originates from the type differences between `serde::json` and `serde::yaml` and the few differences between JSON and YAML.

I deemed code duplication the lesser evil against complicated generics that might not even work, but I haven't given up on reducing it entirely. For now, if you'd like to add support for a new format, feel free to follow this pattern, though if you can think of a way to make the code less repetitive, your changes are welcome!

# For Contributors

## Thank you for taking interest
As this is my first open-source and my first Rust project, I welcome every bit of feedback and code contribution. I am trying my best to offer some information and guidelines, to ease all of our work., but I'm sure things will change as I or rather we will learn from the experience 😄 

## The goal

The initial goal was to support my project [DataDiffer Terminal](https://github.com/Rrayor/datadiff-terminal) in a separate library, so in the future I may derive different projects from it.

Libdtf is supposed to hold the core functionalities to make differences in datasets available to dependent applications in an easy-to-consume format.

## How to contribute
There are a few ways you can contribute:

* Create your own tool built upon Libdtf! Have fun 😉 
* If you have any issues or ideas, open an issue for it
* If you know Rust, take a look at the open issues, or if you have your own improvement ideas, open a Pull Request.
* Don't be afraid to flex your proficiency in Rust or software-development in general, but please provide the rest of us some good descriptions of what you have done, or point us in the direction of some of the sources, you've learnt from. We can all learn from each other.
* If you find the lib helpful, maybe share it with others, who are in need of something like this 😊

## Technical guidelines
Please try to follow some technicalities. If there is an issue, the branch is made for, it should contain its identifier and the branch name should hint at what the code should solve.

* Name your branches following these patterns:
  * 'feature/`{identifier-of-issue}`*': For changes that add new functionality to the software
  * 'bugfix/`{identifier-of-issue}`*': For changes that fix or improve existing functionality
* Commit messages should start with `#{identifier-of-issue}` if they are part of solving a Github issue.

## Quality guidelines
Here are some pointers for software quality:
* We should have meaningful unit and integration tests, where possible.
* If you think, something could be tested, which is not, please don't hesitate to implement the tests yourself.
* Always test your code.

And some points regarding code quality:
* We should follow the Rust conventions but not blindly.
* Clean code also provides some good directions, but that does not mean, it's always right.
* Make your code simple, and easy on the eyes and mind.
* Always write Rustdoc for your structs and public methods/functions.
* If you can't simplify your code enough, write down your reasoning in comments.
* Don't over-comment.  Good structure and naming is better than good comments every time!
* If you see an opportunity for improvement, take it, don't leave it alone!

## Be reasonable

Some social guidelines, I will try my best to uphold:
* Judge the code, but not the people! If you think, someone has to learn more, help them to it!
* Don't make assumptions, ask questions! Instead of "Don't be lazy!" Maybe try this: "Why did you choose this approach?"
* No kind of personal offenses are welcome of any nature!

Thank you for helping in maintaining the code and keeping the conversations friendly! 🙇

## Share your projects

If you have your own project built on Libdtf, share it with the community! I'd be happy to show it off here too! 😊

## Contact
If you have any questions or concerns, please reach out via email at [rrayor.dev@gmail.com](mailto:rrayor.dev@gmail.com) or right here in the discussions!