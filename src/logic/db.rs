use crate::utils::types::{
    Chat, MessagePacket,
    MsgType::{FILE, IMAGE, STICKER, TEXT},
};

use log::{debug, error, info, trace, warn};
use sqlx::{
    Pool, Sqlite,
    sqlite::{SqliteConnectOptions, SqlitePool},
};
use std::collections::HashMap;
use std::fs;
use teloxide::types::ChatId;

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
                chat_type TEXT,
                title TEXT,
                created_at INTEGER DEFAULT (strftime('%s','now'))
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
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                chat_id INTEGER NOT NULL,
                username TEXT NOT NULL,
                text TEXT NOT NULL,
                date TEXT NOT NULL,
                is_bot INTEGER NOT NULL DEFAULT 0,
                is_read INTEGER NOT NULL DEFAULT 0,
                msg_type INTEGER NOT NULL,
                msg_name TEXT
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
        debug!(
            "Saving message to database. ChatId: {}, Username: {}",
            msg.chat_id,
            msg.username.clone().unwrap_or(String::from("You"))
        );
        let msgtype = match msg.msg_type {
            TEXT => 0,
            IMAGE => 1,
            STICKER => 2,
            FILE => 3,
            _ => -1,
        };

        match sqlx::query(
            "INSERT INTO messages (chat_id, username, text, date, is_bot, is_read, msg_type, msg_name)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        )
        .bind(&msg.chat_id.0)
        .bind(&msg.username)
        .bind(&msg.text)
        .bind(&msg.time)
        .bind(msg.is_bot as i64)
        .bind(msg.is_read as i64)
        .bind(msgtype as i64)
        .bind(msg.msg_name.clone().unwrap_or(String::new()))
        .execute(&self.pool)
        .await
        {
            Ok(_) => {
                trace!("Message successfully written to database.");
            }
            Err(e) => {
                error!(
                    "Failed to save message to DB for chat {}: {}",
                    msg.chat_id.0, e
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

    pub async fn load_chats(&self) -> Vec<Chat> {
        debug!("Loading all chat histories from database...");

        let rows: Vec<MessageRow> = match sqlx::query_as(
            "SELECT chat_id, username, text, date, is_bot, is_read, msg_type, msg_name
         FROM messages
         ORDER BY chat_id ASC, id ASC",
        )
        .fetch_all(&self.pool)
        .await
        {
            Ok(r) => r,
            Err(e) => {
                error!("Failed to fetch messages from database: {}", e);
                return Vec::new();
            }
        };

        let mut chats: HashMap<i64, Vec<MessagePacket>> = HashMap::new();
        let rows_count = rows.len();
        let mut count: u32 = 0;
        for r in rows {
            if !r.is_read {
                count += 1;
            }

            let msgtype = match r.msg_type {
                0 => TEXT,
                1 => IMAGE,
                2 => STICKER,
                3 => FILE,
                _ => TEXT,
            };

            chats.entry(r.chat_id).or_default().push(MessagePacket {
                username: Some(r.username),
                text: r.text,
                time: r.date,
                is_bot: r.is_bot,
                is_read: r.is_read,
                msg_type: msgtype,
                chat_id: ChatId(r.chat_id),
                msg_name: Some(r.msg_name),
            });
        }

        let formatted_chats: Vec<Chat> = chats
            .into_iter()
            .map(|(chat_id, messages)| Chat {
                chat_id: ChatId(chat_id),
                messages,
                unreaded_count: count,
            })
            .collect();

        info!(
            "Successfully loaded {} messages, structured into {} chat objects.",
            rows_count,
            formatted_chats.len()
        );

        formatted_chats
    }
}

#[derive(sqlx::FromRow)]
struct MessageRow {
    chat_id: i64,
    username: String,
    text: String,
    date: String,
    is_bot: bool,
    is_read: bool,
    msg_type: i64,
    msg_name: String,
}
