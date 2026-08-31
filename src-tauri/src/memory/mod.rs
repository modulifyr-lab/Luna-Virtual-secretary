pub struct MemoryStore {
    db_path: String,
}

impl MemoryStore {
    pub fn new(db_path: impl Into<String>) -> Self {
        Self {
            db_path: db_path.into(),
        }
    }

    pub fn init(&self) -> Result<(), String> {
        // TODO: Initialize rusqlite DB connection
        // TODO: Create `conversations` table (id, timestamp, role, content)
        // TODO: Create `facts` table (id, key, value, timestamp)
        Ok(())
    }

    pub fn log_conversation(&self, _role: &str, _content: &str) -> Result<(), String> {
        // TODO: Insert row into conversations table
        Ok(())
    }

    pub fn store_fact(&self, _key: &str, _value: &str) -> Result<(), String> {
        // TODO: Insert/Update fact in facts table
        Ok(())
    }
}
