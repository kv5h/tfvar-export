mod clap;
mod get_outputs;
mod terraform_api;

use crate::get_outputs::get_outputs;
use crate::terraform_api::get_workspaces::get_workspaces;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	// Clap: Read command-line options
	let clap = clap::new_clap_command();
	let base_url = clap.get_one::<String>("base_url").unwrap();
	let show_output = clap.get_flag("show_outputs");
	let allow_update = clap.get_flag("allow_update");
	let tfstate_file = clap.get_one::<String>("tfstate_file").unwrap();

	get_workspaces(
		&base_url,
		&env::var("TFVE_ORGANIZATION_NAME").unwrap(),
		&env::var("TFVE_TOKEN").unwrap(),
	)
	.await?;

	get_outputs(show_output, &tfstate_file)?;

	Ok(())
}
