/// Properties for API connection
///
/// Official Doc: [HCP Terraform API Documentation](https://developer.hashicorp.com/terraform/cloud-docs/api-docs)
pub struct TerraformApiConnectionProperty {
    /// Base URL of API (Ex. `https://app.terraform.io`)
    base_url: url::Url,
    /// Authorization token
    token: String,
}

impl TerraformApiConnectionProperty {
    pub fn new(base_url: url::Url, token: String) -> Self {
        Self { base_url, token }
    }

    pub fn base_url(&self) -> &url::Url {
        &self.base_url
    }

    pub fn token(&self) -> &str {
        &self.token
    }
}
