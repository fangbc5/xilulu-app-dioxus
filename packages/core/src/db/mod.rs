use sqlx::{sqlite::SqlitePoolOptions, Pool, Sqlite};
use std::sync::Arc;
use tracing::{info, error};

#[derive(Clone)]
pub struct DbManager {
    pub pool: Pool<Sqlite>,
}

impl DbManager {
    /// Initialize a new SQLite connection pool and run necessary migrations.
    pub async fn new(db_url: &str) -> Result<Self, sqlx::Error> {
        info!("Initializing SQLite database at: {}", db_url);

        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(db_url)
            .await?;

        let manager = Self { pool };
        manager.run_migrations().await?;

        Ok(manager)
    }

    async fn run_migrations(&self) -> Result<(), sqlx::Error> {
        // Table: messages
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS messages (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                msg_id TEXT UNIQUE NOT NULL,
                room_id INTEGER NOT NULL,
                sender_uid INTEGER NOT NULL,
                msg_type INTEGER NOT NULL,
                content TEXT NOT NULL,
                local_status INTEGER NOT NULL DEFAULT 0, -- 0: Success, 1: Sending, 2: Failed
                created_at INTEGER NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_messages_room_id ON messages (room_id);
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Table: contacts (conversations)
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS contacts (
                room_id INTEGER PRIMARY KEY,
                room_type INTEGER NOT NULL,
                unread_count INTEGER NOT NULL DEFAULT 0,
                last_msg_id TEXT,
                draft_text TEXT,
                active_time INTEGER,
                is_mute INTEGER NOT NULL DEFAULT 0,
                is_top INTEGER NOT NULL DEFAULT 0,
                is_deleted INTEGER NOT NULL DEFAULT 0,
                friend_uid INTEGER,
                read_msg_id INTEGER,
                clear_msg_id INTEGER,
                updated_at INTEGER NOT NULL
            );
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Table: friends
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS friends (
                uid INTEGER PRIMARY KEY,
                remark TEXT,
                status INTEGER NOT NULL DEFAULT 1, -- 1: Normal, 2: Deleted
                updated_at INTEGER NOT NULL
            );
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Table: groups
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS groups (
                group_id INTEGER PRIMARY KEY,
                room_id INTEGER NOT NULL UNIQUE,
                name TEXT NOT NULL,
                avatar TEXT,
                notice TEXT,
                is_deleted INTEGER NOT NULL DEFAULT 0,
                owner_uid INTEGER,
                updated_at INTEGER NOT NULL
            );
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Table: group_members
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS group_members (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                group_id INTEGER NOT NULL,
                uid INTEGER NOT NULL,
                role INTEGER NOT NULL,
                updated_at INTEGER NOT NULL,
                UNIQUE(group_id, uid)
            );
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Table: user_profiles
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS user_profiles (
                uid INTEGER PRIMARY KEY,
                nick_name TEXT NOT NULL,
                avatar TEXT,
                updated_at INTEGER NOT NULL
            );
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Table: sync_cursors
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS sync_cursors (
                module TEXT PRIMARY KEY,
                timestamp_ms INTEGER NOT NULL
            );
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Table: sync_queue (Dead-letter queue for offline resending)
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS sync_queue (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                payload TEXT NOT NULL,
                retry_count INTEGER NOT NULL DEFAULT 0,
                next_retry_at INTEGER NOT NULL
            );
            "#,
        )
        .execute(&self.pool)
        .await?;

        info!("SQLite migrations applied successfully.");
        Ok(())
    }
}
