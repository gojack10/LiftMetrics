use rusqlite::{Connection, Result};
use std::path::Path;

pub fn init(db_path: &str) -> Result<()> {
    let conn = Connection::open(Path::new(db_path))?;

    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS diet_cycles (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            phase TEXT NOT NULL,
            start_date TEXT NOT NULL,
            planned_end_date TEXT NOT NULL,
            actual_end_date TEXT,
            notes TEXT,
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        );

        CREATE TABLE IF NOT EXISTS weight_logs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            diet_cycle_id INTEGER,
            log_date TEXT NOT NULL,
            weight_lbs REAL NOT NULL,
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        );

        CREATE TABLE IF NOT EXISTS exercises (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL UNIQUE COLLATE NOCASE,
            description TEXT,
            default_metric_to_track TEXT,
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        );

        CREATE TABLE IF NOT EXISTS workout_sessions (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            session_date TEXT NOT NULL,
            notes TEXT,
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        );

        CREATE TABLE IF NOT EXISTS exercise_sets (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            workout_session_id INTEGER NOT NULL,
            exercise_id INTEGER NOT NULL,
            set_order INTEGER NOT NULL,
            reps INTEGER NOT NULL,
            weight_lbs REAL NOT NULL,
            rpe REAL,
            notes TEXT,
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        );
    ",
    )?;

    Ok(())
}
