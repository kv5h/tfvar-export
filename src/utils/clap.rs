use clap::{crate_description, crate_name, crate_version, Arg, ArgAction, Command};

pub fn new_clap_command() -> clap::ArgMatches {
    Command::new(crate_name!())
        .about(crate_description!())
        .version(crate_version!())
        .arg(
            Arg::new("base_url")
                .short('b')
                .long("base-url")
                .default_value("https://app.terraform.io")
                .require_equals(false)
                .required(false)
                .value_name("BASE_URL")
                .help("Base URL of Terraform API"),
        )
        .arg(
            Arg::new("target_workspaces")
                .short('t')
                .long("target-workspaces")
                .require_equals(false)
                .required(false)
                .required_unless_present("show_workspaces")
                .value_name("WORKSPACE_NAME1,WORKSPACE_NAME2,...")
                .help(
                    "Comma separated Terraform Cloud workspace names.\nRequired unless \
                     `--show-workspaces` is set.",
                ),
        )
        .arg(
            Arg::new("enable_info_log")
                .short('l')
                .long("info-log")
                .action(ArgAction::SetTrue)
                .help(
                    "Enable `Info` log.\nNote that `Error` log is always enabled regardless of \
                     this flag.",
                ),
        )
        .arg(
            Arg::new("show_workspaces")
                .short('w')
                .conflicts_with_all([
                    "export_list",
                    "target_workspaces",
                    "allow_update",
                    "output_values_file",
                    "export_list",
                ])
                .long("show-workspaces")
                .action(ArgAction::SetTrue)
                .help("Show available workspaces."),
        )
        .arg(
            Arg::new("allow_update")
                .short('u')
                .long("allow-update")
                .action(ArgAction::SetTrue)
                .help("Allow update of existing values."),
        )
        .arg(
            Arg::new("output_values_file")
                .index(1)
                .required(false)
                .required_unless_present("show_workspaces")
                .value_name("PATH_TO_OUTPUT_VALUES_FILE")
                .help(
                    "Path to the output values file generated with `terraform output \
                     --json`.\nRequired unless `--show-workspaces` is set.",
                ),
        )
        .arg(
            Arg::new("export_list")
                .index(2)
                .required(false)
                .required_unless_present("show_workspaces")
                .value_name("PATH_TO_EXPORT_LIST")
                .help("Path to the export list.\nRequired unless `--show-workspaces` is set."),
        )
        .get_matches()
}
