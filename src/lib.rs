pub mod core;
pub mod json;
pub mod yaml;

#[cfg(test)]
mod tests {
    use crate::json::read_json_file;
    use crate::yaml::read_yaml_file;

    #[test]
    fn test_read_json_file() {
        let result = read_json_file("test_data.json");
        println!("{:?}", result.unwrap());
        assert!(true);
    }

    #[test]
    fn test_read_yaml_file() {
        let result = read_yaml_file("test_data.yaml");
        println!("{:?}", result.unwrap());
        assert!(true);
    }
}
