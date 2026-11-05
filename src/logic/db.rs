use log::{debug, error, info, trace, warn};
use sqlx::{
    AssertSqlSafe, Pool, Row, Sqlite,
    sqlite::{SqliteConnectOptions, SqlitePool},
};
use std::collections::HashMap;
use std::fs;

use crate::logic::integrations::types::{Chat, MessageType, ReceivedMessage};

const CREATE_CHATS: &str = r#"
    CREATE TABLE IF NOT EXISTS chats (
        chat_id INTEGER NOT NULL,
        account_id INTEGER NOT NULL,
        title TEXT,
        created_at INTEGER NOT NULL DEFAULT (strftime('%s','now')),
        PRIMARY KEY (chat_id, account_id)
    );
"#;

const CREATE_MESSAGES: &str = r#"
    CREATE TABLE IF NOT EXISTS messages (
        id INTEGER NOT NULL,
        chat_id INTEGER NOT NULL,
        account_id INTEGER NOT NULL,
        username TEXT NOT NULL,
        text TEXT NOT NULL DEFAULT '',
        name TEXT NOT NULL DEFAULT '',
        date TEXT NOT NULL,
        is_read INTEGER NOT NULL DEFAULT 0,
        sender_is_you INTEGER NOT NULL DEFAULT 0,
        msg_type INTEGER NOT NULL,
        PRIMARY KEY (account_id, chat_id, id)
    );
"#;

#[derive(Clone)]
pub struct Db {
    pub pool: Pool<Sqlite>,
}

impl Db {
    pub async fn connect() -> Self {
        debug!("Creating 'data' directory if missing...");
        if let Err(e) = fs::create_dir_all("data") {
            error!("Failed to create 'data' directory: {}", e);
            panic!("{}", e);
        }

        info!("Connecting to database at path: data/save.db");
        let connection_options = SqliteConnectOptions::new()
            .filename("data/save.db")
            .create_if_missing(true);

        let pool = match SqlitePool::connect_with(connection_options).await {
            Ok(p) => {
                info!("Successfully connected to SQLite database.");
                p
            }
            Err(e) => {
                error!("Critical error connecting to SQLite: {}", e);
                panic!("{}", e);
            }
        };

        Self { pool }
    }

    pub async fn init(&self) {
        debug!("Initializing database table structures...");

        if let Err(e) = self.migrate_legacy_tables().await {
            error!("Failed to migrate legacy database tables: {}", e);
        }

        match sqlx::query(CREATE_CHATS).execute(&self.pool).await {
            Ok(_) => debug!("Table 'chats' verified/created successfully."),
            Err(e) => {
                error!("Failed to create 'chats' table: {}", e);
                panic!("{}", e);
            }
        }

        match sqlx::query(CREATE_MESSAGES).execute(&self.pool).await {
            Ok(_) => debug!("Table 'messages' verified/created successfully."),
            Err(e) => {
                error!("Failed to create 'messages' table: {}", e);
                panic!("{}", e);
            }
        }

        if let Err(e) = sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_messages_account_chat
             ON messages(account_id, chat_id);",
        )
        .execute(&self.pool)
        .await
        {
            warn!(
                "Failed to create index on messages(account_id, chat_id): {}",
                e
            );
        }

        match sqlx::query("PRAGMA journal_mode=WAL;")
            .execute(&self.pool)
            .await
        {
            Ok(_) => debug!("SQLite WAL journal mode enabled successfully."),
            Err(e) => warn!("Failed to set PRAGMA journal_mode=WAL: {}", e),
        }

