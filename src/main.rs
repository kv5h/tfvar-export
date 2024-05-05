mod clap;
mod get_outputs;
mod terraform_api;

use crate::get_outputs::get_outputs;
use crate::terraform_api::connection_prop::TerraformApiConnectionProperty;
use crate::terraform_api::get_workspaces::get_workspaces;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Clap: Read command-line options
    let clap = clap::new_clap_command();
    let allow_update = clap.get_flag("allow_update");
    let base_url = clap.get_one::<String>("base_url").unwrap();
    let enable_info_log = clap.get_flag("enable_info_log");
    let show_output = clap.get_flag("show_outputs");
    let tfstate_file = clap.get_one::<String>("tfstate_file").unwrap();

    // Log
    let mut builder = env_logger::Builder::new();
    match enable_info_log {
        true => builder.filter_level(log::LevelFilter::Info),
        false => builder.filter_level(log::LevelFilter::Off),
    };
    builder.init();

    let api_conn_prop = TerraformApiConnectionProperty::new(
        url::Url::parse("https://app.terraform.io/api/v2").unwrap(),
        Some(std::env::var("TFVE_ORGANIZATION_NAME").unwrap()),
        std::env::var("TFVE_TOKEN").unwrap(),
        Some(std::env::var("TFVE_WORKSPACE_ID").unwrap()),
    );

    get_workspaces(&api_conn_prop).await?;

    get_outputs(show_output, &tfstate_file)?;

    Ok(())
}
