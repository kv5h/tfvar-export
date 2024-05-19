//! Update or create Terraform Cloud workspace variable.
//!
//! **API Reference:** https://developer.hashicorp.com/terraform/cloud-docs/api-docs/workspace-variables

use std::collections::HashMap;

use log::info;
use serde_json::json;

use crate::terraform_api::connection_prop::TerraformApiConnectionProperty;

/// Terraform variable property
#[derive(Debug)]
pub struct TerraformVariableProperty {
    variable_id: Option<String>,
    variable_name: String,
    variable_description: Option<String>,
    value: serde_json::Value,
}

impl TerraformVariableProperty {
    pub fn new(
        variable_id: Option<String>,
        variable_name: String,
        variable_description: Option<String>,
        value: serde_json::Value,
    ) -> Self {
        Self {
            variable_id,
            variable_name,
            variable_description,
            value,
        }
    }

    fn get_variable_id(&self) -> &Option<String> {
        &self.variable_id
    }

    fn get_variable_name(&self) -> &str {
        &self.variable_name
    }

    fn get_variable_description(&self) -> &Option<String> {
        &self.variable_description
    }

    fn get_value(&self) -> &serde_json::Value {
        &self.value
    }
}

/// Terraform variable Create/Update result
#[derive(Debug)]
#[allow(dead_code)]
pub struct TerraformVariableRegistrationResult {
    variable_id: String,
    variable_name: String,
    variable_description: String,
    value: serde_json::Value,
}

impl TerraformVariableRegistrationResult {
    #[cfg(test)]
    pub fn get_variable_id(&self) -> &str {
        &self.variable_id
    }

    #[cfg(test)]
    pub fn get_variable_name(&self) -> &str {
        &self.variable_name
    }

    #[cfg(test)]
    pub fn get_variable_description(&self) -> &str {
        &self.variable_description
    }

    #[cfg(test)]
    pub fn get_value(&self) -> &serde_json::Value {
        &self.value
    }
}