        info!("Database successfully initialized and optimized.");
    }

    async fn migrate_legacy_tables(&self) -> Result<(), sqlx::Error> {
        let chats = self.table_info("chats").await?;
        if !chats.is_empty() && !is_current_chats_schema(&chats) {
            info!("Migrating legacy 'chats' table to the new schema...");
            self.migrate_chats_table(&chats).await?;
        }

        let messages = self.table_info("messages").await?;
        if !messages.is_empty() && !is_current_messages_schema(&messages) {
            info!("Migrating legacy 'messages' table to the new schema...");
            self.migrate_messages_table(&messages).await?;
        }

        Ok(())
    }

    async fn table_info(&self, table: &str) -> Result<Vec<TableColumn>, sqlx::Error> {
        let rows = sqlx::query(AssertSqlSafe(format!("PRAGMA table_info({table})")))
            .fetch_all(&self.pool)
            .await?;

        Ok(rows
            .into_iter()
            .map(|row| TableColumn {
                name: row.get("name"),
                pk: row.get("pk"),
            })
            .collect())
    }

    async fn migrate_chats_table(&self, columns: &[TableColumn]) -> Result<(), sqlx::Error> {
        let has_account = has_column(columns, "account_id");
        let has_title = has_column(columns, "title");
        let has_created = has_column(columns, "created_at");

        let account = if has_account {
            "COALESCE(account_id, 0)"
        } else {
            "0"
        };
        let title = if has_title { "title" } else { "NULL" };
        let created = if has_created {
            "COALESCE(created_at, strftime('%s','now'))"
        } else {
            "strftime('%s','now')"
        };

        let mut tx = self.pool.begin().await?;

        sqlx::query("DROP TABLE IF EXISTS chats_legacy;")
            .execute(&mut *tx)
            .await?;
        sqlx::query("ALTER TABLE chats RENAME TO chats_legacy;")
            .execute(&mut *tx)
            .await?;
        sqlx::query(CREATE_CHATS).execute(&mut *tx).await?;

        sqlx::query(AssertSqlSafe(format!(
            r#"
            INSERT OR IGNORE INTO chats (chat_id, account_id, title, created_at)
            SELECT chat_id, {account}, {title}, {created}
            FROM chats_legacy;
            "#
        )))
        .execute(&mut *tx)
        .await?;

        sqlx::query("DROP TABLE chats_legacy;")
            .execute(&mut *tx)
            .await?;

        tx.commit().await?;

        Ok(())
    }

    async fn migrate_messages_table(&self, columns: &[TableColumn]) -> Result<(), sqlx::Error> {
        let has_account = has_column(columns, "account_id");
        let has_sender = has_column(columns, "sender_is_you");
        let has_is_bot = has_column(columns, "is_bot");
        let has_is_read = has_column(columns, "is_read");
        let has_text = has_column(columns, "text");
        let has_date = has_column(columns, "date");
        let has_username = has_column(columns, "username");
        let has_msg_type = has_column(columns, "msg_type");

        let account = if has_account {
            "COALESCE(account_id, 0)"
        } else {
            "0"
        };
        let sender = if has_sender {
            "COALESCE(sender_is_you, 0)"
        } else if has_is_bot {
            "COALESCE(is_bot, 0)"
        } else {
            "0"
        };
        let is_read = if has_is_read {
            "COALESCE(is_read, 0)"
        } else {
            "0"
        };
        let text = if has_text { "COALESCE(text, '')" } else { "''" };
        let date = if has_date { "COALESCE(date, '')" } else { "''" };
        let username = if has_username {
            "COALESCE(username, '')"
        } else {
            "''"
        };
        let msg_type = if has_msg_type {
            "COALESCE(msg_type, 0)"
        } else {
            "0"
        };

        let mut tx = self.pool.begin().await?;

        sqlx::query("DROP TABLE IF EXISTS messages_legacy;")
            .execute(&mut *tx)
            .await?;
        sqlx::query("ALTER TABLE messages RENAME TO messages_legacy;")
            .execute(&mut *tx)
            .await?;
        sqlx::query(CREATE_MESSAGES).execute(&mut *tx).await?;

        sqlx::query(AssertSqlSafe(format!(
            r#"
            INSERT OR REPLACE INTO messages (
                id, chat_id, account_id, username, text, date,
                is_read, sender_is_you, msg_type
            )
            SELECT
                id, chat_id, {account}, {username}, {text}, {date},
                {is_read}, {sender}, {msg_type}
            FROM messages_legacy;
            "#
        )))
        .execute(&mut *tx)
        .await?;

        sqlx::query("DROP TABLE messages_legacy;")
            .execute(&mut *tx)
            .await?;

        tx.commit().await?;

        Ok(())
    }

    pub async fn add_message(&self, msg: &ReceivedMessage) {
        let (msg_type, text, name) = message_to_row(&msg.t);

        let name = match name {
            Some(name) => name,
            None => String::new(),
        };

        match sqlx::query(
            r#"
            INSERT OR REPLACE INTO messages (
                id, chat_id, account_id, username, text, name, date,
                is_read, sender_is_you, msg_type
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10);
            "#,
        )
        .bind(msg.id)
        .bind(msg.chat_id)
        .bind(msg.account_id)
        .bind(&msg.username)
        .bind(text)
        .bind(name)
        .bind(&msg.date)
        .bind(msg.readed)
        .bind(msg.sender_is_you)
        .bind(msg_type)
        .execute(&self.pool)
        .await
        {
            Ok(_) => trace!("Message successfully written to database."),
            Err(e) => {
                error!(
                    "Failed to save message to DB for chat {} (account {}): {}",
                    msg.chat_id, msg.account_id, e
                );
            }
        }
    }

    pub async fn edit_message(
        &self,
        account_id: i64,
        chat_id: i64,
        message_id: i32,
        new_text: &str,
    ) {
        match sqlx::query(
            r#"
            UPDATE messages
            SET text = ?1
            WHERE account_id = ?2 AND chat_id = ?3 AND id = ?4;
            "#,
        )
        .bind(new_text)
        .bind(account_id)
        .bind(chat_id)
        .bind(message_id)
        .execute(&self.pool)
        .await
        {
            Ok(result) => {
                trace!(
                    "Message {} successfully edited in database (rows affected: {}).",
                    message_id,
                    result.rows_affected()
                );
            }
            Err(e) => {
                error!(
                    "Failed to edit message {} in DB for chat {} (account {}): {}",
                    message_id, chat_id, account_id, e
                );
            }
        }
    }

    pub async fn _mark_readed_all(&self, account_id: i64, chat_id: i64) {
        debug!(
            "Marking all messages as read for chat {} (account {})...",
            chat_id, account_id
        );

        match sqlx::query(
            r#"
            UPDATE messages
            SET is_read = 1
            WHERE account_id = ?1 AND chat_id = ?2 AND is_read = 0;
            "#,
        )
        .bind(account_id)
        .bind(chat_id)
        .execute(&self.pool)
        .await
        {
            Ok(result) => {
                trace!(
                    "Successfully marked messages as read. Rows affected: {}",
                    result.rows_affected()
                );
            }
            Err(e) => {
                error!(
                    "Failed to mark messages as read for chat {} (account {}): {}",
                    chat_id, account_id, e
                );
            }
        }
    }

    pub async fn add_chat(&self, account_id: i64, chat_id: i64, title: &str) {
        if let Err(e) = sqlx::query(
            r#"
            INSERT INTO chats (chat_id, account_id, title)
            VALUES (?1, ?2, ?3)
            ON CONFLICT(chat_id, account_id) DO UPDATE SET
                title = CASE
                    WHEN excluded.title IS NOT NULL
                         AND excluded.title != ''
                    THEN excluded.title
                    ELSE chats.title
                END;
            "#,
        )
        .bind(chat_id)
        .bind(account_id)
        .bind(title)
        .execute(&self.pool)
        .await
        {
            error!(
                "Failed to add/update chat {} (account {}): {}",
                chat_id, account_id, e
            );
        }
    }

    pub async fn delete_message(&self, account_id: i64, chat_id: i64, message_id: i32) {
        match sqlx::query(
            r#"
            DELETE FROM messages
            WHERE account_id = ?1 AND chat_id = ?2 AND id = ?3;
            "#,
        )
        .bind(account_id)
        .bind(chat_id)
        .bind(message_id)
        .execute(&self.pool)
        .await
        {
            Ok(result) => {
                if result.rows_affected() > 0 {
                    info!(
                        "Deleted message {} from chat {} (account {})",
                        message_id, chat_id, account_id
                    );
                } else {
                    warn!(
                        "Message {} not found in chat {} (account {})",
                        message_id, chat_id, account_id
                    );
                }
            }
            Err(e) => {
                error!(
                    "Failed to delete message {} from chat {} (account {}): {}",
                    message_id, chat_id, account_id, e
                );
            }
        }
    }

    pub async fn load_chats(&self) -> Vec<Chat> {
        debug!("Loading all chat histories from database...");

        let rows: Vec<MessageRow> = match sqlx::query_as(
            r#"
            SELECT
                m.id,
                m.chat_id,
                m.account_id,
                m.username,
                m.text,
                m.name,
                m.date,
                m.is_read,
                m.sender_is_you,
                m.msg_type,
                COALESCE(c.title, '') AS chat_title
            FROM messages AS m
            LEFT JOIN chats AS c
                ON c.chat_id = m.chat_id AND c.account_id = m.account_id
            ORDER BY m.chat_id ASC, m.id ASC;
            "#,
        )
        .fetch_all(&self.pool)
        .await
        {
            Ok(rows) => rows,
            Err(e) => {
                error!("Failed to fetch messages from database: {}", e);
                return Vec::new();
            }
        };

        let rows_count = rows.len();
        let mut chats: HashMap<i64, Vec<MessageRow>> = HashMap::new();

        for row in rows {
            chats.entry(row.chat_id).or_default().push(row);
        }

        let mut formatted_chats: Vec<Chat> = chats
            .into_iter()
            .map(|(chat_id, rows)| {
                let title = rows.first().map(|row| row.chat_title.trim()).unwrap_or("");

                let name = if !title.is_empty() {
                    title.to_string()
                } else {
                    format!("Chat {chat_id}")
                };

                let unreaded_count = rows.iter().filter(|row| !row.is_read).count() as u32;

                let messages: Vec<ReceivedMessage> = rows
                    .into_iter()
                    .map(|row| ReceivedMessage {
                        id: row.id,
                        username: row.username,
                        t: row_to_message_type(row.msg_type, row.text, row.name),
                        date: row.date,
                        chat_id: row.chat_id,
                        readed: row.is_read,
                        sender_is_you: row.sender_is_you,
                        account_id: row.account_id,
                    })
                    .collect();

                Chat {
                    chat_id,
                    messages,
                    unreaded_count,
                    name,
                }
            })
            .collect();

        formatted_chats.sort_by_key(|chat| chat.chat_id);

        info!(
            "Successfully loaded {} messages, structured into {} chat objects.",
            rows_count,
            formatted_chats.len()
        );

        formatted_chats
    }
}

