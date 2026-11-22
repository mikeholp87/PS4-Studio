//! SQLite persistence: settings, console profiles, audit log. Secrets are NEVER stored here.

use crate::model::{ConsoleModel, ConsoleProfile, Firmware, FirmwareSource};
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, thiserror::Error)]
pub enum DbError {
    #[error("database error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("could not create data directory: {0}")]
    Io(#[from] std::io::Error),
    #[error("invalid stored value: {0}")]
    Corrupt(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuditEntry {
    pub id: i64,
    pub ts: i64,
    pub action: String,
    pub target: String,
    pub detail: String,
    pub outcome: String,
}

pub struct Db {
    conn: Connection,
}

const SCHEMA: &str = "
CREATE TABLE IF NOT EXISTS settings (key TEXT PRIMARY KEY, value TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS profiles (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  name TEXT NOT NULL,
  model TEXT NOT NULL,
  address TEXT,
  fw_value TEXT,
  fw_source TEXT NOT NULL DEFAULT 'unknown',
  simulated INTEGER NOT NULL DEFAULT 0,
  last_connected INTEGER
);
CREATE TABLE IF NOT EXISTS audit_log (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  ts INTEGER NOT NULL,
  action TEXT NOT NULL,
  target TEXT NOT NULL,
  detail TEXT NOT NULL,
  outcome TEXT NOT NULL
);
PRAGMA user_version = 1;
";

pub fn now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

fn enum_str<T: Serialize>(v: &T) -> String {
    serde_json::to_value(v)
        .ok()
        .and_then(|v| v.as_str().map(str::to_owned))
        .unwrap_or_default()
}
fn enum_parse<T: for<'de> Deserialize<'de>>(s: &str) -> Result<T, DbError> {
    serde_json::from_value(serde_json::Value::String(s.to_owned()))
        .map_err(|_| DbError::Corrupt(s.to_owned()))
}

impl Db {
    pub fn open(path: &Path) -> Result<Self, DbError> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        Self::init(Connection::open(path)?)
    }

    pub fn open_memory() -> Result<Self, DbError> {
        Self::init(Connection::open_in_memory()?)
    }

    fn init(conn: Connection) -> Result<Self, DbError> {
        conn.execute_batch(SCHEMA)?;
        Ok(Self { conn })
    }

    pub fn get_setting(&self, key: &str) -> Result<Option<String>, DbError> {
        Ok(self
            .conn
            .query_row("SELECT value FROM settings WHERE key=?1", [key], |r| {
                r.get(0)
            })
            .optional()?)
    }

    pub fn set_setting(&self, key: &str, value: &str) -> Result<(), DbError> {
        self.conn.execute(
            "INSERT INTO settings(key,value) VALUES(?1,?2) ON CONFLICT(key) DO UPDATE SET value=excluded.value",
            [key, value],
        )?;
        Ok(())
    }

    pub fn add_profile(
        &self,
        name: &str,
        model: ConsoleModel,
        address: Option<&str>,
        firmware: &Firmware,
        simulated: bool,
    ) -> Result<ConsoleProfile, DbError> {
        self.conn.execute(
            "INSERT INTO profiles(name,model,address,fw_value,fw_source,simulated) VALUES(?1,?2,?3,?4,?5,?6)",
            params![name.trim(), enum_str(&model), address, firmware.value, enum_str(&firmware.source), simulated],
        )?;
        self.get_profile(self.conn.last_insert_rowid())?
            .ok_or_else(|| DbError::Corrupt("profile vanished".into()))
    }

    pub fn get_profile(&self, id: i64) -> Result<Option<ConsoleProfile>, DbError> {
        let row = self
            .conn
            .query_row(
                "SELECT id,name,model,address,fw_value,fw_source,simulated,last_connected FROM profiles WHERE id=?1",
                [id],
                Self::row,
            )
            .optional()?;
        row.map(Self::build).transpose()
    }

    pub fn list_profiles(&self) -> Result<Vec<ConsoleProfile>, DbError> {
        let mut st = self.conn.prepare(
            "SELECT id,name,model,address,fw_value,fw_source,simulated,last_connected FROM profiles ORDER BY id",
        )?;
        let rows = st
            .query_map([], Self::row)?
            .collect::<Result<Vec<_>, _>>()?;
        rows.into_iter().map(Self::build).collect()
    }

    pub fn delete_profile(&self, id: i64) -> Result<bool, DbError> {
        Ok(self
            .conn
            .execute("DELETE FROM profiles WHERE id=?1", [id])?
            > 0)
    }

    pub fn touch_connected(&self, id: i64) -> Result<(), DbError> {
        self.conn.execute(
            "UPDATE profiles SET last_connected=?1 WHERE id=?2",
            params![now(), id],
        )?;
        Ok(())
    }

    pub fn audit(
        &self,
        action: &str,
        target: &str,
        detail: &str,
        outcome: &str,
    ) -> Result<(), DbError> {
        self.conn.execute(
            "INSERT INTO audit_log(ts,action,target,detail,outcome) VALUES(?1,?2,?3,?4,?5)",
            params![now(), action, target, detail, outcome],
        )?;
        Ok(())
    }

    pub fn list_audit(&self, limit: u32) -> Result<Vec<AuditEntry>, DbError> {
        let mut st = self.conn.prepare(
            "SELECT id,ts,action,target,detail,outcome FROM audit_log ORDER BY id DESC LIMIT ?1",
        )?;
        let rows = st
            .query_map([limit], |r| {
                Ok(AuditEntry {
                    id: r.get(0)?,
                    ts: r.get(1)?,
                    action: r.get(2)?,
                    target: r.get(3)?,
                    detail: r.get(4)?,
                    outcome: r.get(5)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    #[allow(clippy::type_complexity)]
    fn row(
        r: &rusqlite::Row,
    ) -> rusqlite::Result<(
        i64,
        String,
        String,
        Option<String>,
        Option<String>,
        String,
        bool,
        Option<i64>,
    )> {
        Ok((
            r.get(0)?,
            r.get(1)?,
            r.get(2)?,
            r.get(3)?,
            r.get(4)?,
            r.get(5)?,
            r.get(6)?,
            r.get(7)?,
        ))
    }

    #[allow(clippy::type_complexity)]
    fn build(
        t: (
            i64,
            String,
            String,
            Option<String>,
            Option<String>,
            String,
            bool,
            Option<i64>,
        ),
    ) -> Result<ConsoleProfile, DbError> {
        Ok(ConsoleProfile {
            id: t.0,
            name: t.1,
            model: enum_parse::<ConsoleModel>(&t.2)?,
            address: t.3,
            firmware: Firmware {
                value: t.4,
                source: enum_parse::<FirmwareSource>(&t.5)?,
            },
            simulated: t.6,
            last_connected: t.7,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settings_roundtrip_and_overwrite() {
        let db = Db::open_memory().unwrap();
        assert_eq!(db.get_setting("theme").unwrap(), None);
        db.set_setting("theme", "dark").unwrap();
        db.set_setting("theme", "light").unwrap();
        assert_eq!(db.get_setting("theme").unwrap().as_deref(), Some("light"));
    }

    #[test]
    fn profiles_crud_and_enums() {
        let db = Db::open_memory().unwrap();
        let fw = Firmware {
            value: Some("9.00".into()),
            source: FirmwareSource::UserEntered,
        };
        let p = db
            .add_profile(
                "  Living room ",
                ConsoleModel::Slim,
                Some("192.168.1.50"),
                &fw,
                false,
            )
            .unwrap();
        assert_eq!(p.name, "Living room");
        assert_eq!(p.model, ConsoleModel::Slim);
        assert_eq!(p.firmware.source, FirmwareSource::UserEntered);
        db.touch_connected(p.id).unwrap();
        assert!(db
            .get_profile(p.id)
            .unwrap()
            .unwrap()
            .last_connected
            .is_some());
        assert_eq!(db.list_profiles().unwrap().len(), 1);
        assert!(db.delete_profile(p.id).unwrap());
        assert!(!db.delete_profile(p.id).unwrap());
    }

    #[test]
    fn audit_is_newest_first_and_persisted_on_disk() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("sub/app.db");
        {
            let db = Db::open(&path).unwrap();
            db.audit("a", "t", "d", "ok").unwrap();
            db.audit("b", "t", "d", "ok").unwrap();
        }
        let db = Db::open(&path).unwrap();
        let log = db.list_audit(10).unwrap();
        assert_eq!(
            log.iter().map(|e| e.action.as_str()).collect::<Vec<_>>(),
            ["b", "a"]
        );
    }
}
