//! Read output values file and return outputs.

use std::io::{prelude::*, BufReader};

#[derive(Debug, PartialEq, Eq)]
/// Struct of output value
pub struct OutputValue {
    name: String,
    value: serde_json::Value,
}

impl OutputValue {
    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_value(&self) -> &serde_json::Value {
        &self.value
    }
}

/// Read outputs from a file generated with `terraform output --json`.
///
/// ## Remark
///
/// - `sensitive` outputs are ignored for security reason.
pub fn get_outputs(file_path: &str) -> Result<Vec<OutputValue>, Box<dyn std::error::Error>> {
    let output_values_file = std::fs::File::open(file_path)?;
    let mut buf_reader = BufReader::new(output_values_file);
    let mut contents = String::new();
    buf_reader.read_to_string(&mut contents)?;

    let contents_json: serde_json::Value = serde_json::from_str(&contents)?;
    let output_values: Vec<OutputValue> = contents_json
        .as_object()
        .unwrap()
        .into_iter()
        .filter(|val| val.1["sensitive"] == false) // Opt out `sensitive` elements.
        .map(|val| OutputValue {
            name: val.0.to_string(),
            value: val.1["value"].clone(),
        })
        .collect();

    Ok(output_values)
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn test_get_outputs() {
        let test_file = "files/test/outputs.json";
        let res = get_outputs(&test_file).unwrap();
        assert_eq!(res, vec![
            OutputValue {
                name: String::from("bool"),
                value: json!(false),
            },
            OutputValue {
                name: String::from("list_of_object"),
                value: json!({"a":"aaa","b":"bbb","c":null}),
            },
            OutputValue {
                name: String::from("map_of_string"),
                value: json!({"a":"aaa","b":"bbb","c":"ccc"}),
            },
            OutputValue {
                name: String::from("number_0"),
                value: json!(0),
            },
            OutputValue {
                name: String::from("number_float"),
                value: json!(1.2345),
            },
            OutputValue {
                name: String::from("number_negative"),
                value: json!(-1.2345),
            },
            OutputValue {
                name: String::from("set_of_object"),
                value: json!([{"name":"aaa","type":"bbb"}]),
            },
            OutputValue {
                name: String::from("string"),
                value: json!("aaa"),
            },
            OutputValue {
                name: String::from("string_with_quote"),
                value: json!("aaa\"bbb"),
            },
            OutputValue {
                name: String::from("tuple"),
                value: json!(["aaa", "bbb"]),
            },
        ])
    }
}
