//! Get variables from a workspace.

use std::collections::HashMap;

use crate::terraform_api::connection_prop::TerraformApiConnectionProperty;

/// Get variables from workspace and return a HashMap of `name : id` of variables.
pub async fn get_variables(
    workspace_id: &str,
    api_conn_prop: &TerraformApiConnectionProperty,
) -> Result<HashMap<String, String>, Box<dyn std::error::Error>> {
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

    // Map of `id : name`
    let mut result = HashMap::new();
    let response_json_value: serde_json::Value = serde_json::from_str(&response)?;
    response_json_value["data"]
        .as_array()
        .unwrap()
        .into_iter()
        .for_each(|val| {
            let variable_id = val["id"].as_str().unwrap().to_string();
            let variable_name = val["attributes"]["key"].as_str().unwrap().to_string();
            result.insert(variable_name, variable_id);
        });

    Ok(result)
}

#[cfg(test)]
mod tests {
    use rand::distributions::{Alphanumeric, DistString};
    use serde_json::json;

    use super::*;
    use crate::terraform_api::register_variable::{
        create_variable,
        tests::delete_variable,
        TerraformVariableProperty,
    };

    #[tokio::test]
    async fn test_get_variables() {
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
            std::env::var("TFVE_TOKEN").unwrap(),
        );
        let workspace_id = &std::env::var("TFVE_WORKSPACE_ID_TESTING")
            .expect("Environment variable `TFVE_WORKSPACE_ID_TESTING` required.");

        // Create temporary variables beforehand
        let test_data = vec![
            TerraformVariableProperty::new(None, var_1.clone(), json!(var_1)),
            TerraformVariableProperty::new(None, var_2.clone(), json!(var_2)),
            TerraformVariableProperty::new(None, var_3.clone(), json!(var_3)),
        ];

        // Get result from `create_variable`
        let creation_result: HashMap<String, String> =
            create_variable(workspace_id, &api_conn_prop, &test_data)
                .await
                .unwrap()
                .into_iter()
                .map(|val| {
                    (
                        val.get_variable_name().to_owned(),
                        val.get_variable_id().to_owned(),
                    )
                })
                .collect();

        let test_fn_result = get_variables(workspace_id, &api_conn_prop).await.unwrap();

        assert!(test_fn_result.get(&var_1).is_some());
        assert!(test_fn_result.get(&var_2).is_some());
        assert!(test_fn_result.get(&var_3).is_some());

        // Delete test data
        let ids = creation_result
            .iter()
            .map(|(_, id)| id.to_owned())
            .collect();
        delete_variable(&api_conn_prop, &ids).await.unwrap();
    }
}
