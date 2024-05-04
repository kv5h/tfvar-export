//! Update or create Terraform Cloud workspace variable.
//!
//! **API Reference:** https://developer.hashicorp.com/terraform/cloud-docs/api-docs/workspace-variables

use log::info;
use ratelimit::Ratelimiter;
use reqwest::Client;
use serde_json::json;
use std::collections::HashMap;
use std::error::Error;
use std::time::Duration;

/// Terraform variable status
#[derive(Debug, Eq, PartialEq)]
pub struct TerraformVariableStatus {
	already_exist: bool,
	variable_id: String,
}

/// Terraform variable property
#[derive(Debug)]
pub struct TerraformVariableProperty {
	variable_id: Option<String>,
	variable_name: String,
	value: String,
}

/// Terraform variable creation result
#[derive(Debug)]
pub struct TerraformVariableCreationResult {
	variable_id: String,
	variable_name: String,
}

/// Update Terraform Workspace variable(s).
///
/// **Remark:** To prevent [`Rate Limiting`](https://developer.hashicorp.com/terraform/cloud-docs/api-docs#rate-limiting), limit the rate 20 requests per second.
pub async fn update_variable(
	api_base_url: &str,
	token: &str,
	workspace_id: &str,
	terraform_variable_property: &Vec<TerraformVariableProperty>,
) -> Result<(), Box<dyn Error>> {
	// Limit the rate 20 requests per second.
	let ratelimiter = Ratelimiter::builder(20, Duration::from_secs(1))
		.build()
		.unwrap();

	let count = terraform_variable_property.len();
	for i in 0..count {
		if let Err(sleep) = ratelimiter.try_wait() {
			std::thread::sleep(sleep);
			continue;
		}

		let mut map = HashMap::new();
		let variable_id = terraform_variable_property[i].variable_id.as_ref().unwrap();
		let data = json!({
		  "type": "vars",
		"id": variable_id,
		  "attributes": {
			"key": terraform_variable_property[i].variable_name,
			"value": terraform_variable_property[i].value,
			"description": "",
			"category": "terraform",
			"hcl": true,
		  }});
		map.insert("data", data.to_string());

		let response = Client::new()
			.post(format!(
				"{}/workspaces/{}/vars/{}",
				api_base_url, workspace_id, variable_id
			))
			.header("Authorization", format!("Bearer {}", token))
			.header("Content-Type", "application/vnd.api+json")
			.json(&map)
			.send()
			.await?;

		assert!(
			response.status() == 200,
			"Response status is {}.",
			response.status()
		);
	}

	info!("Variables updated: {}.", count);
	println!("Variables updated: {}.", count); // TODO:

	Ok(())
}

/// Create Terraform Workspace variable(s).
///
/// **Remark:** To prevent [`Rate Limiting`](https://developer.hashicorp.com/terraform/cloud-docs/api-docs#rate-limiting), limit the rate 20 requests per second.
pub async fn create_variable(
	api_base_url: &str,
	token: &str,
	workspace_id: &str,
	terraform_variable_property: &Vec<TerraformVariableProperty>,
) -> Result<Vec<TerraformVariableCreationResult>, Box<dyn Error>> {
	// Limit the rate 20 requests per second.
	let ratelimiter = Ratelimiter::builder(20, Duration::from_secs(1))
		.build()
		.unwrap();

	let mut result = Vec::new();

	let count = terraform_variable_property.len();
	for i in 0..count {
		if let Err(sleep) = ratelimiter.try_wait() {
			std::thread::sleep(sleep);
			continue;
		}

		let mut map = HashMap::new();
		let data = json!({
		"type": "vars",
		"attributes": {
		  "key": terraform_variable_property[i].variable_name,
		  "value": terraform_variable_property[i].value,
		  "description": "",
		  "category": "terraform",
		  "hcl": true,
		}});
		map.insert("data", data.to_string());

		let response = Client::new()
			.post(format!("{}/workspaces/{}/vars", api_base_url, workspace_id))
			.header("Authorization", format!("Bearer {}", token))
			.header("Content-Type", "application/vnd.api+json")
			.json(&map)
			.send()
			.await?;

		assert!(
			response.status() == 201,
			"Response status is {}.",
			response.status()
		);

		let json_value: serde_json::Value = serde_json::from_str(&response.text().await.unwrap())?;
		result.push(TerraformVariableCreationResult {
			variable_id: json_value["data"]["id"].as_str().unwrap().to_string(),
			variable_name: json_value["data"]["attributes"]["key"]
				.as_str()
				.unwrap()
				.to_string(),
		});
	}

	info!("Variables created: {}.", count);
	println!("Variables created: {}.", count); // TODO:

	Ok(result)
}

