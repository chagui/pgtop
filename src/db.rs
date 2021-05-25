use tokio_postgres::{Client, Row};

use crate::CliResult;

#[derive(Debug)]
pub struct PGSystemInfo {
    pub(crate) version: String,
    pub(crate) uptime: String,
    pub(crate) nb_of_conn: i64,
}

/// See https://www.postgresql.org/docs/9.4/monitoring-stats.html#PG-STAT-ACTIVITY-VIEW
#[derive(Debug)]
pub struct PGStatActivity {
    pub(crate) datname: String,
    pub(crate) pid: i32,
    pub(crate) usename: String,
    pub(crate) client_addr: String,
    pub(crate) client_port: i32,
    pub(crate) xact_start: Option<String>,
    pub(crate) backend_duration: String,
    pub(crate) query_duration: String,
    pub(crate) state: String,
    pub(crate) query: String,
}

impl From<Row> for PGStatActivity {
    fn from(row: Row) -> PGStatActivity {
        PGStatActivity {
            datname: row.get("datname"),
            pid: row.get("pid"),
            usename: row.get("usename"),
            client_addr: row.get("client_addr"),
            client_port: row.get("client_port"),
            xact_start: row.get("xact_start"),
            backend_duration: row.get("backend_duration"),
            query_duration: row.get("query_duration"),
            state: row.get("state"),
            query: row.get("query"),
        }
    }
}

pub async fn get_activities(client: &Client) -> CliResult<Vec<PGStatActivity>> {
    // todo: decide on using diesel instead of raw (untyped) query
    let activities_query = r"
        SELECT datname,
        pid,
        usename,
        client_addr::text,
        client_port,
        xact_start::text,
        to_char(current_timestamp - backend_start, 'HH24:MI:SS:MS') AS backend_duration,
        to_char(current_timestamp - query_start, 'HH24:MI:SS:MS')::text AS query_duration,
        state,
        query
 FROM pg_stat_activity
 WHERE backend_type = 'client backend'
        ";
    let stats = client
        .query(activities_query, &[])
        .await?
        .into_iter()
        .map(|row| PGStatActivity::from(row))
        .collect::<Vec<_>>();
    Ok(stats)
}

pub async fn get_system_info(client: &Client) -> CliResult<PGSystemInfo> {
    // https://www.postgresql.org/docs/9.2/monitoring-stats.html#PG-STAT-DATABASE-VIEW
    let system_info_query = r"
  SELECT version(),
         justify_interval(current_timestamp - pg_postmaster_start_time())::text,
         sum(numbackends)
    FROM pg_stat_database";
    // retrieve version
    let row = client.query_one(system_info_query, &[]).await?;

    // And then check that we got back the same string we sent over.
    let version: String = row.get(0);
    // notes: https://github.com/sfackler/rust-postgres/issues/60
    let uptime: String = row.get(1);
    let nb_of_conn: i64 = row.get(2);

    Ok(PGSystemInfo {
        version,
        uptime,
        nb_of_conn,
    })
}
