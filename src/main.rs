#[macro_use]
extern crate serde_derive;

use std::env;
use std::sync::mpsc::Receiver;

use clap::{App, Arg, ArgMatches};
use tokio;
use tokio_postgres::{Client, NoTls};

use banner::BANNER;

mod banner;
mod db;
mod error;
mod event;
mod settings;
mod ui;

/// A `Result` alias where the `Err` case is `CliError`.
pub type CliResult<T> = std::result::Result<T, error::CliError>;

pub struct Context {
    client: Client,
    events: event::Events,
}

fn parse_args() -> ArgMatches<'static> {
    let user = env::var("USER").expect("expected variable USER not set");
    let parser = App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .before_help(BANNER);

    parser
        .arg(
            Arg::with_name("config_file")
                .short("c")
                .long("config")
                .takes_value(true)
                .value_name("FILE")
                .help(r#"Use custom config file (default: "~/.config/jw-cli/config.yaml")"#),
        )
        .arg(
            Arg::with_name("host")
                .short("h")
                .long("host")
                .takes_value(true)
                .help(r#"Database server host or socket directory (default: "local socket")"#),
        )
        .arg(
            Arg::with_name("port")
                .short("p")
                .long("port")
                .default_value("5432")
                .help(r#"Database server port (default: "5432")"#),
        )
        .arg(
            Arg::with_name("dbname")
                .short("d")
                .long("dbname")
                .default_value("5432")
                .help(&format!(r#"database name to connect to (default: "{}")"#, user)),
        )
        .arg(
            Arg::with_name("user")
                .short("u")
                .long("username")
                .takes_value(true)
                .help(&format!(r#"Database user name (default: "{}")"#, user)),
        )
        .arg(
            Arg::with_name("disable_password")
                .short("w")
                .long("no-password")
                .help("Never prompt for password"),
        )
        .arg(
            Arg::with_name("force_password")
                .short("W")
                .long("password")
                .help("Force password prompt (should happen automatically)"),
        )
        .get_matches()
}

#[tokio::main] // By default, tokio_postgres uses the tokio crate as its runtime.
async fn main() -> CliResult<()> {
    let args = parse_args();
    let mut settings = settings::ConnectionSettings::new().unwrap_or_else(|err| {
        println!("configuration error: {:}", err);
        std::process::exit(exitcode::CONFIG);
    });
    // cli args have precedence over env config
    if let Some(host) = args.value_of("host") {
        settings.pghost = Some(String::from(host));
    }
    if let Some(port) = args.value_of("port") {
        settings.pgport = Some(String::from(port));
    }
    if let Some(dbname) = args.value_of("dbname") {
        settings.pgdatabase = Some(String::from(dbname));
    }
    if let Some(user) = args.value_of("user") {
        settings.pguser = Some(String::from(user));
    }

    // Connect to the database.
    let (client, connection) =
        tokio_postgres::connect(&settings.get_kv_connection_string(), NoTls).await?;

    // The connection object performs the actual communication with the database,
    // so spawn it off to run on its own.
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    let events = event::Events::new();
    let ctx = Context { client, events };
    ui::start_ui(ctx).await
}
