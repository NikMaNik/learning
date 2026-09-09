use anyhow::Result;
use chrono::Utc;
use rusqlite::{params, Connection};

use crate::models::Word;

pub fn init_db(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS words (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            english TEXT NOT NULL,
            russian TEXT NOT NULL,
            ease_factor REAL NOT NULL DEFAULT 2.5,
            interval INTEGER NOT NULL DEFAULT 0,
            repetitions INTEGER NOT NULL DEFAULT 0,
            next_review TEXT NOT NULL,
            created_at TEXT NOT NULL,
            last_quality INTEGER
        );",
    )?;
    migrate(conn)?;
    Ok(())
}

/// Аdds missing columns to an existing database (migration).
fn migrate(conn: &Connection) -> Result<()> {
    let has_last_quality: bool = conn
        .prepare("PRAGMA table_info(words)")?
        .query_map([], |row| row.get::<_, String>(1))?
        .collect::<std::result::Result<Vec<_>, _>>()?
        .iter()
        .any(|name| name == "last_quality");

    if !has_last_quality {
        conn.execute_batch("ALTER TABLE words ADD COLUMN last_quality INTEGER;")?;
    }
    Ok(())
}

pub fn add_word(conn: &Connection, english: &str, russian: &str) -> Result<Word> {
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO words (english, russian, ease_factor, interval, repetitions, next_review, created_at)
         VALUES (?1, ?2, 2.5, 0, 0, ?3, ?3)",
        params![english, russian, now],
    )?;
    let id = conn.last_insert_rowid();
    get_word(conn, id)
}

fn row_to_word(row: &rusqlite::Row) -> rusqlite::Result<Word> {
    Ok(Word {
        id: row.get(0)?,
        english: row.get(1)?,
        russian: row.get(2)?,
        ease_factor: row.get(3)?,
        interval: row.get(4)?,
        repetitions: row.get(5)?,
        next_review: row.get(6)?,
        created_at: row.get(7)?,
        last_quality: row.get(8)?,
    })
}

const WORD_COLUMNS: &str = "id, english, russian, ease_factor, interval, repetitions, next_review, created_at, last_quality";

pub fn get_all_words(conn: &Connection) -> Result<Vec<Word>> {
    let mut stmt = conn.prepare(&format!(
        "SELECT {WORD_COLUMNS} FROM words ORDER BY id"
    ))?;
    let words = stmt
        .query_map([], row_to_word)?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(words)
}

pub fn get_word(conn: &Connection, id: i64) -> Result<Word> {
    Ok(conn.query_row(
        &format!("SELECT {WORD_COLUMNS} FROM words WHERE id = ?1"),
        params![id],
        row_to_word,
    )?)
}

pub fn update_word(conn: &Connection, id: i64, english: &str, russian: &str) -> Result<Word> {
    conn.execute(
        "UPDATE words SET english = ?1, russian = ?2 WHERE id = ?3",
        params![english, russian, id],
    )?;
    get_word(conn, id)
}

pub fn delete_word(conn: &Connection, id: i64) -> Result<()> {
    conn.execute("DELETE FROM words WHERE id = ?1", params![id])?;
    Ok(())
}

pub fn get_words_for_review(conn: &Connection) -> Result<Vec<Word>> {
    let now = Utc::now().to_rfc3339();
    let mut stmt = conn.prepare(&format!(
        "SELECT {WORD_COLUMNS}
         FROM words WHERE next_review <= ?1 ORDER BY next_review"
    ))?;
    let words = stmt
        .query_map(params![now], row_to_word)?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(words)
}

/// Words that are due now or will become due within the next `days` days.
pub fn get_words_upcoming(conn: &Connection, days: i64) -> Result<Vec<Word>> {
    let now = Utc::now();
    let deadline = (now + chrono::Duration::days(days)).to_rfc3339();
    let mut stmt = conn.prepare(&format!(
        "SELECT {WORD_COLUMNS}
         FROM words WHERE next_review <= ?1 ORDER BY next_review"
    ))?;
    let words = stmt
        .query_map(params![deadline], row_to_word)?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(words)
}

