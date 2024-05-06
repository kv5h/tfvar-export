mod clap;
mod get_outputs;
mod read_export_list;
mod terraform_api;

use read_export_list::read_export_list;

use crate::get_outputs::get_outputs;
use crate::terraform_api::connection_prop::TerraformApiConnectionProperty;
use crate::terraform_api::get_workspaces::get_workspaces;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Clap: Read command-line options
    let clap = clap::new_clap_command();
    let base_url = clap.get_one::<String>("base_url").unwrap();
    let enable_info_log = clap.get_flag("enable_info_log");
    let show_outputs = clap.get_flag("show_outputs"); // TODO: Show and quit
    let show_workspaces = clap.get_flag("show_workspaces"); // TODO: Show and quit
    let allow_update = clap.get_flag("allow_update");
    let output_values_file = clap.get_one::<String>("output_values_file").unwrap();
    let export_list = clap.get_one::<String>("export_list").unwrap();

    // Log
    let mut builder = env_logger::Builder::new();
    match enable_info_log {
        true => builder.filter_level(log::LevelFilter::Info),
        false => builder.filter_level(log::LevelFilter::Error),
    };
    builder.init();

    let api_conn_prop = TerraformApiConnectionProperty::new(
        url::Url::parse(&base_url).expect("Failed to parse `base_url`."),
        Some(std::env::var("TFVE_ORGANIZATION_NAME").expect(&format!(
            "Failed to read an environment variable `{}`.",
            "TFVE_ORGANIZATION_NAME"
        ))),
        std::env::var("TFVE_TOKEN").expect(&format!(
            "Failed to read an environment variable `{}`.",
            "TFVE_TOKEN"
        )),
        Some(std::env::var("TFVE_WORKSPACE_ID").expect(&format!(
            "Failed to read an environment variable `{}`.",
            "TFVE_WORKSPACE_ID"
        ))),
    );

    let _workspaces = get_workspaces(show_workspaces, &api_conn_prop).await?;

    get_outputs(show_outputs, &output_values_file)?;

    let el = read_export_list(&export_list).unwrap();
    el.unwrap().iter().for_each(|(k, v)| {
        println!("{} : {}", k, v);
    });

    Ok(())
}
