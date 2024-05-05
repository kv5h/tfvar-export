//! Read tfstate and return outputs.

use std::io::prelude::*;
use std::io::BufReader;

#[derive(Debug, PartialEq, Eq)]
/// Struct of output in tfstate
pub struct TfstateOutput {
    name: String,
    value: serde_json::Value,
}

/// Read outputs from a file generated with `terraform output --json`.
///
/// ## Remark
///
/// - `sensitive` outputs are ignored for security reason.
/// - Setting `show_output` to `true` displays a list of outputs on stdout.
///
/// ## Output example
///
/// ```text
/// Number of outputs: 10.
/// --- 1 ---
/// name : bool
/// value: false
/// --- 2 ---
/// name : list_of_object
/// value: {"a":"aaa","b":"bbb","c":null}
/// --- 3 ---
/// name : map_of_string
/// value: {"a":"aaa","b":"bbb","c":"ccc"}
/// --- 4 ---
/// name : number_0
/// value: 0
/// --- 5 ---
/// name : number_float
/// value: 1.2345
/// --- 6 ---
/// name : number_negative
/// value: -1.2345
/// --- 7 ---
/// name : set_of_object
/// value: [{"name":"aaa","type":"bbb"}]
/// --- 8 ---
/// name : string
/// value: "aaa"
/// --- 9 ---
/// name : string_with_quote
/// value: "aaa\"bbb"
/// --- 10 ---
/// name : tuple
/// value: ["aaa","bbb"]
/// ```
pub fn get_outputs(
    show_output: bool,
    file_path: &str,
) -> Result<Vec<TfstateOutput>, Box<dyn std::error::Error>> {
    let tfstate = std::fs::File::open(file_path)?;
    let mut buf_reader = BufReader::new(tfstate);
    let mut contents = String::new();
    buf_reader.read_to_string(&mut contents)?;

    let contents_json: serde_json::Value = serde_json::from_str(&contents)?;
    let tfstate_outputs: Vec<TfstateOutput> = contents_json
        .as_object()
        .unwrap()
        .into_iter()
        .filter(|val| val.1["sensitive"] == false) // Opt out `sensitive` elements.
        .map(|val| TfstateOutput {
            name: val.0.to_string(),
            value: val.1["value"].clone(),
        })
        .collect();

    if show_output {
        println!("Number of outputs: {}.", tfstate_outputs.len());
        tfstate_outputs.iter().enumerate().for_each(|(idx, val)| {
            println!(
                "--- {} ---\nname : {}\nvalue: {}",
                idx + 1,
                val.name,
                val.value,
            );
        });
    }

    Ok(tfstate_outputs)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_get_outputs() {
        let test_file = "files/test/outputs.json";
        let res = get_outputs(false, &test_file).unwrap();
        assert_eq!(res, vec![
            TfstateOutput {
                name: String::from("bool"),
                value: json!(false),
            },
            TfstateOutput {
                name: String::from("list_of_object"),
                value: json!({"a":"aaa","b":"bbb","c":null}),
            },
            TfstateOutput {
                name: String::from("map_of_string"),
                value: json!({"a":"aaa","b":"bbb","c":"ccc"}),
            },
            TfstateOutput {
                name: String::from("number_0"),
                value: json!(0),
            },
            TfstateOutput {
                name: String::from("number_float"),
                value: json!(1.2345),
            },
            TfstateOutput {
                name: String::from("number_negative"),
                value: json!(-1.2345),
            },
            TfstateOutput {
                name: String::from("set_of_object"),
                value: json!([{"name":"aaa","type":"bbb"}]),
            },
            TfstateOutput {
                name: String::from("string"),
                value: json!("aaa"),
            },
            TfstateOutput {
                name: String::from("string_with_quote"),
                value: json!("aaa\"bbb"),
            },
            TfstateOutput {
                name: String::from("tuple"),
                value: json!(["aaa", "bbb"]),
            },
        ])
    }
}
