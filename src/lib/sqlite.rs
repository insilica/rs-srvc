use std::collections::HashMap;
use std::path::Path;

use log::trace;
use rusqlite::{params, CachedStatement, Connection, OpenFlags, Row};
use serde_json::Value;

use crate::errors::*;
use crate::event::Event;

fn load_schema(conn: &Connection) -> Result<()> {
    conn.execute_batch(include_str!("schema/sqlite.sql"))
        .chain_err(|| "Failed to load sqlite schema")
}

pub fn prepare_cached<'a>(conn: &'a Connection, stmt: &str) -> Result<CachedStatement<'a>> {
    conn.prepare_cached(stmt)
        .chain_err(|| "Failed to prepare SQL statement")
}

pub fn trigger_exists(conn: &Connection, trigger_name: &str) -> Result<bool> {
    let mut stmt = conn
        .prepare("SELECT sql FROM sqlite_master WHERE type='trigger' AND name=?1")
        .chain_err(|| "Failed to execute statement")?;
    let mut rows = stmt
        .query(params![trigger_name])
        .chain_err(|| "Failed to execute query")?;

    match rows.next().chain_err(|| "Failed to retrieve rows")? {
        Some(_) => Ok(true),
        None => Ok(false),
    }
}

/// Return an error if the DB format is too old
pub fn check_db_format(conn: &Connection) -> Result<()> {
    if trigger_exists(conn, &"srvc_event_label_answer_document_constraint")? {
        Err(Error::from(ErrorKind::SQLiteError(
            "SQLite SRVC sink is too old and must be upgraded to work with this version of SRVC."
                .into(),
        )))
    } else {
        Ok(())
    }
}

/// Open a read/write connection to a sqlite file. Creates or updates
/// the schema as needed.
pub fn open(path: &Path) -> Result<Connection> {
    let conn = Connection::open(path)
        .chain_err(|| format!("Failed to open sqlite file: {}", path.to_string_lossy()))?;
    load_schema(&conn)?;
    check_db_format(&conn)?;
    Ok(conn)
}

/// Open a read-only connection to a sqlite file.
pub fn open_ro(path: &Path) -> Result<Connection> {
    // Check for file existence in order to give a more clear error message.
    // We don't care about the TOCTOU race condition in this case.
    match path.try_exists() {
        Ok(true) => {
            let conn = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY)
                .chain_err(|| format!("Failed to open sqlite file: {}", path.to_string_lossy()))?;
            check_db_format(&conn)?;
            Ok(conn)
        }
        _ => Err(format!("File does not exist: {}", path.to_string_lossy()).into()),
    }
}

/// Close the sqlite connection, with a clear error message in case
/// of failure.
pub fn close(conn: Connection) -> Result<()> {
    match conn.close() {
        Ok(_) => Ok(()),
        Err((conn, e)) => Err(e).chain_err(|| {
            format!(
                "Failed to close sqlite connection to: {}",
                conn.path()
                    .map(|v| v.to_string_lossy())
                    .unwrap_or("".into())
            )
        }),
    }
}

pub fn insert_event(conn: &Connection, event: Event) -> Result<usize> {
    let hash = event.hash.clone().expect("Hash not set");
    trace! {"Inserting event: {}", hash};
    let data = event
        .data
        .map(|v| serde_json::to_string(&v))
        .transpose()
        .chain_err(|| format!("Failed to serialize data property of event: {}", hash))?;
    let extra = if event.extra.is_empty() {
        String::from("{}")
    } else {
        let v = serde_json::to_value(event.extra).chain_err(|| {
            format!(
                "Failed to convert HashMap to serde_json::Value for event: {}",
                hash
            )
        })?;
        serde_json::to_string(&v)
            .chain_err(|| format!("Failed to serialize extra properties for event: {}", hash))?
    };
    match conn.execute(
        "INSERT INTO srvc_event (hash, data, extra, type, uri) VALUES (?, ?, ?, ?, ?) ON CONFLICT (hash) DO NOTHING",
        [event.hash, data, Some(extra), Some(event.r#type), event.uri],
    ) {
        Ok(rows) => {
            trace!("Modified {} rows", rows);
            Ok(rows)
        }
        Err(e) => Err(e).chain_err(|| format!("Error inserting event: {}", hash)),
    }
}

fn value_to_map(value: &Value) -> Option<HashMap<String, Value>> {
    if let Some(map) = value.as_object() {
        let mut new_map = HashMap::new();
        for (k, v) in map.iter() {
            new_map.insert(k.clone(), v.clone());
        }
        Some(new_map)
    } else {
        None
    }
}

pub fn parse_event_rusqlite(row: &Row) -> rusqlite::Result<Event> {
    let extra_json: Option<Value> = row.get(1)?;
    let extra = extra_json
        .and_then(|v| value_to_map(&v))
        .unwrap_or_else(HashMap::new);
    Ok(Event {
        data: row.get(0)?,
        extra,
        hash: row.get(2)?,
        r#type: row.get(3)?,
        uri: row.get(4)?,
    })
}

pub fn parse_event(row: &Row) -> Result<Event> {
    parse_event_rusqlite(row).chain_err(|| "Failed to parse event row data")
}
