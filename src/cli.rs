use clap::{App, Arg, ArgMatches};

use crate::banner::BANNER;

pub fn parse_args(user: &str) -> ArgMatches {
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
                .help(&format!(
                    r#"Use custom config file (default: "~/.config/{}/config.yaml")"#,
                    env!("CARGO_PKG_NAME")
                )),
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
                .env("PGPORT")
                .default_value("5432")
                .help(r#"Database server port (default: "5432")"#),
        )
        .arg(
            Arg::with_name("dbname")
                .short("d")
                .long("dbname")
                .default_value(user)
                .help(&format!(
                    r#"Database name to connect to (default: "{}")"#,
                    user
                )),
        )
        .arg(
            Arg::with_name("user")
                .short("u")
                .long("username")
                .default_value(user)
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
