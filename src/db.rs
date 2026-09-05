use chrono::Utc;
use rusqlite::{params, Connection, Result};

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
            created_at TEXT NOT NULL
        );",
    )?;
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

pub fn get_all_words(conn: &Connection) -> Result<Vec<Word>> {
    let mut stmt = conn.prepare(
        "SELECT id, english, russian, ease_factor, interval, repetitions, next_review, created_at
         FROM words ORDER BY id",
    )?;
    let words = stmt
        .query_map([], |row| {
            Ok(Word {
                id: row.get(0)?,
                english: row.get(1)?,
                russian: row.get(2)?,
                ease_factor: row.get(3)?,
                interval: row.get(4)?,
                repetitions: row.get(5)?,
                next_review: row.get(6)?,
                created_at: row.get(7)?,
            })
        })?
        .collect::<Result<Vec<_>>>()?;
    Ok(words)
}

pub fn get_word(conn: &Connection, id: i64) -> Result<Word> {
    conn.query_row(
        "SELECT id, english, russian, ease_factor, interval, repetitions, next_review, created_at
         FROM words WHERE id = ?1",
        params![id],
        |row| {
            Ok(Word {
                id: row.get(0)?,
                english: row.get(1)?,
                russian: row.get(2)?,
                ease_factor: row.get(3)?,
                interval: row.get(4)?,
                repetitions: row.get(5)?,
                next_review: row.get(6)?,
                created_at: row.get(7)?,
            })
        },
    )
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
    let mut stmt = conn.prepare(
        "SELECT id, english, russian, ease_factor, interval, repetitions, next_review, created_at
         FROM words WHERE next_review <= ?1 ORDER BY next_review",
    )?;
    let words = stmt
        .query_map(params![now], |row| {
            Ok(Word {
                id: row.get(0)?,
                english: row.get(1)?,
                russian: row.get(2)?,
                ease_factor: row.get(3)?,
                interval: row.get(4)?,
                repetitions: row.get(5)?,
                next_review: row.get(6)?,
                created_at: row.get(7)?,
            })
        })?
        .collect::<Result<Vec<_>>>()?;
    Ok(words)
}

pub fn save_review(
    conn: &Connection,
    id: i64,
    _quality: u8,
    new_interval: i64,
    new_ef: f64,
    new_reps: i64,
) -> Result<()> {
    let now = Utc::now();
    let next_review = (now + chrono::Duration::days(new_interval)).to_rfc3339();
    conn.execute(
        "UPDATE words SET ease_factor = ?1, interval = ?2, repetitions = ?3, next_review = ?4
         WHERE id = ?5",
        params![new_ef, new_interval, new_reps, next_review, id],
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
}
