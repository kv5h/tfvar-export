mod terraform_api;
mod utils;

use std::{collections::HashMap, process::id};

use crate::{
    terraform_api::{
        check_variable_status::check_variable_status,
        connection_prop::TerraformApiConnectionProperty,
        get_variables::get_variables,
        get_workspaces::get_workspaces,
        register_variable::{create_variable, TerraformVariableProperty},
    },
    utils::{get_outputs::get_outputs, read_export_list::read_export_list},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Clap: Read command-line options
    let clap = utils::clap::new_clap_command();
    let base_url = clap.get_one::<String>("base_url").unwrap();
    let target_workspaces = clap.get_one::<String>("target_workspaces");
    let enable_info_log = clap.get_flag("enable_info_log");
    let show_workspaces = clap.get_flag("show_workspaces");
    let allow_update = clap.get_flag("allow_update");
    let output_values_file = clap.try_get_one::<String>("output_values_file");
    let export_list = clap.try_get_one::<String>("export_list");

    // Log
    let mut builder = env_logger::Builder::new();
    match enable_info_log {
        true => builder.filter_level(log::LevelFilter::Info),
        false => builder.filter_level(log::LevelFilter::Error),
    };
    builder.init();

    let organization_name = match std::env::var("TFVE_ORGANIZATION_NAME") {
        Ok(x) => x,
        _ => {
            // `TFVE_ORGANIZATION_NAME` must be set if `show_workspaces` is specified
            if show_workspaces {
                panic!(
                    "Failed to read an environment variable `{}`.",
                    "TFVE_ORGANIZATION_NAME"
                );
            }
            String::new()
        },
    };

    let api_conn_prop = TerraformApiConnectionProperty::new(
        url::Url::parse(&base_url).expect("Failed to parse `base_url`."),
        std::env::var("TFVE_TOKEN").expect(&format!(
            "Failed to read an environment variable `{}`.",
            "TFVE_TOKEN"
        )),
    );

    if show_workspaces {
        get_workspaces(true, &organization_name, &api_conn_prop).await?;
        return Ok(());
    }

    // Workspace
    let workspaces = get_workspaces(false, &organization_name, &api_conn_prop).await?;
    // Map of Workspace Name and ID
    let workspace_name_id: HashMap<String, String> = workspaces
        .into_iter()
        .map(|val| {
            (
                val.get_workspace_name().to_string(),
                val.get_workspace_id().to_string(),
            )
        })
        .collect();
    let workspace_names: Vec<String> = target_workspaces.unwrap()
        .split(',')
        .map(|val| val.to_string())
        .collect();
    let workspace_ids: Vec<String> = workspace_names
        .into_iter()
        .map(|val| workspace_name_id.get(&val).unwrap().to_owned())
        .collect();

    let export_map = read_export_list(&export_list.clone().unwrap().unwrap()).unwrap().unwrap();
    let variable_names: Vec<String> = export_map
        .iter()
        .map(|(_, variable)| variable.to_string())
        .collect();

    // Outputs
    let outputs_list = get_outputs(&export_list.unwrap().unwrap()).unwrap();

    for workspace_id in workspace_ids {
        let variable_name_id = get_variables(&workspace_id, &api_conn_prop).await?;
        let variable_id_name: HashMap<String, String> = variable_name_id
            .iter()
            .map(|(name, id)| (id.to_owned(), name.to_owned()))
            .collect();

        // Vector of tuple `(id, name, index)`
        let existing_vars: Vec<(String, String, usize)> = variable_names
            .iter()
            .enumerate()
            .filter(|(_, name)| variable_name_id.get(*name).is_some())
            .map(|(idx, name)| {
                (
                    variable_name_id.get(name).unwrap().to_owned(),
                    name.to_owned(),
                    idx,
                )
            })
            .collect();
        // Update // TODO:

        // Vector of tuple `(name, idx)``
        let new_vars: Vec<(String, usize)> = variable_names
            .iter()
            .enumerate()
            .filter(|(_, name)| variable_name_id.get(*name).is_none())
            .map(|(idx, name)| (name.to_owned(), idx))
            .collect();

        for (name, idx) in new_vars.into_iter() {
            create_variable(&workspace_id, &api_conn_prop, &vec![
                TerraformVariableProperty::new(
                    None,
                    name,
                    outputs_list.get(idx).unwrap().get_value().to_owned(),
                ),
            ])
            .await?;
        }
    }

    Ok(())
}
