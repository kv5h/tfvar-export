//! Get a list of Terraform workspaces.

use reqwest::Client;
use std::collections::HashMap;
use std::error::Error;

/// Terraform Project info
#[derive(Debug)]
struct TerraformProject {
	terraform_project_id: String,
	terraform_project_name: String,
}

/// Terraform Workspace info
#[derive(Debug)]
pub struct TerraformWorkspace {
	terraform_workspace_id: String,
	terraform_workspace_name: String,
	terraform_project: TerraformProject,
}

/// Max element numbers per page.
/// - TODO: If your case exceeds this, additional implementations are required.
/// - Ref: https://developer.hashicorp.com/terraform/cloud-docs/api-docs/projects#list-projects
const TERRAFORM_API_QS_PAGE_SIZE: u8 = 100;

/// Get Terraform projects and return HashMap of `Project ID: Project Name`.
///
/// # Example
///
/// ```rust
/// let res: HashMap<String, String> =
/// 	get_projects("https://app.terraform.io/api/v2", "ORG_NAME", "TOKEN")
/// 		.await?;
/// ```
pub async fn get_projects(
	api_base_url: &str,
	organization_name: &str,
	token: &str,
) -> Result<HashMap<String, String>, Box<dyn Error>> {
	let response_projects = Client::new()
		.get(format!(
			"{}/organizations/{}/projects",
			api_base_url, organization_name
		))
		.header("Authorization", format!("Bearer {}", token))
		.header("Content-Type", "application/vnd.api+json")
		.query(&[("page[size]", TERRAFORM_API_QS_PAGE_SIZE)])
		.send()
		.await?
		.text()
		.await?;

	let response_projects_val: serde_json::Value = serde_json::from_str(&response_projects)?;
	let mut terraform_projects_map = HashMap::new();
	response_projects_val["data"]
		.as_array()
		.unwrap()
		.into_iter()
		.for_each(|val| {
			let terraform_project_id = val["id"].as_str().unwrap().to_string();
			let terraform_project_name = val["attributes"]["name"].as_str().unwrap().to_string();

			terraform_projects_map.insert(terraform_project_id, terraform_project_name);
		});

	Ok(terraform_projects_map)
}

/// Get Terraform workspaces and return vector of `TerraformWorkspace` struct.
///
/// # Example
///
/// ```rust
/// let res: Vec<TerraformWorkspace> =
/// 	get_workspaces("https://app.terraform.io/api/v2", "ORG_NAME", "TOKEN")
/// 		.await?;
/// ```
pub async fn get_workspaces(
	api_base_url: &str,
	organization_name: &str,
	token: &str,
) -> Result<Vec<TerraformWorkspace>, Box<dyn Error>> {
	let response_workspaces = Client::new()
		.get(format!(
			"{}/organizations/{}/workspaces",
			api_base_url, organization_name
		))
		.header("Authorization", format!("Bearer {}", token))
		.header("Content-Type", "application/vnd.api+json")
		.query(&[("page[size]", TERRAFORM_API_QS_PAGE_SIZE)])
		.send()
		.await?
		.text()
		.await?;

	let response_workspaces_val: serde_json::Value = serde_json::from_str(&response_workspaces)?;
	let mut terraform_workspaces = Vec::new();
	let terraform_projects_map = get_projects(api_base_url, organization_name, token).await?;
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

	Ok(terraform_workspaces)
}
