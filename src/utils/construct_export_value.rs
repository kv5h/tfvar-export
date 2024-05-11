//! Construct value for exporting.

use std::collections::HashMap;

use crate::{get_outputs, read_export_list};

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
    file_path: &str,
) -> Result<Vec<ExportValue>, Box<dyn std::error::Error>> {
    // HashMap of `source output name : dest val name`
    let var_and_output_name = read_export_list(file_path)?.unwrap();
    // HashMap of `output name : output value`
    let output_value: HashMap<String, serde_json::Value> = get_outputs(file_path)?
        .iter()
        .map(|val| (val.get_name().to_owned(), val.get_value().to_owned()))
        .collect();

    let result = var_and_output_name
        .iter()
        .map(|(output_name, var_name)| ExportValue {
            variable_name: var_name.to_owned(),
            value: output_value.get(output_name).unwrap().to_owned(),
        }).collect();

    Ok(result)
}
