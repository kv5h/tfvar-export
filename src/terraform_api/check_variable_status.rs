//! Checks the status of Terraform variables.

use std::collections::HashMap;

use crate::terraform_api::connection_prop::TerraformApiConnectionProperty;

/// Terraform variable status
#[derive(Debug, Eq, PartialEq)]
pub struct TerraformVariableStatus {
    variable_id: String,
    variable_name: Option<String>,
}

impl TerraformVariableStatus {
    pub fn get_variable_id(&self) -> &str {
        &self.variable_id
    }

    pub fn get_variable_name(&self) -> &Option<String> {
        &self.variable_name
    }
}

/// Checks specified variables are already exist or not.
pub async fn check_variable_status(
    workspace_id: &str,
    api_conn_prop: &TerraformApiConnectionProperty,
    target_variable_ids: &Vec<String>,
) -> Result<Vec<TerraformVariableStatus>, Box<dyn std::error::Error>> {
    let mut url = api_conn_prop.base_url().clone();
    let token = api_conn_prop.token();

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

    //  `(name, id)` of existing variables
    let mut vars_already_exist = HashMap::new();
    let response_json_value: serde_json::Value = serde_json::from_str(&response)?;
    response_json_value["data"]
        .as_array()
        .unwrap()
        .into_iter()
        .for_each(|val| {
            vars_already_exist.insert(
                val["id"].as_str().unwrap().to_string(),
                val["attributes"]["key"].as_str().unwrap().to_string(),
            );
        });

    let mut result = Vec::new();
    target_variable_ids
        .iter()
        .for_each(|val| match vars_already_exist.get(val) {
            Some(name) => result.push(TerraformVariableStatus {
                variable_id: val.to_string(),
                variable_name: Some(name.to_owned()),
            }),
            None => result.push(TerraformVariableStatus {
                variable_id: val.to_string(),
                variable_name: None,
            }),
        });

    log::info!("Variable status: {:#?}", result);

    Ok(result)
}

#[cfg(test)]
mod tests {
    use rand::distributions::{Alphanumeric, DistString};
    use serde_json::json;

    use super::*;
    use crate::terraform_api::register_variable::{create_variable, TerraformVariableProperty};

    #[tokio::test]
    async fn test_check_variable_status() {
        // Should NOT exist
        let var_1 = Alphanumeric
            .sample_string(&mut rand::thread_rng(), 32)
            .to_lowercase();
        // Should exist
        let var_2 = Alphanumeric
            .sample_string(&mut rand::thread_rng(), 32)
            .to_lowercase();
        // Should NOT exist
        let var_3 = Alphanumeric
            .sample_string(&mut rand::thread_rng(), 32)
            .to_lowercase();
        // Should exist
        let var_4 = Alphanumeric
            .sample_string(&mut rand::thread_rng(), 32)
            .to_lowercase();
        // Should NOT exist
        let var_5 = Alphanumeric
            .sample_string(&mut rand::thread_rng(), 32)
            .to_lowercase();

        let api_conn_prop = TerraformApiConnectionProperty::new(
            url::Url::parse("https://app.terraform.io").unwrap(),
            std::env::var("TFVE_TOKEN").unwrap(),
        );
        let workspace_id = &std::env::var("TFVE_WORKSPACE_ID_TESTING")
            .expect("Environment variable `TFVE_WORKSPACE_ID_TESTING` required.");

        let _ = create_variable(workspace_id, &api_conn_prop, &vec![
            TerraformVariableProperty::new(None, var_2.clone(), json!(var_2)),
            TerraformVariableProperty::new(None, var_4.clone(), json!(var_4)),
        ])
        .await
        .unwrap();

        let res = check_variable_status(workspace_id, &api_conn_prop, &vec![
            var_1.clone(),
            var_2.clone(),
            var_3.clone(),
            var_4.clone(),
            var_5.clone(),
        ])
        .await
        .unwrap();

        assert_eq!(res, vec![
            TerraformVariableStatus {
                variable_id: var_1,
                variable_name: None
            },
            TerraformVariableStatus {
                variable_id: var_2.clone(),
                variable_name: Some(var_2)
            },
            TerraformVariableStatus {
                variable_id: var_3,
                variable_name: None
            },
            TerraformVariableStatus {
                variable_id: var_4.clone(),
                variable_name: Some(var_4)
            },
            TerraformVariableStatus {
                variable_id: var_5,
                variable_name: None
            },
        ])
    }
}