/// Update Terraform Workspace variable(s).
///
/// **Remark:** To prevent [`Rate Limiting`](https://developer.hashicorp.com/terraform/cloud-docs/api-docs#rate-limiting), limit the rate 20 requests per second.
pub async fn update_variable(
    workspace_id: &str,
    api_conn_prop: &TerraformApiConnectionProperty,
    terraform_variable_property: &Vec<TerraformVariableProperty>,
) -> Result<Vec<TerraformVariableRegistrationResult>, Box<dyn std::error::Error>> {
    let mut url = api_conn_prop.base_url().clone();
    let token = api_conn_prop.token();

    info!("Processing workspace ID: {}.", workspace_id);

    let mut result = Vec::new();

    // Limit the rate 20 requests per second.
    let ratelimiter = ratelimit::Ratelimiter::builder(20, std::time::Duration::from_secs(1))
        .max_tokens(20)
        .initial_available(20)
        .build()
        .unwrap();
    let count = terraform_variable_property.len();
    for i in 0..count {
        let path = format!(
            "/api/v2/workspaces/{}/vars/{}",
            workspace_id,
            terraform_variable_property
                .get(i)
                .unwrap()
                .get_variable_id()
                .clone()
                .unwrap()
        );
        url.set_path(&path);

        if let Err(sleep) = ratelimiter.try_wait() {
            std::thread::sleep(sleep);
            continue;
        }

        let is_hcl = match &terraform_variable_property.get(i).unwrap().get_value() {
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

        let is_string = match &terraform_variable_property.get(i).unwrap().get_value() {
            x if x.is_string() => true,
            _ => false,
        };

        let description = match &terraform_variable_property
            .get(i)
            .unwrap()
            .get_variable_description()
        {
            Some(val) => val,
            None => "",
        };

        let data_value = if is_string {
            terraform_variable_property
                .get(i)
                .unwrap()
                .get_value()
                .as_str()
                .unwrap()
                .to_string()
        } else {
            terraform_variable_property
                .get(i)
                .unwrap()
                .get_value()
                .to_string()
        };

        let data = json!({
            "data":{
                "id": terraform_variable_property.get(i).unwrap().get_variable_id().clone().unwrap(),
                "type": "vars",
                "attributes": {
                    "key": terraform_variable_property.get(i).unwrap().get_variable_name(),
                    "value": data_value,
                    "description": description,
                    "category": "terraform",
                    "hcl": is_hcl
                  }
              }
        });
        let mut map = HashMap::new();
        map.insert("data", data.to_string());

        let response = reqwest::Client::new()
            .patch(url.as_str())
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/vnd.api+json")
            .body(data.to_string())
            .send()
            .await?;

        assert!(
            response.status() == 200,
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
        result.push(TerraformVariableRegistrationResult {
            variable_id: json_value["data"]["id"].as_str().unwrap().to_string(),
            variable_name: json_value["data"]["attributes"]["key"]
                .as_str()
                .unwrap()
                .to_string(),
            variable_description: json_value["data"]["attributes"]["description"]
                .as_str()
                .unwrap()
                .to_string(),
            value,
        });
    }

    log::info!("{} Variable(s) successfully updated.", count);

    Ok(result)
}

/// Create Terraform Workspace variable(s).
///
/// **Remark:** To prevent [`Rate Limiting`](https://developer.hashicorp.com/terraform/cloud-docs/api-docs#rate-limiting), limit the rate 20 requests per second.
pub async fn create_variable(
    workspace_id: &str,
    api_conn_prop: &TerraformApiConnectionProperty,
    terraform_variable_property: &Vec<TerraformVariableProperty>,
) -> Result<Vec<TerraformVariableRegistrationResult>, Box<dyn std::error::Error>> {
    let mut url = api_conn_prop.base_url().clone();
    let token = api_conn_prop.token();

    let path = format!("/api/v2/workspaces/{}/vars", workspace_id);
    url.set_path(&path);

    info!("Processing workspace ID: {}.", workspace_id);

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

        let is_hcl = match &terraform_variable_property.get(i).unwrap().get_value() {
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

        let is_string = match &terraform_variable_property.get(i).unwrap().get_value() {
            x if x.is_string() => true,
            _ => false,
        };

        let description = match &terraform_variable_property
            .get(i)
            .unwrap()
            .get_variable_description()
        {
            Some(val) => val,
            None => "",
        };

        let data_value = if is_string {
            terraform_variable_property
                .get(i)
                .unwrap()
                .get_value()
                .as_str()
                .unwrap()
                .to_string()
        } else {
            terraform_variable_property
                .get(i)
                .unwrap()
                .get_value()
                .to_string()
        };

        let data = json!({
            "data":{
                "type": "vars",
                "attributes": {
                    "key": terraform_variable_property.get(i).unwrap().get_variable_name(),
                    "value": data_value,
                    "description": description,
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
        result.push(TerraformVariableRegistrationResult {
            variable_id: json_value["data"]["id"].as_str().unwrap().to_string(),
            variable_name: json_value["data"]["attributes"]["key"]
                .as_str()
                .unwrap()
                .to_string(),
            variable_description: json_value["data"]["attributes"]["description"]
                .as_str()
                .unwrap()
                .to_string(),
            value,
        });
    }

    log::info!("{} Variable(s) successfully created.", count);

    Ok(result)
}

#[cfg(test)]
pub mod tests {

    use super::*;
    use crate::terraform_api::check_variable_status::check_variable_status;

    // Function for deleting test data
    // Call on demand.
    pub async fn delete_variable(
        api_conn_prop: &TerraformApiConnectionProperty,
        variable_ids: &Vec<String>,
        workspace_id: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut url = api_conn_prop.base_url().clone();
        let token = api_conn_prop.token();

        // Limit the rate 20 requests per second.
        let ratelimiter = ratelimit::Ratelimiter::builder(20, std::time::Duration::from_secs(1))
            .max_tokens(20)
            .initial_available(20)
            .build()
            .unwrap();

        let count = variable_ids.len();
        for i in 0..count {
            if let Err(sleep) = ratelimiter.try_wait() {
                std::thread::sleep(sleep);
                continue;
            }

            let variable_id = &variable_ids.get(i).expect("Failed to get variable_id.");
            let path = format!("/api/v2/workspaces/{}/vars/{}", workspace_id, variable_id);
            url.set_path(&path);

            let response = reqwest::Client::new()
                .delete(url.as_str())
                .header("Authorization", format!("Bearer {}", token))
                .header("Content-Type", "application/vnd.api+json")
                .send()
                .await?;

            assert!(
                response.status() == 204,
                "Response status is {}.",
                response.status()
            );

            println!("Temporarily created variable deleted: {}.", variable_id);
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_update_variable() {
        let api_conn_prop = TerraformApiConnectionProperty::new(
            url::Url::parse("https://app.terraform.io").unwrap(),
            std::env::var("TFVE_TOKEN").unwrap(),
        );

        let cases: Vec<serde_json::Value> = vec![
            json!("aaa"),   // string
            json!(-1.2345), // negative float
        ];

        // Workspaces for testing
        let workspaces_ids = vec![
            std::env::var("TFVE_WORKSPACE_ID_TESTING")
                .expect("Environment variable `TFVE_WORKSPACE_ID_TESTING` required."),
            std::env::var("TFVE_WORKSPACE_ID_TESTING2")
                .expect("Environment variable `TFVE_WORKSPACE_ID_TESTING2` required."),
        ];

        // Iterates over workspaces
        for workspace_id in workspaces_ids.into_iter() {
            // Iterates over cases
            for case in cases.iter() {
                let test_val = uuid::Uuid::new_v4().to_string();
                // Create temporary variable to be updated
                let res = create_variable(&workspace_id, &api_conn_prop, &vec![
                    TerraformVariableProperty {
                        variable_id: None,
                        variable_name: test_val.to_owned(),
                        variable_description: None,
                        value: case.clone(),
                    },
                ])
                .await
                .unwrap();

                let status = check_variable_status(&workspace_id, &api_conn_prop, &vec![res
                    .get(0)
                    .unwrap()
                    .get_variable_name()
                    .to_owned()
                    .clone()])
                .await
                .unwrap();

                // Exec update
                let res_update = update_variable(&workspace_id, &api_conn_prop, &vec![
                    TerraformVariableProperty {
                        variable_id: Some(
                            status.get(0).unwrap().get_variable_id().clone().unwrap(),
                        ),
                        variable_name: test_val.to_owned(),
                        variable_description: Some(test_val.to_owned()),
                        value: json!("updated_val"),
                    },
                ])
                .await
                .unwrap();

                // Value
                assert_eq!(
                    json!("updated_val"),
                    res_update.get(0).unwrap().get_value().to_owned()
                );
                // Description
                assert_eq!(
                    test_val,
                    res_update.get(0).unwrap().get_variable_description()
                );

                // Delete test data
                delete_variable(
                    &api_conn_prop,
                    &vec![status.get(0).unwrap().get_variable_id().clone().unwrap()],
                    &workspace_id,
                )
                .await
                .unwrap();
            }
        }
    }

    #[tokio::test]
    async fn test_create_variable_with_description_short() {
        let api_conn_prop = TerraformApiConnectionProperty::new(
            url::Url::parse("https://app.terraform.io").unwrap(),
            std::env::var("TFVE_TOKEN").unwrap(),
        );

        let workspace_id = &std::env::var("TFVE_WORKSPACE_ID_TESTING")
            .expect("Environment variable `TFVE_WORKSPACE_ID_TESTING` required.");

        let cases: Vec<serde_json::Value> = vec![
            json!("aaa\"bbb"), // string with quote
            json!("aaa"),      // string
            json!(-1.2345),    // negative float
        ];

        // Temporary variable IDs to be deleted after testing
        let mut variable_ids = Vec::new();
        // Iterates over cases
        for case in cases.iter() {
            let test_val = uuid::Uuid::new_v4().to_string();
            let res = create_variable(workspace_id, &api_conn_prop, &vec![
                TerraformVariableProperty {
                    variable_id: None,
                    variable_name: test_val.to_owned(),
                    variable_description: Some(test_val.to_owned()),
                    value: case.clone(),
                },
            ])
            .await
            .unwrap();

            let status = check_variable_status(workspace_id, &api_conn_prop, &vec![res
                .get(0)
                .unwrap()
                .variable_name
                .clone()])
            .await
            .unwrap();

            // Variable ID should be Some
            assert!(status.get(0).unwrap().get_variable_id().is_some());
            // Value
            assert_eq!(
                &serde_json::from_str::<serde_json::Value>(
                    &res.get(0).unwrap().get_value().to_string()
                )
                .unwrap(),
                case
            );
            // Description
            assert_eq!(
                res.get(0).unwrap().get_variable_description().to_owned(),
                test_val
            );

            variable_ids.push(status.get(0).unwrap().get_variable_id().clone().unwrap());
        }
        // Delete test data
        delete_variable(&api_conn_prop, &variable_ids, workspace_id)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_create_variable_without_description_short() {
        let api_conn_prop = TerraformApiConnectionProperty::new(
            url::Url::parse("https://app.terraform.io").unwrap(),
            std::env::var("TFVE_TOKEN").unwrap(),
        );

        let workspace_id = &std::env::var("TFVE_WORKSPACE_ID_TESTING")
            .expect("Environment variable `TFVE_WORKSPACE_ID_TESTING` required.");

        let cases: Vec<serde_json::Value> = vec![
            json!("aaa\"bbb"), // string with quote
            json!(-1.2345),    // negative float
        ];

        // Temporary variable IDs to be deleted after testing
        let mut variable_ids = Vec::new();
        // Iterates over cases
        for case in cases.iter() {
            let test_val = uuid::Uuid::new_v4().to_string();
            let res = create_variable(workspace_id, &api_conn_prop, &vec![
                TerraformVariableProperty {
                    variable_id: None,
                    variable_name: test_val.to_owned(),
                    variable_description: None,
                    value: case.clone(),
                },
            ])
            .await
            .unwrap();

            let status = check_variable_status(workspace_id, &api_conn_prop, &vec![res
                .get(0)
                .unwrap()
                .variable_name
                .clone()])
            .await
            .unwrap();

            // Variable ID should be Some
            assert!(status.get(0).unwrap().get_variable_id().is_some());
            // Value
            assert_eq!(
                &serde_json::from_str::<serde_json::Value>(
                    &res.get(0).unwrap().get_value().to_string()
                )
                .unwrap(),
                case
            );
            // Description
            assert_eq!(
                res.get(0).unwrap().get_variable_description().to_owned(),
                ""
            );

            variable_ids.push(status.get(0).unwrap().get_variable_id().clone().unwrap());
        }
        // Delete test data
        delete_variable(&api_conn_prop, &variable_ids, workspace_id)
            .await
            .unwrap();
    }

    #[tokio::test]
    #[ignore]
    async fn test_create_variable_full() {
        let api_conn_prop = TerraformApiConnectionProperty::new(
            url::Url::parse("https://app.terraform.io").unwrap(),
            std::env::var("TFVE_TOKEN").unwrap(),
        );

        let workspace_id = &std::env::var("TFVE_WORKSPACE_ID_TESTING")
            .expect("Environment variable `TFVE_WORKSPACE_ID_TESTING` required.");

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

        // Temporary variable IDs to be deleted after testing
        let mut variable_ids = Vec::new();
        // Iterates over cases
        for case in cases.iter() {
            let test_val = uuid::Uuid::new_v4().to_string();
            let res = create_variable(workspace_id, &api_conn_prop, &vec![
                TerraformVariableProperty {
                    variable_id: None,
                    variable_name: test_val.to_owned(),
                    variable_description: Some(test_val.to_owned()),
                    value: case.clone(),
                },
            ])
            .await
            .unwrap();

            let status = check_variable_status(workspace_id, &api_conn_prop, &vec![res
                .get(0)
                .unwrap()
                .get_variable_name()
                .to_owned()
                .clone()])
            .await
            .unwrap();

            // Variable ID should be Some
            assert!(status.get(0).unwrap().get_variable_id().is_some());
            // Value
            assert_eq!(
                &serde_json::from_str::<serde_json::Value>(&res.get(0).unwrap().value.to_string())
                    .unwrap(),
                case
            );
            // Description
            assert_eq!(
                res.get(0).unwrap().get_variable_description().to_owned(),
                test_val
            );

            variable_ids.push(status.get(0).unwrap().get_variable_id().clone().unwrap());
        }
        // Delete test data
        delete_variable(&api_conn_prop, &variable_ids, workspace_id)
            .await
            .unwrap();
    }
}
