//! Construct value for exporting.

use std::collections::HashMap;

use crate::utils::{get_outputs::get_outputs, read_export_list::read_export_list};

#[derive(Debug, PartialEq)]
pub struct ExportValue {
    variable_name: String,
    variable_description: Option<String>,
    value: serde_json::Value,
}

impl ExportValue {
    pub fn get_variable_name(&self) -> &str {
        &self.variable_name
    }

    pub fn get_variable_description(&self) -> &Option<String> {
        &self.variable_description
    }

    pub fn get_value(&self) -> &serde_json::Value {
        &self.value
    }
}

/// Construct a vector of values for exporting
/// by mapping the output value and the variable name.
pub fn construct_export_value(
    file_path_export_list: &str,
    file_path_output: &str,
) -> Result<Vec<ExportValue>, Box<dyn std::error::Error>> {
    let export_list = read_export_list(file_path_export_list)?.unwrap();
    let output_value: HashMap<String, serde_json::Value> = get_outputs(file_path_output)?
        .iter()
        .map(|val| (val.get_name().to_owned(), val.get_value().to_owned()))
        .collect();

    // Merge values
    let result = export_list
        .iter()
        .map(|(output_name, (var_name, opt_description))| ExportValue {
            variable_name: var_name.to_owned(),
            variable_description: opt_description.to_owned(),
            value: output_value.get(output_name).unwrap().to_owned(),
        })
        .collect();

    Ok(result)
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn test_construct_export_value() {
        let file_path_export_list = "files/test/export_list_construct_export_value.txt";
        let file_path_output = "files/test/outputs.json";

        let result = construct_export_value(file_path_export_list, file_path_output).unwrap();

        assert!(result.contains(&ExportValue {
            variable_name: String::from("number_0_out"),
            variable_description: None,
            value: json!(0),
        }));
        assert!(result.contains(&ExportValue {
            variable_name: String::from("string_out"),
            variable_description: Some(String::from("string_description")),
            value: json!("aaa"),
        }));
        assert!(result.contains(&ExportValue {
            variable_name: String::from("set_of_object_out"),
            variable_description: Some(String::from("set_of_object_description")),
            value: json!([{"name":"aaa","type":"bbb"}]),
        }));
        assert!(result.len() == 3);
    }
}
