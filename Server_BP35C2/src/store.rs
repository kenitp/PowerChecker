use std::path::Path;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use rusqlite::{params, Connection};

const MIN_STEP_SEC: i64 = 60;
const MAX_DURATION_SEC: i64 = 90 * 24 * 3600;
const MAX_POINTS: i64 = 2500;

#[derive(Debug, Clone, PartialEq)]
pub struct Sample {
    pub ts: i64,
    pub power_w: u32,
    pub power_a: Option<f64>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct HistoryRange {
    pub from: i64,
    pub to: i64,
    pub step: i64,
}

pub struct Store {
    conn: Mutex<Connection>,
}

impl Store {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, rusqlite::Error> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
            }
        }
        let conn = Connection::open(path)?;
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "synchronous", "NORMAL")?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS power_samples (
                ts INTEGER PRIMARY KEY,
                power_w INTEGER NOT NULL,
                power_a REAL
            );",
        )?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    pub fn insert(&self, power_w: u32, power_a: Option<f64>) -> Result<(), rusqlite::Error> {
        self.insert_at(now_secs(), power_w, power_a)
    }

    pub fn insert_at(
        &self,
        ts: i64,
        power_w: u32,
        power_a: Option<f64>,
    ) -> Result<(), rusqlite::Error> {
        let conn = self.conn.lock().expect("store mutex poisoned");
        conn.execute(
            "INSERT OR REPLACE INTO power_samples (ts, power_w, power_a) VALUES (?1, ?2, ?3)",
            params![ts, power_w as i64, power_a],
        )?;
        Ok(())
    }

    pub fn query(&self, range: HistoryRange) -> Result<Vec<Sample>, rusqlite::Error> {
        let conn = self.conn.lock().expect("store mutex poisoned");
        let mut stmt = conn.prepare(
            "SELECT (ts / ?1) * ?1 AS bucket,
                    CAST(AVG(power_w) AS INTEGER) AS power_w,
                    AVG(power_a) AS power_a
             FROM power_samples
             WHERE ts >= ?2 AND ts <= ?3
             GROUP BY bucket
             ORDER BY bucket",
        )?;
        let rows = stmt.query_map(params![range.step, range.from, range.to], |row| {
            Ok(Sample {
                ts: row.get(0)?,
                power_w: row.get::<_, i64>(1)? as u32,
                power_a: row
                    .get::<_, Option<f64>>(2)?
                    .map(|v| (v * 10.0).round() / 10.0),
            })
        })?;
        rows.collect()
    }
}

pub fn now_secs() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock before unix epoch")
        .as_secs() as i64
}

pub fn resolve_range(
    from: Option<i64>,
    to: Option<i64>,
    step: Option<i64>,
) -> Result<HistoryRange, String> {
    let to = to.unwrap_or_else(now_secs);
    let from = from.unwrap_or(to - 24 * 3600);

    if from >= to {
        return Err("from must be less than to".to_string());
    }

    let duration = to - from;
    if duration > MAX_DURATION_SEC {
        return Err("range exceeds 90 days".to_string());
    }

    let mut step = step.unwrap_or_else(|| auto_step(duration));
    if step < MIN_STEP_SEC {
        return Err("step must be at least 60 seconds".to_string());
    }

    let points = duration / step;
    if points > MAX_POINTS {
        step = ((duration + MAX_POINTS - 1) / MAX_POINTS).max(MIN_STEP_SEC);
    }

    Ok(HistoryRange { from, to, step })
}

fn auto_step(duration: i64) -> i64 {
    if duration <= 24 * 3600 {
        60
    } else if duration <= 7 * 24 * 3600 {
        300
    } else {
        3600
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_range_is_24h_at_1min() {
        let range = resolve_range(None, Some(1_000_000), None).unwrap();
        assert_eq!(range.to, 1_000_000);
        assert_eq!(range.from, 1_000_000 - 24 * 3600);
        assert_eq!(range.step, 60);
    }

    #[test]
    fn week_range_uses_5min_step() {
        let to = 2_000_000;
        let from = to - 7 * 24 * 3600;
        let range = resolve_range(Some(from), Some(to), None).unwrap();
        assert_eq!(range.step, 300);
    }

    #[test]
    fn rejects_inverted_range() {
        assert!(resolve_range(Some(100), Some(50), None).is_err());
    }

    #[test]
    fn insert_and_aggregate() {
        let store = Store::open(":memory:").unwrap();
        store.insert_at(1000, 100, Some(1.0)).unwrap();
        store.insert_at(1060, 200, Some(2.0)).unwrap();
        store.insert_at(1120, 300, Some(3.0)).unwrap();

        let points = store
            .query(HistoryRange {
                from: 1000,
                to: 1200,
                step: 120,
            })
            .unwrap();

        assert_eq!(points.len(), 2);
        assert_eq!(points[0].ts, 960);
        assert_eq!(points[0].power_w, 150);
        assert_eq!(points[0].power_a, Some(1.5));
        assert_eq!(points[1].ts, 1080);
        assert_eq!(points[1].power_w, 300);
        assert_eq!(points[1].power_a, Some(3.0));
    }
}
