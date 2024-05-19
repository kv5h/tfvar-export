mod terraform_api;
mod utils;

use std::collections::HashMap;

use log::warn;

use crate::{
    terraform_api::{
        check_variable_status::check_variable_status,
        connection_prop::TerraformApiConnectionProperty,
        get_workspaces::get_workspaces,
        register_variable::{create_variable, update_variable, TerraformVariableProperty},
    },
    utils::construct_export_value::construct_export_value,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Clap: Read command-line options
    let clap = utils::clap::new_clap_command();
    let base_url = clap.get_one::<String>("base_url").unwrap();
    let target_workspaces = clap.try_get_one::<String>("target_workspaces").unwrap();
    let disable_log = clap.get_flag("disable_log");
    let show_workspaces = clap.get_flag("show_workspaces");
    let allow_update = clap.get_flag("allow_update");
    let output_values_file = clap.try_get_one::<String>("output_values_file").unwrap();
    let export_list = clap.try_get_one::<String>("export_list").unwrap();

    // Log
    let mut builder = env_logger::Builder::new();
    match disable_log {
        true => builder.filter_level(log::LevelFilter::Error),
        false => builder.filter_level(log::LevelFilter::Info),
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

    // Workspace(s)
    let workspace_name_id: HashMap<String, String> =
        get_workspaces(false, &organization_name, &api_conn_prop)
            .await?
            .into_iter()
            .map(|val| {
                (
                    val.get_workspace_name().to_string(),
                    val.get_workspace_id().to_string(),
                )
            })
            .collect();
    let workspace_names: Vec<String> = target_workspaces
        .unwrap()
        .split(',')
        .map(|val| val.to_string())
        .collect();
    let workspace_ids: Vec<String> = workspace_names
        .into_iter()
        .map(|val| workspace_name_id.get(&val).unwrap().to_owned())
        .collect();

    // Variable name and its value
    let var_name_val = construct_export_value(export_list.unwrap(), output_values_file.unwrap())?;
    let var_name_val_des_map: HashMap<String, (Option<String>, serde_json::Value)> = var_name_val
        .iter()
        .map(|val| {
            (
                val.get_variable_name().to_owned(),
                (
                    val.get_variable_description().to_owned(),
                    val.get_value().to_owned(),
                ),
            )
        })
        .collect();
    let target_variables = var_name_val
        .iter()
        .map(|val| val.get_variable_name().to_owned())
        .collect();

    // Loop over workspace(s)
    for workspace_id in workspace_ids {
        // Variable status; existing or not
        let status =
            check_variable_status(&workspace_id, &api_conn_prop, &target_variables).await?;
        // Variable(s) to be created
        let vars_new: Vec<TerraformVariableProperty> = status
            .iter()
            .filter(|val| val.get_variable_id().is_none())
            .map(|val| {
                TerraformVariableProperty::new(
                    None,
                    val.get_variable_name().to_owned(),
                    var_name_val_des_map
                        .get(val.get_variable_name())
                        .unwrap()
                        .0
                        .to_owned(),
                    var_name_val_des_map
                        .get(val.get_variable_name())
                        .unwrap()
                        .1
                        .to_owned(),
                )
            })
            .collect();
        if 0 < vars_new.len() {
            let create_variable_result =
                create_variable(&workspace_id, &api_conn_prop, &vars_new).await?;
            println!("Variable(s) created: {:#?}", create_variable_result);
        }

        if allow_update {
            // Variable(s) already existing
            let vars_existing: Vec<TerraformVariableProperty> = status
                .iter()
                .filter(|val| val.get_variable_id().is_some())
                .map(|val| {
                    TerraformVariableProperty::new(
                        Some(val.get_variable_id().clone().unwrap()),
                        val.get_variable_name().to_owned(),
                        var_name_val_des_map
                            .get(val.get_variable_name())
                            .unwrap()
                            .0
                            .to_owned(),
                        var_name_val_des_map
                            .get(val.get_variable_name())
                            .unwrap()
                            .1
                            .to_owned(),
                    )
                })
                .collect();

            if 0 < vars_existing.len() {
                let update_variable_result =
                    update_variable(&workspace_id, &api_conn_prop, &vars_existing).await?;
                println!("Variable(s) updated: {:#?}", update_variable_result);
            }
        } else {
            // Variable(s) already existing
            let vars_existing: Vec<&str> = status
                .iter()
                .filter(|val| val.get_variable_id().is_some())
                .map(|val| val.get_variable_name())
                .collect();
            if 0 < vars_existing.len() {
                warn!(
                    "Following variable(s) were ignored because they are existing but \
                     `--allow_update` is not specified: {:#?}",
                    vars_existing
                );
            }
        }
    }

    Ok(())
}
