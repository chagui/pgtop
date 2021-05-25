#[macro_use]
extern crate serde_derive;

use tokio_postgres::{Client, NoTls};

use cli::parse_args;

mod banner;
mod cli;
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

#[tokio::main] // By default, tokio_postgres uses the tokio crate as its runtime.
async fn main() -> CliResult<()> {
    let args = parse_args();
    let mut settings = settings::ConnectionSettings::new().unwrap_or_else(|err| {
        eprintln!("configuration error: {:}", err);
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
