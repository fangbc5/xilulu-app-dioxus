use sqlx::{sqlite::{SqliteConnectOptions, SqlitePoolOptions}, Pool, Sqlite};
use tracing::info;

#[derive(Clone)]
pub struct DbManager {
    pub pool: Pool<Sqlite>,
}

use std::str::FromStr;

impl DbManager {
    /// 初始化 SQLite 连接池并执行建表迁移
    pub async fn new(db_url: &str) -> Result<Self, sqlx::Error> {
        info!("Initializing SQLite database at: {}", db_url);

        let options = SqliteConnectOptions::from_str(db_url)?
            .create_if_missing(true)
            .pragma("foreign_keys", "ON")
            .pragma("journal_mode", "WAL");

        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect_with(options)
            .await?;

        let manager = Self { pool };
        manager.run_migrations().await?;

        Ok(manager)
    }

    async fn run_migrations(&self) -> Result<(), sqlx::Error> {
        // ─────────────────────────────────────────────────────────
        // Table: message  (对齐服务端 `message` 表)
        // ─────────────────────────────────────────────────────────
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS message (
                id           INTEGER PRIMARY KEY,        -- 与服务端 message.id 对齐
                msg_id       TEXT UNIQUE NOT NULL,       -- 本地临时 UUID，发送成功后更新为服务端 id
                room_id      INTEGER NOT NULL,
                from_uid     INTEGER NOT NULL,           -- 对齐服务端 message.from_uid
                content      TEXT,                       -- 可为 NULL（例如撤回消息）
                type         INTEGER NOT NULL DEFAULT 1, -- 1文本 2图片 3文件 4语音 5视频 6撤回 7系统
                reply_msg_id INTEGER,
                status       INTEGER NOT NULL DEFAULT 0, -- 0正常 1撤回
                extra        TEXT,                       -- JSON 扩展字段
                local_status INTEGER NOT NULL DEFAULT 0, -- 仅客户端：0已完成 1发送中 2失败
                created_at   INTEGER NOT NULL,           -- 毫秒时间戳
                updated_at   INTEGER NOT NULL DEFAULT 0
            );
            CREATE INDEX IF NOT EXISTS idx_message_room_id ON message (room_id);
            CREATE INDEX IF NOT EXISTS idx_message_created_at ON message (room_id, created_at DESC);
            "#,
        )
        .execute(&self.pool)
        .await?;

        // ─────────────────────────────────────────────────────────
        // Table: contact  (对齐服务端 `contact` 表)
        // ─────────────────────────────────────────────────────────
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS contact (
                room_id      INTEGER PRIMARY KEY,
                room_type    INTEGER NOT NULL DEFAULT 1, -- 来自 room.type，1单聊 2群聊
                read_time    INTEGER,
                active_time  INTEGER,
                last_msg_id  INTEGER,
                read_msg_id  INTEGER,
                clear_msg_id INTEGER NOT NULL DEFAULT 0,
                is_mute      INTEGER NOT NULL DEFAULT 0,
                is_top       INTEGER NOT NULL DEFAULT 0,
                is_deleted   INTEGER NOT NULL DEFAULT 0,
                unread_count INTEGER NOT NULL DEFAULT 0,
                friend_uid   INTEGER,                   -- 本地冗余，单聊对方 uid
                draft_text   TEXT,                      -- 纯客户端草稿
                created_at   INTEGER,
                updated_at   INTEGER NOT NULL
            );
            "#,
        )
        .execute(&self.pool)
        .await?;

        // ─────────────────────────────────────────────────────────
        // Table: user_friend  (对齐服务端 `user_friend` 表)
        // ─────────────────────────────────────────────────────────
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS user_friend (
                id           INTEGER PRIMARY KEY AUTOINCREMENT,
                uid          INTEGER NOT NULL,           -- 自己的 uid
                friend_uid   INTEGER NOT NULL,           -- 好友的 uid
                remark       TEXT,
                status       INTEGER NOT NULL DEFAULT 1, -- 1正常 2删除
                created_at   INTEGER,
                updated_at   INTEGER NOT NULL,
                UNIQUE(uid, friend_uid)
            );
            "#,
        )
        .execute(&self.pool)
        .await?;

        // ─────────────────────────────────────────────────────────
        // Table: room_group  (对齐服务端 `room_group` 表)
        // ─────────────────────────────────────────────────────────
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS room_group (
                id           INTEGER PRIMARY KEY,
                room_id      INTEGER NOT NULL UNIQUE,
                name         TEXT NOT NULL,
                avatar       TEXT,
                notice       TEXT,
                is_deleted   INTEGER NOT NULL DEFAULT 0,
                created_by   INTEGER,
                created_at   INTEGER,
                updated_at   INTEGER NOT NULL
            );
            "#,
        )
        .execute(&self.pool)
        .await?;

        // ─────────────────────────────────────────────────────────
        // Table: group_member  (对齐服务端 `group_member` 表)
        // ─────────────────────────────────────────────────────────
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS group_member (
                id           INTEGER PRIMARY KEY AUTOINCREMENT,
                group_id     INTEGER NOT NULL,
                uid          INTEGER NOT NULL,
                role         INTEGER NOT NULL DEFAULT 3, -- 1群主 2管理员 3普通成员
                created_at   INTEGER NOT NULL,
                updated_at   INTEGER NOT NULL,
                UNIQUE(group_id, uid)
            );
            "#,
        )
        .execute(&self.pool)
        .await?;

        // ─────────────────────────────────────────────────────────
        // Table: user_profile  (来自 ms-identity UserBrief，本地缓存)
        // ─────────────────────────────────────────────────────────
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS user_profile (
                uid          INTEGER PRIMARY KEY,        -- 对应 user.id
                nick_name    TEXT NOT NULL,
                avatar       TEXT,
                updated_at   INTEGER NOT NULL
            );
            "#,
        )
        .execute(&self.pool)
        .await?;

        // ─────────────────────────────────────────────────────────
        // Table: sync_cursor  (纯客户端：增量同步游标)
        // ─────────────────────────────────────────────────────────
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS sync_cursor (
                module       TEXT PRIMARY KEY,
                timestamp_ms INTEGER NOT NULL
            );
            "#,
        )
        .execute(&self.pool)
        .await?;

        // ─────────────────────────────────────────────────────────
        // Table: sync_queue  (纯客户端：离线消息重发队列)
        // ─────────────────────────────────────────────────────────
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS sync_queue (
                id           INTEGER PRIMARY KEY AUTOINCREMENT,
                payload      TEXT NOT NULL,
                retry_count  INTEGER NOT NULL DEFAULT 0,
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
