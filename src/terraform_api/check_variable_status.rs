//! Checks the status of Terraform variables.

use crate::terraform_api::connection_prop::TerraformApiConnectionProperty;

/// Terraform variable status
#[derive(Debug, Eq, PartialEq)]
pub struct TerraformVariableStatus {
    already_exist: bool,
    variable_id: String,
}

impl TerraformVariableStatus {
    pub fn get_already_exist(&self) -> &bool {
        &self.already_exist
    }

    pub fn get_variable_id(&self) -> &str {
        &self.variable_id
    }
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
    use rand::distributions::{Alphanumeric, DistString};

    use super::*;

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
}
