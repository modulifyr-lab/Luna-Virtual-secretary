use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ConversationRow {
    pub id: i64,
    pub timestamp: String,
    pub role: String,
    pub text: String,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FactRow {
    pub id: i64,
    pub extracted_at: String,
    pub fact_text: String,
    pub source_conversation_id: Option<i64>,
}

pub struct MemoryStore {
    db_path: String,
}

impl MemoryStore {
    pub fn new(db_path: impl Into<String>) -> Self {
        Self {
            db_path: db_path.into(),
        }
    }

    fn get_connection(&self) -> Result<rusqlite::Connection, String> {
        Connection::open(&self.db_path).map_err(|e| format!("Failed to open SQLite database at {}: {}", self.db_path, e))
    }

    pub fn init(&self) -> Result<(), String> {
        let conn = self.get_connection()?;

        // Step 1 - Schema definition:
        // conversations table: id, timestamp, role (user/assistant), text, source (voice/typed).
        // facts table: id, extracted_at, fact_text, source_conversation_id (nullable).
        conn.execute(
            "CREATE TABLE IF NOT EXISTS conversations (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp TEXT NOT NULL,
                role TEXT NOT NULL,
                text TEXT NOT NULL,
                source TEXT NOT NULL
            )",
            [],
        )
        .map_err(|e| format!("Failed to create conversations table: {}", e))?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS facts (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                extracted_at TEXT NOT NULL,
                fact_text TEXT NOT NULL,
                source_conversation_id INTEGER
            )",
            [],
        )
        .map_err(|e| format!("Failed to create facts table: {}", e))?;

        Ok(())
    }

    pub fn log_conversation(&self, role: &str, text: &str, source: &str) -> Result<i64, String> {
        let conn = self.get_connection()?;
        let timestamp = chrono_lite_timestamp();
        conn.execute(
            "INSERT INTO conversations (timestamp, role, text, source) VALUES (?1, ?2, ?3, ?4)",
            params![timestamp, role, text, source],
        )
        .map_err(|e| format!("Failed to insert conversation: {}", e))?;

        Ok(conn.last_insert_rowid())
    }

    pub fn get_recent_conversations(&self, limit: usize) -> Result<Vec<ConversationRow>, String> {
        let conn = self.get_connection()?;
        let mut stmt = conn
            .prepare("SELECT id, timestamp, role, text, source FROM conversations ORDER BY id DESC LIMIT ?1")
            .map_err(|e| format!("Failed to prepare conversation query: {}", e))?;

        let rows = stmt
            .query_map(params![limit as i64], |row| {
                Ok(ConversationRow {
                    id: row.get(0)?,
                    timestamp: row.get(1)?,
                    role: row.get(2)?,
                    text: row.get(3)?,
                    source: row.get(4)?,
                })
            })
            .map_err(|e| format!("Failed to query conversations: {}", e))?;

        let mut conversations = Vec::new();
        for res in rows {
            if let Ok(conv) = res {
                conversations.push(conv);
            }
        }
        // Reverse to return them in chronological order
        conversations.reverse();
        Ok(conversations)
    }

    pub fn get_conversation_count(&self) -> Result<usize, String> {
        let conn = self.get_connection()?;
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM conversations", [], |row| row.get(0))
            .map_err(|e| format!("Failed to count conversations: {}", e))?;
        Ok(count as usize)
    }

    pub fn store_fact(&self, fact_text: &str, source_conversation_id: Option<i64>) -> Result<(), String> {
        let trimmed_fact = fact_text.trim();
        if trimmed_fact.is_empty() {
            return Ok(());
        }

        let conn = self.get_connection()?;

        // Simple deduplication against existing facts (case-insensitive trimmed text match)
        let mut stmt = conn
            .prepare("SELECT fact_text FROM facts")
            .map_err(|e| format!("Failed to prepare facts select query: {}", e))?;

        let existing_facts = stmt
            .query_map([], |row| row.get::<_, String>(0))
            .map_err(|e| format!("Failed to query facts: {}", e))?;

        let lower_trimmed = trimmed_fact.to_lowercase();
        for fact_res in existing_facts {
            if let Ok(existing) = fact_res {
                if existing.trim().to_lowercase() == lower_trimmed {
                    // Fact already exists, skip duplicate insertion
                    return Ok(());
                }
            }
        }

        let extracted_at = chrono_lite_timestamp();
        conn.execute(
            "INSERT INTO facts (extracted_at, fact_text, source_conversation_id) VALUES (?1, ?2, ?3)",
            params![extracted_at, trimmed_fact, source_conversation_id],
        )
        .map_err(|e| format!("Failed to insert fact: {}", e))?;

        Ok(())
    }

    pub fn get_recent_facts(&self, limit: usize) -> Result<Vec<FactRow>, String> {
        let conn = self.get_connection()?;
        let mut stmt = conn
            .prepare("SELECT id, extracted_at, fact_text, source_conversation_id FROM facts ORDER BY id DESC LIMIT ?1")
            .map_err(|e| format!("Failed to prepare facts query: {}", e))?;

        let rows = stmt
            .query_map(params![limit as i64], |row| {
                Ok(FactRow {
                    id: row.get(0)?,
                    extracted_at: row.get(1)?,
                    fact_text: row.get(2)?,
                    source_conversation_id: row.get(3)?,
                })
            })
            .map_err(|e| format!("Failed to query facts: {}", e))?;

        let mut facts = Vec::new();
        for res in rows {
            if let Ok(fact) = res {
                facts.push(fact);
            }
        }
        Ok(facts)
    }
}

/// Helper function to generate an ISO-8601-like timestamp using standard library.
fn chrono_lite_timestamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let start = SystemTime::now();
    let since_epoch = start.duration_since(UNIX_EPOCH).unwrap_or_default();
    format!("{}", since_epoch.as_secs())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_store_init_and_operations() {
        let temp_dir = std::env::temp_dir();
        let db_path = temp_dir.join(format!("luna_test_{}.db", std::process::id()));
        let db_path_str = db_path.to_str().unwrap();

        let store = MemoryStore::new(db_path_str);
        assert!(store.init().is_ok());

        // Test logging conversation
        let c1_id = store.log_conversation("user", "Hello Luna, I like coffee.", "typed").unwrap();
        let _c2_id = store.log_conversation("assistant", "Nice to meet you! Coffee is great.", "system").unwrap();

        assert_eq!(store.get_conversation_count().unwrap(), 2);

        let convs = store.get_recent_conversations(10).unwrap();
        assert_eq!(convs.len(), 2);
        assert_eq!(convs[0].role, "user");
        assert_eq!(convs[0].text, "Hello Luna, I like coffee.");
        assert_eq!(convs[1].role, "assistant");

        // Test storing facts and deduplication
        store.store_fact("User likes coffee", Some(c1_id)).unwrap();
        store.store_fact("User likes coffee ", Some(c1_id)).unwrap(); // Duplicate, should be ignored
        store.store_fact("user likes COFFEE", Some(c1_id)).unwrap(); // Case-insensitive duplicate

        let facts = store.get_recent_facts(20).unwrap();
        assert_eq!(facts.len(), 1);
        assert_eq!(facts[0].fact_text, "User likes coffee");
        assert_eq!(facts[0].source_conversation_id, Some(c1_id));

        // Cleanup
        let _ = std::fs::remove_file(db_path);
    }
}