fn message_to_row(message_type: &MessageType) -> (i64, String, Option<String>) {
    match message_type {
        MessageType::TEXT { text } => (0, text.clone(), None),
        MessageType::PHOTO { path } => (1, path.clone(), None),
        MessageType::FILE { path, name } => (2, path.clone(), Some(name.clone())),
        MessageType::STICKER { path } => (3, path.clone(), None),
        MessageType::AUDIO { path, name } => (4, path.clone(), Some(name.clone())),
        MessageType::VOICE { path } => (5, path.clone(), None),
    }
}

fn row_to_message_type(msg_type: i64, text: String, name: String) -> MessageType {
    match msg_type {
        0 => MessageType::TEXT { text },
        1 => MessageType::PHOTO { path: text },
        2 => MessageType::FILE {
            path: text,
            name: name,
        },
        3 => MessageType::STICKER { path: text },
        4 => MessageType::AUDIO {
            path: text,
            name: name,
        },
        5 => MessageType::VOICE { path: text },
        _ => MessageType::TEXT { text },
    }
}

#[derive(Clone)]
struct TableColumn {
    name: String,
    pk: i64,
}

fn has_column(columns: &[TableColumn], name: &str) -> bool {
    columns.iter().any(|column| column.name == name)
}

fn is_current_chats_schema(columns: &[TableColumn]) -> bool {
    has_column(columns, "chat_id")
        && has_column(columns, "account_id")
        && has_column(columns, "title")
        && has_column(columns, "created_at")
        && primary_key(columns) == ["chat_id", "account_id"]
}

fn is_current_messages_schema(columns: &[TableColumn]) -> bool {
    has_column(columns, "id")
        && has_column(columns, "chat_id")
        && has_column(columns, "account_id")
        && has_column(columns, "username")
        && has_column(columns, "text")
        && has_column(columns, "name")
        && has_column(columns, "date")
        && has_column(columns, "is_read")
        && has_column(columns, "sender_is_you")
        && has_column(columns, "msg_type")
        && primary_key(columns) == ["account_id", "chat_id", "id"]
}

fn primary_key(columns: &[TableColumn]) -> Vec<&str> {
    let mut key: Vec<&TableColumn> = columns.iter().filter(|column| column.pk > 0).collect();
    key.sort_by_key(|column| column.pk);
    key.into_iter().map(|column| column.name.as_str()).collect()
}

#[derive(Clone, sqlx::FromRow)]
struct MessageRow {
    id: i32,
    chat_id: i64,
    account_id: i64,
    username: String,
    text: String,
    name: String,
    date: String,
    is_read: bool,
    sender_is_you: bool,
    msg_type: i64,
    chat_title: String,
}
