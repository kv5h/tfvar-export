//! Construct value for exporting.

use std::collections::HashMap;

use crate::{get_outputs, read_export_list};

#[derive(Debug, PartialEq)]
pub struct ExportValue {
    variable_name: String,
    value: serde_json::Value,
}

impl ExportValue {
    pub fn new(variable_name: String, value: serde_json::Value) -> Self {
        Self {
            variable_name,
            value,
        }
    }

    pub fn get_variable_name(&self) -> &str {
        &self.variable_name
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
    // HashMap of `source output name : dest var name`
    let var_and_output_name = read_export_list(file_path_export_list)?.unwrap();
    // HashMap of `output name : output value`
    let output_value: HashMap<String, serde_json::Value> = get_outputs(file_path_output)?
        .iter()
        .map(|val| (val.get_name().to_owned(), val.get_value().to_owned()))
        .collect();

    let result = var_and_output_name
        .iter()
        .map(|(output_name, var_name)| ExportValue {
            variable_name: var_name.to_owned(),
            value: output_value.get(output_name).unwrap().to_owned(),
        })
        .collect();

    Ok(result)
}

#[cfg(test)]
mod tests {
    use std::collections::vec_deque;

    use serde_json::json;

    use super::*;

    #[test]
    fn test_construct_export_value() {
        let file_path_export_list = "files/test/export_list_construct_export_value.txt";
        let file_path_output = "files/test/outputs.json";

        let result = construct_export_value(file_path_export_list, file_path_output).unwrap();

        assert!(result.contains(&ExportValue {
            variable_name: String::from("number_0_out"),
            value: json!(0),
        }));
        assert!(result.contains(&ExportValue {
            variable_name: String::from("string_out"),
            value: json!("aaa"),
        }));
        assert!(result.contains(&ExportValue {
            variable_name: String::from("set_of_object_out"),
            value: json!([{"name":"aaa","type":"bbb"}]),
        }));
        assert!(result.len() == 3);
    }
}
