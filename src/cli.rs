use std::env;

use clap::{App, Arg, ArgMatches};

use crate::banner::BANNER;

pub fn parse_args() -> ArgMatches<'static> {
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
                .help(&format!(
                    r#"database name to connect to (default: "{}")"#,
                    user
                )),
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
