use anyhow::Error;
use teloxide::{
    RequestError,
    types::{Message, True},
};
use tokio::sync::mpsc::{self, Receiver, Sender};

use crate::logic::integrations::{manager::account::Account, types::ReceivedMessage};

pub struct AccountManager {
    accounts: Vec<Account>,
    tx: Sender<ReceivedMessage>,
}
impl AccountManager {
    pub fn new() -> (Receiver<ReceivedMessage>, Self) {
        let (tx, rx) = mpsc::channel(100);
        (
            rx,
            Self {
                accounts: Vec::new(),
                tx,
            },
        )
    }
    pub async fn start_account(&self, account_id: i64) {
        self.accounts
            .iter()
            .find(|account| account.id == account_id)
            .expect("Account not found!")
            .integration
            .start(self.tx.clone(), account_id)
            .await;
    }
    pub async fn start_all_accounts(&self) {
        for a in &self.accounts {
            a.integration.start(self.tx.clone(), a.id).await;
        }
    }
    pub async fn edit_message_text(
        &self,
        account_id: i64,
        chat_id: i64,
        message_id: i32,
        text: &str,
    ) -> anyhow::Result<Message, RequestError> {
        let r = self
            .accounts
            .iter()
            .find(|account| account.id == account_id)
            .expect("Account not found!")
            .integration
            .edit_message_text(chat_id, message_id, text)
            .await;
        r
    }
    pub fn add_account(&mut self, account: Account) {
        self.accounts.push(account);
    }
    pub async fn send_message(
        &self,
        account_id: i64,
        chat_id: i64,
        msg: &str,
    ) -> anyhow::Result<Message, RequestError> {
        let r = self
            .accounts
            .iter()
            .find(|account| account.id == account_id)
            .expect("Account not found!")
            .integration
            .send_message(chat_id, msg)
            .await;
        r
    }
    pub async fn delete_message(
        &self,
        account_id: i64,
        chat_id: i64,
        msg_id: i32,
    ) -> Result<True, Error> {
        let r = self
            .accounts
            .iter()
            .find(|account| account.id == account_id)
            .expect("Account not found!")
            .integration
            .delete_message(chat_id, msg_id)
            .await;
        r
    }
    pub async fn send_photo(
        &self,
        account_id: i64,
        chat_id: i64,
        path: &str,
    ) -> anyhow::Result<Message, RequestError> {
        let r = self
            .accounts
            .iter()
            .find(|account| account.id == account_id)
            .expect("Account not found!")
            .integration
            .send_photo(chat_id, path)
            .await;
        r
    }
    pub async fn send_document(
        &self,
        account_id: i64,
        chat_id: i64,
        path: &str,
    ) -> anyhow::Result<Message, RequestError> {
        let r = self
            .accounts
            .iter()
            .find(|account| account.id == account_id)
            .expect("Account not found!")
            .integration
            .send_document(chat_id, path)
            .await;
        r
    }
    pub async fn get_info(&self) -> Vec<(String, i64)> {
        let mut result = Vec::new();

        for a in &self.accounts {
            result.push((a.integration.get_info().await, a.id));
        }

        result
    }
}
