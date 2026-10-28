use crate::utils::types::{
    self, Chat, MessagePacket,
    MsgType::{FILE, IMAGE, STICKER, TEXT},
};

use log::{debug, error, info, trace, warn};
use sqlx::{
    Pool, Sqlite,
    sqlite::{SqliteConnectOptions, SqlitePool},
};
use std::collections::HashMap;
use std::fs;
use teloxide::types::{ChatId, MessageId};

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

        // Creating 'chats' table
        match sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS chats (
                chat_id INTEGER PRIMARY KEY,
                title TEXT,
                created_at INTEGER DEFAULT (strftime('%s','now')),
                account_id INTEGER NOT NULL
            );
            "#,
        )
        .execute(&self.pool)
        .await
        {
            Ok(_) => debug!("Table 'chats' verified/created successfully."),
            Err(e) => error!("Failed to create 'chats' table: {}", e),
        }

        // Creating 'messages' table
        match sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS messages (
                id INTEGER NOT NULL,
                chat_id INTEGER NOT NULL,
                username TEXT NOT NULL,
                text TEXT NOT NULL,
                date TEXT NOT NULL,
                is_bot INTEGER NOT NULL DEFAULT 0,
                is_read INTEGER NOT NULL DEFAULT 0,
                msg_type INTEGER NOT NULL,
                msg_name TEXT,
                account_id INTEGER NOT NULL
            );
            "#,
        )
        .execute(&self.pool)
        .await
        {
            Ok(_) => debug!("Table 'messages' verified/created successfully."),
            Err(e) => {
                error!("Failed to create 'messages' table: {}", e);
                panic!("{}", e);
            }
        }

        // Creating index
        match sqlx::query("CREATE INDEX IF NOT EXISTS idx_messages_chat_id ON messages(chat_id);")
            .execute(&self.pool)
            .await
        {
            Ok(_) => debug!("Index 'idx_messages_chat_id' verified/created."),
            Err(e) => warn!("Failed to create index on messages(chat_id): {}", e),
        }

        // Enabling WAL mode
        match sqlx::query("PRAGMA journal_mode=WAL;")
            .execute(&self.pool)
            .await
        {
            Ok(_) => debug!("SQLite WAL journal mode enabled successfully."),
            Err(e) => warn!("Failed to set PRAGMA journal_mode=WAL: {}", e),
        }

        info!("Database successfully initialized and optimized.");
    }

    pub async fn add_message(self, msg: &MessagePacket) {
        let Some(message_id) = msg.message_id else {
            error!("Cannot save message without Telegram message ID");
            return;
        };

        match sqlx::query(
            r#"
    INSERT INTO messages (
        chat_id, username, text, date, is_bot,
        is_read, msg_type, msg_name, id
    )
    VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9);
    "#,
        )
        .bind(msg.chat_id)
        .bind(&msg.username)
        .bind(&msg.text)
        .bind(&msg.time)
        .bind(msg.is_bot)
        .bind(msg.is_read)
        .bind(msg.msg_type.clone() as i64)
        .bind(msg.message_name.as_deref().unwrap_or(""))
        .bind(message_id)
        .execute(&self.pool)
        .await
        {
            Ok(_) => trace!("Message successfully written to database."),
            Err(e) => {
                error!(
                    "Failed to save message to DB for chat {}: {}",
                    msg.chat_id, e
                );
            }
        }
    }
    pub async fn edit_message(self, msg: types::Message) {
        match sqlx::query(
            "UPDATE messages
                SET text = ?
                WHERE chat_id = ? AND telegram_message_id = ?;",
        )
        .bind(msg.chat_id)
        .bind(msg.id)
        .execute(&self.pool)
        .await
        {
            Ok(_) => {
                trace!("Message successfully edited in database.");
            }
            Err(e) => {
                error!(
                    "Failed to edit message: {} in DB for chat {}: {}",
                    msg.id, msg.chat_id, e
                );
            }
        }
    }

    pub async fn _mark_readed_all(&self, chat_id: ChatId) {
        debug!("Marking all messages as read for ChatId: {}", chat_id.0);

        match sqlx::query(
            "UPDATE messages 
             SET is_read = 1 
             WHERE chat_id = ?1 AND is_read = 0",
        )
        .bind(chat_id.0)
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
                    "Failed to mark messages as read for chat {}: {}",
                    chat_id.0, e
                );
            }
        }
    }
    pub async fn add_chat(&self, chat_id: ChatId, title: &str) {
        if let Err(e) = sqlx::query(
            r#"
        INSERT INTO chats (chat_id, title)
        VALUES (?1, ?2)
        ON CONFLICT(chat_id) DO UPDATE SET
            title = CASE
                WHEN excluded.title IS NOT NULL
                     AND excluded.title != ''
                THEN excluded.title
                ELSE chats.title
            END;
        "#,
        )
        .bind(chat_id.0)
        .bind(title)
        .execute(&self.pool)
        .await
        {
            error!("Failed to add/update chat {}: {}", chat_id.0, e);
        }
    }
    pub async fn delete_message(&self, chat_id: i64, message_id: i32) {
        match sqlx::query(
            r#"
        DELETE FROM messages
        WHERE chat_id = ?1
          AND telegram_message_id = ?2;
        "#,
        )
        .bind(chat_id)
        .bind(message_id)
        .execute(&self.pool)
        .await
        {
            Ok(result) => {
                if result.rows_affected() > 0 {
                    info!("Deleted message {} from chat {}", message_id, chat_id);
                } else {
                    warn!("Message {} not found in chat {}", message_id, chat_id);
                }
            }
            Err(e) => {
                error!(
                    "Failed to delete message {} from chat {}: {}",
                    message_id, chat_id, e
                );
            }
        }
    }

    pub async fn load_chats(&self) -> Vec<Chat> {
        debug!("Loading all chat histories from database...");

        let rows: Vec<MessageRow> = match sqlx::query_as(
            r#"
        SELECT
            m.chat_id,
            m.username,
            m.text,
            m.date,
            m.is_bot,
            m.is_read,
            m.msg_type,
            COALESCE(m.msg_name, '') AS msg_name,
            m.telegram_message_id,
            COALESCE(c.title, '') AS chat_title
        FROM messages AS m
        LEFT JOIN chats AS c ON c.chat_id = m.chat_id
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

                debug!(
                    "Loaded chat: id={}, title='{}', displayed_name='{}'",
                    chat_id, title, name
                );

                let unreaded_count = rows.iter().filter(|row| !row.is_read).count() as u32;

                let messages: Vec<MessagePacket> = rows
                    .into_iter()
                    .map(|row| {
                        let msg_type = match row.msg_type {
                            0 => TEXT,
                            1 => IMAGE,
                            2 => STICKER,
                            3 => FILE,
                            _ => TEXT,
                        };

                        MessagePacket {
                            username: row.username,
                            text: row.text,
                            time: row.date,
                            is_bot: row.is_bot,
                            is_read: row.is_read,
                            msg_type,
                            chat_id: row.chat_id,
                            message_name: Some(row.msg_name),
                            chat_name: Some(row.chat_title),
                            message_id: Some(row.telegram_message_id),
                            account_id: row.account_id,
                        }
                    })
                    .collect();

                Chat {
                    chat_id: chat_id,
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

#[derive(Clone, sqlx::FromRow)]
struct MessageRow {
    chat_id: i64,
    username: String,
    text: String,
    date: String,
    is_bot: bool,
    is_read: bool,
    msg_type: i64,
    msg_name: String,
    telegram_message_id: i32,
    chat_title: String,
    account_id: i64,
}
