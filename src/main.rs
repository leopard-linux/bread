mod database;
mod net;
mod utils;
mod style;
mod constants;
mod crumble;

use clap::{App, Arg, SubCommand, AppSettings};
use crate::constants::PATH_CONFIGS;

#[tokio::main]
async fn main() {
    if !std::env::var("BREAD_VERBOSITY").is_ok() {
        std::env::set_var("BREAD_VERBOSITY", "WARN,bread=INFO");
    }

    let app =
        App::new("bread")
            .version("1.0.0")
            .author("Robin A. P. <me@mempler.de>")
            .about("a crumbly package manager")
            .setting(AppSettings::ColoredHelp)
            .setting(AppSettings::SubcommandRequiredElseHelp)

            .arg(Arg::with_name("verbose")
                .short("v")
                .help("Sets the level of verbosity")
                .possible_value("trace")
                .possible_value("debug")
                .possible_value("info")
                .possible_value("warn")
                .possible_value("error")
                .default_value("info"))

            .subcommand(SubCommand::with_name("kitchen")
                .setting(AppSettings::ColoredHelp)
                .setting(AppSettings::SubcommandRequiredElseHelp)
                .about("Kitchen for setting up a custom mirror")

                .arg(Arg::with_name("output")
                    .short("o")
                    .help("Sets the output directory")
                    .default_value("."))

                .subcommand(SubCommand::with_name("cook")
                    .setting(AppSettings::ColoredHelp)
                    .about("Cook up a mirror for production use"))

                .subcommand(SubCommand::with_name("update")
                    .setting(AppSettings::ColoredHelp)
                    .about("Update all outdated package(s) in this mirror"))

                .subcommand(SubCommand::with_name("bake")
                    .setting(AppSettings::ColoredHelp)
                    .about("Bakes a custom mirror for custom package(s)"))
            )

            .subcommand(SubCommand::with_name("strip")
                .setting(AppSettings::ColoredHelp)
                .setting(AppSettings::SubcommandRequiredElseHelp)
                .about("Installs packages in a folder, useful for making a linux distribution\n(E.G: `bread strip linux linux-fs coreutils bread grub2 -o ./leopard`) for a basic distribution"))

            .subcommand(SubCommand::with_name("bake")
                .setting(AppSettings::ColoredHelp)
                .setting(AppSettings::SubcommandRequiredElseHelp)
                .about("Bakes a package into an .crumb file"))

            .subcommand(SubCommand::with_name("install")
                .setting(AppSettings::ColoredHelp)
                .setting(AppSettings::SubcommandRequiredElseHelp)
                .about("Install a package from the mirrors declared in /etc/bread/mirror.toml and /etc/bread/mirrors.d/"))

            .subcommand(SubCommand::with_name("update")
                .setting(AppSettings::ColoredHelp)
                .about("Updates the crumble cache database"))

            .subcommand(SubCommand::with_name("upgrade")
                .setting(AppSettings::ColoredHelp)
                .about("Upgrade all packages")
                .arg(Arg::with_name("force")
                    .short("f")
                    .help("Force upgrading a package (overrides freeze status)")
                    .value_delimiter(" ")))
        ;

    let matches = app.get_matches();

    match matches.value_of("verbose") {
        Some(e) => std::env::set_var("BREAD_VERBOSITY", format!("WARN,bread={}", e)),

        None => if matches.is_present("verbose") {  std::env::set_var("BREAD_VERBOSITY", "WARN,bread=TRACE");  }
    }

    pretty_env_logger::init_custom_env("BREAD_VERBOSITY");

    match matches.subcommand_name() {
        Some(c) => {
            match c {
                "update" => {
                    // TODO: use multiple mirrors
                    let db = database::Database::from_mirror("https://mirror.mempler.de".to_string(), "leopard").await;

                    db.save_to_file(PATH_CONFIGS.to_string() + "/databases");
                }

                _ => println!("{}", matches.usage())
            }
        }

        None => println!("{}", matches.usage())
    }
}
