use async_trait::async_trait;
use teloxide::{
    RequestError,
    types::{Message, True},
};
use tokio::sync::mpsc::Sender;

use crate::logic::integrations::types::ReceivedMessage;
#[async_trait]
pub trait AccauntIntegration: Send + Sync {
    async fn send_message(&self, chat_id: i64, text: &str)
    -> anyhow::Result<Message, RequestError>;
    async fn send_document(
        &self,
        chat_id: i64,
        path: &str,
    ) -> anyhow::Result<Message, RequestError>;
    async fn send_photo(&self, chat_id: i64, path: &str) -> anyhow::Result<Message, RequestError>;
    async fn start(&self, tx: Sender<ReceivedMessage>, id: i64);
    async fn edit_message_text(
        &self,
        chat_id: i64,
        message_id: i32,
        text: &str,
    ) -> anyhow::Result<Message, RequestError>;
    async fn delete_message(&self, chat_id: i64, message_id: i32) -> anyhow::Result<True>;
    async fn get_info(&self) -> String;
}
