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
    value: serde_json::Value,
}

/// Update Terraform Workspace variable(s).
///
/// **Remark:** To prevent [`Rate Limiting`](https://developer.hashicorp.com/terraform/cloud-docs/api-docs#rate-limiting), limit the rate 20 requests per second.
pub async fn update_variable(
    api_conn_prop: &TerraformApiConnectionProperty,
    terraform_variable_property: &Vec<TerraformVariableProperty>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut url = api_conn_prop.base_url().clone();
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

        let path = format!("/api/v2/workspaces/{}/vars/{}", workspace_id, variable_id);
        url.set_path(&path);

        let response = reqwest::Client::new()
            .post(url.as_str())
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

    Ok(())
}

/// Create Terraform Workspace variable(s).
///
/// **Remark:** To prevent [`Rate Limiting`](https://developer.hashicorp.com/terraform/cloud-docs/api-docs#rate-limiting), limit the rate 20 requests per second.
pub async fn create_variable(
    api_conn_prop: &TerraformApiConnectionProperty,
    terraform_variable_property: &Vec<TerraformVariableProperty>,
) -> Result<Vec<TerraformVariableCreationResult>, Box<dyn std::error::Error>> {
    let mut url = api_conn_prop.base_url().clone();
    let token = api_conn_prop.token();
    let workspace_id = api_conn_prop.workspace_id();

    let path = format!("/api/v2/workspaces/{}/vars", workspace_id);
    url.set_path(&path);

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

        let is_hcl = match &terraform_variable_property[i].value {
            x if x.is_boolean()
                | x.is_f64()
                | x.is_i64()
                | x.is_number()
                | x.is_string()
                | x.is_u64() =>
            {
                false
            },
            _ => true,
        };

        let is_string = match &terraform_variable_property[i].value {
            x if x.is_string() => true,
            _ => false,
        };

        let data_value = if is_string {
            terraform_variable_property[i]
                .value
                .as_str()
                .unwrap()
                .to_string()
        } else {
            terraform_variable_property[i].value.to_string()
        };

        let data = json!({
            "data":{
                "type": "vars",
                "attributes": {
                    "key": terraform_variable_property[i].variable_name,
                    "value": data_value,
                    "description": "",
                    "category": "terraform",
                    "hcl": is_hcl
                  }
              }
        });
        let mut map = HashMap::new();
        map.insert("data", data.to_string());

        let response = reqwest::Client::new()
            .post(url.as_str())
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
        let value = if is_string {
            json_value["data"]["attributes"]["value"].clone()
        } else {
            serde_json::from_str::<serde_json::Value>(
                json_value["data"]["attributes"]["value"].as_str().unwrap(),
            )
            .unwrap()
        };
        result.push(TerraformVariableCreationResult {
            variable_id: json_value["data"]["id"].as_str().unwrap().to_string(),
            variable_name: json_value["data"]["attributes"]["key"]
                .as_str()
                .unwrap()
                .to_string(),
            value: value,
        });
    }

    log::info!("Variables created: {}.", count);

    Ok(result)
}

pub async fn check_variable_status(
    api_conn_prop: &TerraformApiConnectionProperty,
    target_variable_ids: &Vec<String>,
) -> Result<Vec<TerraformVariableStatus>, Box<dyn std::error::Error>> {
    let mut url = api_conn_prop.base_url().clone();
    let token = api_conn_prop.token();
    let workspace_id = api_conn_prop.workspace_id();

    let path = format!("/api/v2/workspaces/{}/vars", workspace_id);
    url.set_path(&path);

    let response = reqwest::Client::new()
        .get(url.as_str())
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
    use chrono::Local;
    use rand::distributions::{Alphanumeric, DistString};

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
            url::Url::parse("https://app.terraform.io").unwrap(),
            None,
            std::env::var("TFVE_TOKEN").unwrap(),
            Some(std::env::var("TFVE_WORKSPACE_ID").unwrap().to_string()),
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
        let api_conn_prop = TerraformApiConnectionProperty::new(
            url::Url::parse("https://app.terraform.io").unwrap(),
            None,
            std::env::var("TFVE_TOKEN").unwrap(),
            Some(
                std::env::var("TFVE_WORKSPACE_ID_TESTING")
                    .unwrap()
                    .to_string(),
            ),
        );

        let cases: Vec<serde_json::Value> = vec![
            json!("aaa\"bbb"),                        // string with quote
            json!("aaa"),                             // string
            json!(-1.2345),                           // negative float
            json!(0),                                 // number
            json!(1.2345),                            // float
            json!(["aaa", "bbb", "ccc"]),             // array
            json!([{"a":"aaa","b":"bbb","c":"ccc"}]), // list of map
            json!(false),                             // bool
            json!({"a":"aaa","b":"bbb","c":null}),    // map
            json!({"bool":{"sensitive":false,"type":"bool","value":false},"list_of_object":{"sensitive":false,"type":["object",{"a":"string","b":"string","c":"string"}],"value":{"a":"aaa","b":"bbb","c":null}},"map_of_string":{"sensitive":false,"type":["map","string"],"value":{"a":"aaa","b":"bbb","c":"ccc"}},"number_0":{"sensitive":false,"type":"number","value":0},"number_float":{"sensitive":false,"type":"number","value":1.2345},"number_negative":{"sensitive":false,"type":"number","value":-1.2345},"sensitive":{"sensitive":true,"type":"string","value":"**************"},"set_of_object":{"sensitive":false,"type":["set",["object",{"name":"string","type":"string"}]],"value":[{"name":"aaa","type":"bbb"}]},"string":{"sensitive":false,"type":"string","value":"aaa"},"string_with_quote":{"sensitive":false,"type":"string","value":"aaa\"bbb"},"tuple":{"sensitive":false,"type":["tuple",["string","string"]],"value":["aaa","bbb"]}}
            ), // complex
        ];
        for case in cases.iter() {
            let date = Local::now();
            let val = Alphanumeric
                .sample_string(&mut rand::thread_rng(), 8)
                .to_lowercase();
            let res = create_variable(&api_conn_prop, &vec![TerraformVariableProperty {
                variable_id: None,
                variable_name: format!("{}-{}", date.format("%Y%m%d%H%M%S%f"), val.clone()),
                value: case.clone(),
            }])
            .await
            .unwrap();

            let status = check_variable_status(&api_conn_prop, &vec![res
                .get(0)
                .unwrap()
                .variable_id
                .clone()])
            .await
            .unwrap();

            assert_eq!(status[0].already_exist, true);
            assert_eq!(
                &serde_json::from_str::<serde_json::Value>(&res[0].value.to_string()).unwrap(),
                case
            );
        }
    }
}
