//! Read tfstate and return outputs.

use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;

/// Struct of output in tfstate
pub struct TfstateOutput {
	name: String,
	value: serde_json::Value,
}

/// Get outputs in tfstate.
///
/// Setting `show_output` to `true` displays a list of outputs on stdout.
///
/// # Output example
///
/// ```text
/// Number of outputs: 3.
/// --- 1 ---
/// name : aws_dynamodb_table_attribute
/// value: Array [Object {"name": String("channel_url"), "type": String("S")}]
/// --- 2 ---
/// name : aws_dynamodb_table_deletion_protection_enabled
/// value: Bool(false)
/// --- 3 ---
/// name : aws_dynamodb_table_read_capacity
/// value: Number(0)
/// ```
pub fn get_outputs(
	show_output: bool,
	file_path: &str,
) -> Result<Vec<TfstateOutput>, Box<dyn Error>> {
	let tfstate = File::open(file_path)?;
	let mut buf_reader = BufReader::new(tfstate);
	let mut contents = String::new();
	buf_reader.read_to_string(&mut contents)?;

	let contents_json: serde_json::Value = serde_json::from_str(&contents)?;
	let tfstate_outputs: Vec<TfstateOutput> = contents_json["outputs"]
		.as_object()
		.unwrap()
		.into_iter()
		.map(|val| TfstateOutput {
			name: val.0.to_string(),
			value: val.1["value"].clone(),
		})
		.collect();

	if show_output {
		// Stdout
		println!("Number of outputs: {}.", tfstate_outputs.len());
		tfstate_outputs.iter().enumerate().for_each(|(idx, val)| {
			println!(
				"--- {} ---\nname : {}\nvalue: {:?}",
				idx + 1,
				val.name,
				val.value
			);
		});
	}

	Ok(tfstate_outputs)
}
