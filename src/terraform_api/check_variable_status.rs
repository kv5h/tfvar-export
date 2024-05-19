//! Checks the status of Terraform variables.

use std::collections::HashMap;

use crate::terraform_api::connection_prop::TerraformApiConnectionProperty;

/// Terraform variable status
#[derive(Debug, Eq, PartialEq)]
pub struct TerraformVariableStatus {
    variable_name: String,
    variable_id: Option<String>,
}

impl TerraformVariableStatus {
    pub fn get_variable_name(&self) -> &str {
        &self.variable_name
    }

    pub fn get_variable_id(&self) -> &Option<String> {
        &self.variable_id
    }
}

/// Checks specified variables are already exist or not.
pub async fn check_variable_status(
    workspace_id: &str,
    api_conn_prop: &TerraformApiConnectionProperty,
    target_variable_names: &Vec<String>,
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
    let mut existing_variables = HashMap::new();
    let response_json_value: serde_json::Value = serde_json::from_str(&response)?;
    response_json_value["data"]
        .as_array()
        .unwrap()
        .into_iter()
        .for_each(|val| {
            existing_variables.insert(
                val["attributes"]["key"].as_str().unwrap().to_string(),
                val["id"].as_str().unwrap().to_string(),
            );
        });

    let mut result: Vec<TerraformVariableStatus> = Vec::new();
    target_variable_names
        .iter()
        .for_each(|val_name| match existing_variables.get(val_name) {
            Some(val_id) => result.push(TerraformVariableStatus {
                variable_name: val_name.to_owned(),
                variable_id: Some(val_id.to_owned()),
            }),
            None => result.push(TerraformVariableStatus {
                variable_name: val_name.to_owned(),
                variable_id: None,
            }),
        });

    log::info!("Variable status: {:#?}", result);

    Ok(result)
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::terraform_api::register_variable::{
        create_variable,
        tests::delete_variable,
        TerraformVariableProperty,
    };

    #[tokio::test]
    async fn test_check_variable_status() {
        // Should NOT exist
        let test_val_1 = uuid::Uuid::new_v4().to_string();
        // Should exist
        let test_val_2 = uuid::Uuid::new_v4().to_string();
        // Should NOT exist
        let test_val_3 = uuid::Uuid::new_v4().to_string();
        // Should exist
        let test_val_4 = uuid::Uuid::new_v4().to_string();
        // Should NOT exist
        let test_val_5 = uuid::Uuid::new_v4().to_string();

        let api_conn_prop = TerraformApiConnectionProperty::new(
            url::Url::parse("https://app.terraform.io").unwrap(),
            std::env::var("TFVE_TOKEN").unwrap(),
        );
        let workspace_id = &std::env::var("TFVE_WORKSPACE_ID_TESTING")
            .expect("Environment variable `TFVE_WORKSPACE_ID_TESTING` required.");

        let create_result = create_variable(workspace_id, &api_conn_prop, &vec![
            TerraformVariableProperty::new(
                None,
                test_val_2.clone(),
                Some(test_val_2.clone()),
                json!(test_val_2),
            ),
            TerraformVariableProperty::new(
                None,
                test_val_4.clone(),
                Some(test_val_4.clone()),
                json!(test_val_4),
            ),
        ])
        .await
        .unwrap();

        let res = check_variable_status(workspace_id, &api_conn_prop, &vec![
            test_val_1.clone(),
            test_val_2.clone(),
            test_val_3.clone(),
            test_val_4.clone(),
            test_val_5.clone(),
        ])
        .await
        .unwrap();

        assert!(res.get(0).unwrap().get_variable_id().is_none());
        assert!(res.get(1).unwrap().get_variable_id().is_some());
        assert!(res.get(2).unwrap().get_variable_id().is_none());
        assert!(res.get(3).unwrap().get_variable_id().is_some());
        assert!(res.get(4).unwrap().get_variable_id().is_none());

        // Delete test data
        let ids = create_result
            .iter()
            .map(|val| val.get_variable_id().to_owned())
            .collect();
        delete_variable(&api_conn_prop, &ids, workspace_id)
            .await
            .unwrap();
    }
}
