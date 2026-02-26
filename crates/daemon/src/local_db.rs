//! Local SQLite database for daemon state persistence.

pub struct LocalDb {
    // TODO: SQLite connection
}

impl LocalDb {
    pub fn open(_path: &str) -> anyhow::Result<Self> {
        todo!("Open or create SQLite database")
    }
}
