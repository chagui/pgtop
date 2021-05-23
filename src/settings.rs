use std::env;

use config::{Config, ConfigError, Environment};

/// Program settings representation.
/// https://www.postgresql.org/docs/9.1/libpq-envars.html
#[derive(Debug, Deserialize)]
pub(crate) struct ConnectionSettings {
    pub(crate) pghost: Option<String>,
    pub(crate) pghostaddr: Option<String>,
    pub(crate) pgport: Option<String>,
    pub(crate) pgdatabase: Option<String>,
    pub(crate) pguser: Option<String>,
    pub(crate) pgpassword: Option<String>,
}

impl ConnectionSettings {
    /// Returns the runtime settings of the program inferred from environment and config files.
    pub fn new() -> Result<Self, ConfigError> {
        let mut settings = Config::default();
        // if available use the operating system name of the user running the application.
        if let Ok(user) = env::var("USER") {
            let _ = settings.set_default("pguser", user.clone());
        }
        settings.merge(Environment::default())?;
        settings.try_into()
    }

    /// Generates a Key-Value libpq-style connection string.
    pub fn get_kv_connection_string(&self) -> String {
        let mut kv_connection_string = String::new();
        if let Some(host) = &self.pghost {
            kv_connection_string = format!("host={} {}", host, kv_connection_string);
        } else if let Some(hostaddr) = &self.pghostaddr {
            kv_connection_string = format!("host={} {}", hostaddr, kv_connection_string);
        }

        if let Some(port) = &self.pgport {
            kv_connection_string = format!("port={} {}", port, kv_connection_string);
        }
        if let Some(database) = &self.pgdatabase {
            kv_connection_string = format!("dbname={} {}", database, kv_connection_string);
        }
        if let Some(user) = &self.pguser {
            kv_connection_string = format!("user={} {}", user, kv_connection_string);
        }
        if let Some(password) = &self.pgpassword {
            kv_connection_string = format!("password={} {}", password, kv_connection_string);
        }
        println!("{}", kv_connection_string);
        kv_connection_string
    }
}
