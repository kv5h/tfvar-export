//! Get a list of Terraform Cloud workspaces.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::terraform_api::connection_prop::TerraformApiConnectionProperty;

/// Terraform Project info
#[derive(Debug, Serialize, Deserialize)]
struct TerraformProject {
    terraform_project_id: String,
    terraform_project_name: String,
}

/// Terraform Workspace info
#[derive(Debug, Serialize, Deserialize)]
pub struct TerraformWorkspace {
    terraform_workspace_id: String,
    terraform_workspace_name: String,
    terraform_project: TerraformProject,
}

/// Max element numbers per page.
/// - TODO: If your case exceeds this, additional implementations are required.
/// - Ref: https://developer.hashicorp.com/terraform/cloud-docs/api-docs/projects#list-projects
const TERRAFORM_API_QS_PAGE_SIZE: u8 = 100;

/// Get Terraform projects and return a HashMap of `Project ID: Project Name`.
///
/// ## Example
///
/// ```rust
/// let res: HashMap<String, String> = get_projects(api_conn_prop).await?;
/// ```
pub async fn get_projects(
    api_conn_prop: &TerraformApiConnectionProperty,
) -> Result<HashMap<String, String>, Box<dyn std::error::Error>> {
    let mut result = HashMap::new();

    let mut url = api_conn_prop.base_url().clone();
    let organization_name = api_conn_prop.organization_name();
    let token = api_conn_prop.token();

    let path = format!("/api/v2/organizations/{}/projects", organization_name);
    url.set_path(&path);

    let response_projects = reqwest::Client::new()
        .get(url.as_str())
        .header("Authorization", format!("Bearer {}", token))
        .header("Content-Type", "application/vnd.api+json")
        .query(&[("page[size]", TERRAFORM_API_QS_PAGE_SIZE)])
        .send()
        .await?
        .text()
        .await?;

    let response_projects_val: serde_json::Value = serde_json::from_str(&response_projects)?;
    response_projects_val["data"]
        .as_array()
        .unwrap()
        .into_iter()
        .for_each(|val| {
            let terraform_project_id = val["id"].as_str().unwrap().to_string();
            let terraform_project_name = val["attributes"]["name"].as_str().unwrap().to_string();

            result.insert(terraform_project_id, terraform_project_name);
        });

    log::info!("{} workspaces found.", result.len());

    Ok(result)
}

/// Get Terraform workspaces and return vector of `TerraformWorkspace` struct.
///
/// Using `--show-workspaces` flag prints workspaces with their associated project.
///
/// ## Example
///
/// ```rust
/// let res: Vec<TerraformWorkspace> =
///     get_workspaces(false, api_conn_prop).await?;
/// ```
pub async fn get_workspaces(
    show_workspaces: bool,
    api_conn_prop: &TerraformApiConnectionProperty,
) -> Result<Vec<TerraformWorkspace>, Box<dyn std::error::Error>> {
    let mut url = api_conn_prop.base_url().clone();
    let organization_name = api_conn_prop.organization_name();
    let token = api_conn_prop.token();

    let path = format!("/api/v2/organizations/{}/workspaces", organization_name);
    url.set_path(&path);

    let response_workspaces = reqwest::Client::new()
        .get(url.as_str())
        .header("Authorization", format!("Bearer {}", token))
        .header("Content-Type", "application/vnd.api+json")
        .query(&[("page[size]", TERRAFORM_API_QS_PAGE_SIZE)])
        .send()
        .await?
        .text()
        .await?;

    let response_workspaces_val: serde_json::Value = serde_json::from_str(&response_workspaces)?;
    let mut terraform_workspaces = Vec::new();
    let terraform_projects_map = get_projects(api_conn_prop).await?;
    response_workspaces_val["data"]
        .as_array()
        .unwrap()
        .into_iter()
        .for_each(|val| {
            let terraform_workspace_id = val["id"].as_str().unwrap().to_string();
            let terraform_workspace_name = val["attributes"]["name"].as_str().unwrap().to_string();
            let terraform_project_id = val["relationships"]["project"]["data"]["id"]
                .as_str()
                .unwrap()
                .to_string();

            terraform_workspaces.push(TerraformWorkspace {
                terraform_workspace_id,
                terraform_workspace_name,
                terraform_project: TerraformProject {
                    terraform_project_id: terraform_project_id.clone(),
                    terraform_project_name: terraform_projects_map
                        .get(&terraform_project_id)
                        .unwrap()
                        .to_string(),
                },
            })
        });

    log::info!("{} projects found.", terraform_workspaces.len());

    if show_workspaces {
        println!("{}", serde_json::to_string_pretty(&terraform_workspaces)?)
    }

    Ok(terraform_workspaces)
}