/// Words that were answered with quality < 3 (i.e. not remembered) on their last review.
pub fn get_forgotten_words(conn: &Connection) -> Result<Vec<Word>> {
    let mut stmt = conn.prepare(&format!(
        "SELECT {WORD_COLUMNS}
         FROM words WHERE last_quality IS NOT NULL AND last_quality < 3
         ORDER BY next_review"
    ))?;
    let words = stmt
        .query_map([], row_to_word)?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(words)
}

pub fn save_review(
    conn: &Connection,
    id: i64,
    quality: u8,
    new_interval: i64,
    new_ef: f64,
    new_reps: i64,
) -> Result<()> {
    let now = Utc::now();
    let next_review = (now + chrono::Duration::days(new_interval)).to_rfc3339();
    conn.execute(
        "UPDATE words SET ease_factor = ?1, interval = ?2, repetitions = ?3, next_review = ?4, last_quality = ?5
         WHERE id = ?6",
        params![new_ef, new_interval, new_reps, next_review, quality, id],
    )?;
    Ok(())
}

pub fn import_words(conn: &Connection, words: &[(String, String)]) -> Result<usize> {
    let mut count = 0;
    let now = Utc::now().to_rfc3339();
    for (english, russian) in words {
        conn.execute(
            "INSERT INTO words (english, russian, ease_factor, interval, repetitions, next_review, created_at)
             VALUES (?1, ?2, 2.5, 0, 0, ?3, ?3)",
            params![english, russian, now],
        )?;
        count += 1;
    }
    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_conn() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        init_db(&conn).unwrap();
        conn
    }

    #[test]
    fn test_add_and_get() {
        let conn = test_conn();
        let word = add_word(&conn, "hello", "привет").unwrap();
        assert_eq!(word.english, "hello");
        assert_eq!(word.russian, "привет");
        assert_eq!(word.ease_factor, 2.5);
    }

    #[test]
    fn test_update() {
        let conn = test_conn();
        let word = add_word(&conn, "hello", "привет").unwrap();
        let updated = update_word(&conn, word.id, "world", "мир").unwrap();
        assert_eq!(updated.english, "world");
        assert_eq!(updated.russian, "мир");
    }

    #[test]
    fn test_delete() {
        let conn = test_conn();
        let word = add_word(&conn, "hello", "привет").unwrap();
        delete_word(&conn, word.id).unwrap();
        let words = get_all_words(&conn).unwrap();
        assert!(words.is_empty());
    }

    #[test]
    fn test_get_for_review() {
        let conn = test_conn();
        add_word(&conn, "hello", "привет").unwrap();
        let review = get_words_for_review(&conn).unwrap();
        assert_eq!(review.len(), 1);
    }

    #[test]
    fn test_add_has_null_last_quality() {
        let conn = test_conn();
        let word = add_word(&conn, "hello", "привет").unwrap();
        assert!(word.last_quality.is_none());
    }

    #[test]
    fn test_save_review_records_quality() {
        let conn = test_conn();
        let word = add_word(&conn, "hello", "привет").unwrap();
        save_review(&conn, word.id, 2, 1, 2.5, 0).unwrap();
        let updated = get_word(&conn, word.id).unwrap();
        assert_eq!(updated.last_quality, Some(2));
    }

    #[test]
    fn test_get_forgotten_words() {
        let conn = test_conn();
        let good = add_word(&conn, "good", "хороший").unwrap();
        let bad = add_word(&conn, "bad", "плохой").unwrap();
        save_review(&conn, good.id, 5, 1, 2.6, 1).unwrap();
        save_review(&conn, bad.id, 2, 1, 2.5, 0).unwrap();
        let forgotten = get_forgotten_words(&conn).unwrap();
        assert_eq!(forgotten.len(), 1);
        assert_eq!(forgotten[0].id, bad.id);
    }

    #[test]
    fn test_get_words_upcoming() {
        let conn = test_conn();
        add_word(&conn, "now", "сейчас").unwrap();
        let words = get_words_upcoming(&conn, 2).unwrap();
        assert_eq!(words.len(), 1);
    }
}
