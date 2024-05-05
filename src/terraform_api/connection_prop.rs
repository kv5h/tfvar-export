/// Properties for API connection
///
/// Official Doc: [HCP Terraform API Documentation](https://developer.hashicorp.com/terraform/cloud-docs/api-docs)
pub struct TerraformApiConnectionProperty {
    /// Base URL of API (Ex. `https://app.terraform.io/api/v2`)
    base_url: url::Url,
    /// Name of the organization
    organization_name: Option<String>,
    /// Authorization token
    token: String,
    /// The ID of the workspace
    workspace_id: Option<String>,
}

impl TerraformApiConnectionProperty {
    pub fn new(
        base_url: url::Url,
        organization_name: Option<String>,
        token: String,
        workspace_id: Option<String>,
    ) -> Self {
        Self {
            base_url,
            organization_name,
            token,
            workspace_id,
        }
    }

    pub fn base_url(&self) -> &url::Url {
        &self.base_url
    }

    pub fn organization_name(&self) -> &str {
        &self.organization_name.as_ref().unwrap().as_ref()
    }

    pub fn token(&self) -> &str {
        &self.token
    }

    pub fn workspace_id(&self) -> &str {
        &self.workspace_id.as_ref().unwrap().as_ref()
    }
}
