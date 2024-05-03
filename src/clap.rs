use clap::{Arg, Command};

pub fn new_clap_command() -> clap::ArgMatches {
	Command::new("tfvar-export")
		.arg(
			Arg::new("base_url")
				.short('u')
				.long("base-url")
				.default_value("https://app.terraform.io/api/v2")
				.require_equals(false)
				.help("Base URL of Terraform API"),
		)
		.get_matches()
}