pub async fn check_variable_status(
	api_base_url: &str,
	target_variable_ids: &Vec<String>,
	token: &str,
	workspace_id: &str,
) -> Result<Vec<TerraformVariableStatus>, Box<dyn Error>> {
	let response = Client::new()
		.get(format!("{}/workspaces/{}/vars", api_base_url, workspace_id))
		.header("Authorization", format!("Bearer {}", token))
		.header("Content-Type", "application/vnd.api+json")
		.send()
		.await?
		.text()
		.await?;

	let mut vars_already_exist = Vec::new();
	let response_jv: serde_json::Value = serde_json::from_str(&response)?;
	response_jv["data"]
		.as_array()
		.unwrap()
		.into_iter()
		.for_each(|val| {
			vars_already_exist.push(val["id"].as_str().unwrap().to_string());
		});

	let mut result = Vec::new();
	target_variable_ids
		.iter()
		.for_each(|val| match vars_already_exist.contains(val) {
			true => result.push(TerraformVariableStatus {
				already_exist: true,
				variable_id: val.to_string(),
			}),
			false => result.push(TerraformVariableStatus {
				already_exist: false,
				variable_id: val.to_string(),
			}),
		});

	info!("Variable status: {:#?}", result);

	Ok(result)
}

#[cfg(test)]
mod tests {
	use super::*;
	use rand::distributions::{Alphanumeric, DistString};
	use std::env;

	#[tokio::test]
	async fn test_check_variable_status() {
		let var_1 = Alphanumeric
			.sample_string(&mut rand::thread_rng(), 32)
			.to_lowercase();
		let var_2 = Alphanumeric
			.sample_string(&mut rand::thread_rng(), 32)
			.to_lowercase();
		let var_3 = Alphanumeric
			.sample_string(&mut rand::thread_rng(), 32)
			.to_lowercase();
		let res = check_variable_status(
			"https://app.terraform.io/api/v2",
			&vec![
				var_1.clone(),
				"var-Tppa4XRHcAt7qniZ".to_string(),
				var_2.clone(),
				var_3.clone(),
			],
			&env::var("TFVE_TOKEN").unwrap(),
			&env::var("TFVE_WORKSPACE_ID").unwrap(),
		)
		.await
		.unwrap();

		assert_eq!(res, vec![
			TerraformVariableStatus {
				already_exist: false,
				variable_id: var_1,
			},
			TerraformVariableStatus {
				already_exist: true,
				variable_id: "var-Tppa4XRHcAt7qniZ".to_string(),
			},
			TerraformVariableStatus {
				already_exist: false,
				variable_id: var_2,
			},
			TerraformVariableStatus {
				already_exist: false,
				variable_id: var_3,
			},
		])
	}

	#[tokio::test]
	async fn test_create_variable() {
		let var_1 = Alphanumeric
			.sample_string(&mut rand::thread_rng(), 32)
			.to_lowercase();
    let json_value = serde_json::from_str(&var_1).unwrap();
		let res = create_variable(
			"https://app.terraform.io/api/v2",
			&env::var("TFVE_TOKEN").unwrap(),
			&env::var("TFVE_WORKSPACE_ID").unwrap(),
			&vec![TerraformVariableProperty {
				variable_id: None,
				variable_name: var_1.clone(),
				value: json_value,
			}],
		)
		.await
		.unwrap();

		assert!(
			check_variable_status(
				"https://app.terraform.io/api/v2",
				&vec![res.get(0).unwrap().variable_id.clone()],
				&env::var("TFVE_TOKEN").unwrap(),
				&env::var("TFVE_WORKSPACE_ID").unwrap(),
			)
			.await
			.unwrap()[0]
				.already_exist
		)
	}
}
