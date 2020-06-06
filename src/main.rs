mod database;
mod net;
mod utils;
mod style;
mod constants;
mod crumble;

use clap::{App, Arg, SubCommand, AppSettings};

#[tokio::main]
async fn main() {
    if !std::env::var("BREAD_VERBOSITY").is_ok() {
        std::env::set_var("BREAD_VERBOSITY", "WARN,bread=INFO");
    }

    let app = App::new("bread")
        .version("1.0.0")
        .author("Robin A. P. <me@mempler.de>")
        .about("a crumbly package manager")
        .setting(AppSettings::ColoredHelp)
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .arg(Arg::with_name("verbose")
            .short("v")
            .help("Sets the level of verbosity")
            .possible_value("trace").possible_value("debug").possible_value("info").possible_value("warn").possible_value("error")
            .default_value("info"))
        .subcommand(SubCommand::with_name("update")
            .setting(AppSettings::ColoredHelp)
            .about("Updates the crumble cache database"));

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
                    let db = database::Database::from_mirror("https://mirror.mempler.de".to_string(), "leopard").await;

                    println!("{:#?}", db);
                }

                _ => println!("{}", matches.usage())
            }
        }

        None => println!("{}", matches.usage())
    }
}
