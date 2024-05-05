//! Update or create Terraform Cloud workspace variable.
//!
//! **API Reference:** https://developer.hashicorp.com/terraform/cloud-docs/api-docs/workspace-variables

use serde_json::json;
use std::collections::HashMap;

use crate::terraform_api::connection_prop::TerraformApiConnectionProperty;

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
    value: serde_json::Value,
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
    api_conn_prop: &TerraformApiConnectionProperty,
    terraform_variable_property: &Vec<TerraformVariableProperty>,
) -> Result<(), Box<dyn std::error::Error>> {
    let api_base_url = api_conn_prop.base_url();
    let token = api_conn_prop.token();
    let workspace_id = api_conn_prop.workspace_id();

    // Limit the rate 20 requests per second.
    let ratelimiter = ratelimit::Ratelimiter::builder(20, std::time::Duration::from_secs(1))
        .max_tokens(20)
        .initial_available(20)
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

        let response = reqwest::Client::new()
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

    log::info!("Variables updated: {}.", count);
    println!("Variables updated: {}.", count); // TODO:

    Ok(())
}

/// Create Terraform Workspace variable(s).
///
/// **Remark:** To prevent [`Rate Limiting`](https://developer.hashicorp.com/terraform/cloud-docs/api-docs#rate-limiting), limit the rate 20 requests per second.
pub async fn create_variable(
    api_conn_prop: &TerraformApiConnectionProperty,
    terraform_variable_property: &Vec<TerraformVariableProperty>,
) -> Result<Vec<TerraformVariableCreationResult>, Box<dyn std::error::Error>> {
    let api_base_url = api_conn_prop.base_url();
    let token = api_conn_prop.token();
    let workspace_id = api_conn_prop.workspace_id();

    let mut result = Vec::new();

    // Limit the rate 20 requests per second.
    let ratelimiter = ratelimit::Ratelimiter::builder(20, std::time::Duration::from_secs(1))
        .max_tokens(20)
        .initial_available(20)
        .build()
        .unwrap();
    let count = terraform_variable_property.len();
    for i in 0..count {
        if let Err(sleep) = ratelimiter.try_wait() {
            std::thread::sleep(sleep);
            continue;
        }

        let data = json!({
                  "data":{
                      "type": "vars",
                      "attributes": {
                          "key": terraform_variable_property[i].variable_name,
                          "value": json!(terraform_variable_property[i].value).to_string(),
                          "description": "",
                          "category": "terraform",
                          "hcl": true}}}
        );
        let mut map = HashMap::new();
        map.insert("data", data.to_string());

        let response = reqwest::Client::new()
            .post(format!("{}/workspaces/{}/vars", api_base_url, workspace_id))
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/vnd.api+json")
            .body(data.to_string())
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

    log::info!("Variables created: {}.", count);

    Ok(result)
}

pub async fn check_variable_status(
    api_conn_prop: &TerraformApiConnectionProperty,
    target_variable_ids: &Vec<String>,
) -> Result<Vec<TerraformVariableStatus>, Box<dyn std::error::Error>> {
    let api_base_url = api_conn_prop.base_url();
    let token = api_conn_prop.token();
    let workspace_id = api_conn_prop.workspace_id();

    let response = reqwest::Client::new()
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

    log::info!("Variable status: {:#?}", result);

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::distributions::{Alphanumeric, DistString};
    use std::env;
    use url::Url;

    use crate::terraform_api::connection_prop::TerraformApiConnectionProperty;

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
        let api_conn_prop = TerraformApiConnectionProperty::new(
            Url::parse("https://app.terraform.io/api/v2").unwrap(),
            None,
            env::var("TFVE_TOKEN").unwrap(),
            Some(env::var("TFVE_WORKSPACE_ID").unwrap().to_string()),
        );
        let res = check_variable_status(&api_conn_prop, &vec![
            var_1.clone(),
            "var-Tppa4XRHcAt7qniZ".to_string(),
            var_2.clone(),
            var_3.clone(),
        ])
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
        // NOTE:
        //   value: json!(["twitter-feed-rs", "tflambda"]),
        //   value: json!(&str),
        let var_1 = Alphanumeric
            .sample_string(&mut rand::thread_rng(), 32)
            .to_lowercase();
        let api_conn_prop = TerraformApiConnectionProperty::new(
            Url::parse("https://app.terraform.io/api/v2").unwrap(),
            None,
            env::var("TFVE_TOKEN").unwrap(),
            Some(env::var("TFVE_WORKSPACE_ID").unwrap().to_string()),
        );
        let res = create_variable(&api_conn_prop, &vec![TerraformVariableProperty {
            variable_id: None,
            variable_name: var_1.clone(),
            value: json!(["twitter-feed-rs", "tflambda"]),
        }])
        .await
        .unwrap();

        assert!(
            check_variable_status(&api_conn_prop, &vec![res
                .get(0)
                .unwrap()
                .variable_id
                .clone()],)
            .await
            .unwrap()[0]
                .already_exist
        )
    }
}
