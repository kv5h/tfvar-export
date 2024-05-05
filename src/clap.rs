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
            Arg::new("enable_info_log")
                .short('l')
                .long("info-log")
                .action(ArgAction::SetTrue)
                .help("Enable `Info` log"),
        )
        .arg(
            Arg::new("show_outputs")
                .short('o')
                .long("show-outputs")
                .action(ArgAction::SetTrue)
                .help("Show a list of outputs and exit"),
        )
        .arg(
            Arg::new("show_workspaces")
                .short('w')
                .long("show-workspaces")
                .action(ArgAction::SetTrue)
                .help("Show workspaces"),
        )
        .arg(
            Arg::new("allow_update")
                .short('u')
                .long("allow-update")
                .action(ArgAction::SetTrue)
                .help("Allow update of the value when a variable already exists"),
        )
        .arg(
            Arg::new("tfstate_file")
                .index(1)
                .required(true)
                .value_name("PATH_TO_TFSTATE_FILE")
                .help("Path to tfstate file"),
        )
        .arg(
            Arg::new("export_list")
                .index(2)
                .required(true)
                .value_name("PATH_TO_EXPORT_LIST")
                .help("Path to export list"),
        )
        .get_matches()
}
