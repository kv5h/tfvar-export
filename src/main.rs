mod clap;
mod terraform_api;

use crate::terraform_api::get_workspaces::get_workspaces;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	// Clap: Read command-line options
	let clap = clap::new_clap_command();
	let base_url = clap.get_one::<String>("base_url").unwrap();

	get_workspaces(
		&base_url,
		&env::var("TFVE_ORGANIZATION_NAME").unwrap(),
		&env::var("TFVE_TOKEN").unwrap(),
	)
	.await?;

	Ok(())
}
