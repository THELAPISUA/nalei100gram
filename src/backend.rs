use crate::audio::AudioPlayer;
use crate::audio::SoundType;
use crate::audio::SoundType::Send;
use crate::db::Db;
use crate::types::MessagePacket;
use crate::types::MsgType::FILE;
use crate::types::MsgType::STICKER;
use crate::types::MsgType::{IMAGE, TEXT};
use chrono::Local;
use eframe::egui::Shape::Path;
use log::{error, info, warn};

use std::fmt::format;
use std::fs;
use std::sync::Arc;
use teloxide::net::Download;
use teloxide::{dispatching::dialogue::GetChatId, prelude::*};
use tokio::fs::File as TokioFile;
use tokio::sync::mpsc::{Receiver, Sender};
pub struct Backend {
    tx_bot: Sender<MessagePacket>,
    rx_bot: Receiver<MessagePacket>,
    database: Db,
    player: Arc<AudioPlayer>,
}
impl Backend {
    pub fn new(
        tx_bot: Sender<MessagePacket>,
        rx_bot: Receiver<MessagePacket>,
        database: Db,
    ) -> Self {
        Self {
            tx_bot,
            rx_bot,
            database,
            player: Arc::new(AudioPlayer::init()),
        }
    }
    pub async fn tick(mut self) {
        #[cfg(target_os = "linux")]
        let bot = Bot::new("8297296496:AAHqAR8s3BEUX-mQ9KKo6U2Z6hBojSeTRGQ");

        #[cfg(target_os = "windows")]
        let bot = Bot::new("5649333069:AAH3I94QCOkQZaDQ8rWHvHJWscEe5v2ybYk");

        let bot_for_gui = bot.clone();
        let player = self.player.clone();
        let db_for_gui = self.database.clone();
        let mut rx_bot = self.rx_bot;

        tokio::spawn(async move {
            while let Some(packet) = rx_bot.recv().await {
                info!("Received packet for chat {}", packet.chat_id);
                let r = bot_for_gui
                    .send_message(packet.chat_id, packet.text.clone())
                    .await;

                self.player.play(Send);

                db_for_gui.clone().add_message(&packet).await;

                match r {
                    Ok(m) => {
                        let chat_id = m.chat.id.0 as i64;
                        info!(
                            "Successful send msg! chat_id:{}, msg:{}",
                            chat_id,
                            m.text().unwrap_or("occurred message")
                        );
                    }
                    Err(e) => {
                        error!("Failed to send message, err: {}", e);
                    }
                }
            }
        });
        let tx_bot_clone = self.tx_bot.clone();
        let database = self.database.clone();

        teloxide::repl(bot, move |bot: Bot, msg: Message| {
            let tx = tx_bot_clone.clone();
            let database = database.clone();
            let player = player.clone();
            async move {
                if let Some(text) = msg.text() {
                    let m = MessagePacket {
                        username: msg.chat.first_name().map(String::from),
                        text: text.to_string(),
                        chat_id: msg.chat_id().unwrap_or(ChatId(1)),
                        time: Local::now().format("%H:%M").to_string(),
                        msgtype: TEXT,
                        is_bot: false,
                        is_read: true,
                        msg_name: None,
                    };
                    let t = tx.send(m.clone()).await;

                    database.clone().add_message(&m).await;
                    player.play(SoundType::Message);

                    match t {
                        Ok(()) => {
                            log::info!("Seccesful send paket to ui!");
                        }
                        Err(e) => {
                            log::error!("Failed send packet to ui, err: {}", e);
                        }
                    }
                } else if let Some(imgs) = msg.photo() {
                    if let Err(e) = fs::create_dir_all("data/temp") {
                        error!("Failed to create 'data' directory: {}", e);
                        panic!("{}", e);
                    }
                    if let Some(best_photo) = imgs.iter().max_by_key(|p| p.width) {
                        let file_id = &best_photo.file.id;

                        info!("Успішно знайдено file_id: {}", file_id);

                        let file_info = bot.get_file(file_id.clone()).await?;

                        let path = format!("data/temp/{}.jpg", file_id);

                        let mut destination_file = TokioFile::create(path.clone()).await?;

                        bot.download_file(&file_info.path, &mut destination_file)
                            .await?;
                        let p = MessagePacket {
                            username: msg.chat.first_name().map(String::from),
                            text: path,
                            chat_id: msg.chat_id().unwrap_or(ChatId(1)),
                            time: Local::now().format("%H:%M").to_string(),
                            msgtype: IMAGE,
                            is_bot: false,
                            is_read: true,
                            msg_name: None,
                        };
                        let _ = tx.send(p.clone()).await;
                        database.clone().add_message(&p).await;
                        player.play(SoundType::Message);
                    } else {
                        warn!("Список PhotoSize порожній!");
                    }
                } else if let Some(stiker) = msg.sticker() {
                    if let Err(e) = fs::create_dir_all("data/temp") {
                        error!("Failed to create 'data' directory: {}", e);
                        panic!("{}", e);
                    }
                    if stiker.is_static() {
                        let file_id = &stiker.file.id;

                        let file_info = bot.get_file(file_id.clone()).await?;

                        let path = format!("data/temp/{}.webp", file_id);

                        let mut destination_file = TokioFile::create(path.clone()).await?;
                        bot.download_file(&file_info.path, &mut destination_file)
                            .await?;
                        let p = MessagePacket {
                            username: msg.chat.first_name().map(String::from),
                            text: path,
                            chat_id: msg.chat_id().unwrap_or(ChatId(1)),
                            time: Local::now().format("%H:%M").to_string(),
                            msgtype: STICKER,
                            is_bot: false,
                            is_read: true,
                            msg_name: None,
                        };
                        let _ = tx.send(p.clone()).await;
                        database.clone().add_message(&p).await;
                        player.play(SoundType::Message);
                    } else {
                        bot.delete_message(msg.chat.id, msg.id).await?;
                    }
                } else if let Some(doc) = msg.document() {
                    if let Err(e) = fs::create_dir_all("data/temp") {
                        error!("Failed to create 'data' directory: {}", e);
                        panic!("{}", e);
                    }
                    let file_id = &doc.file.id;

                    let file_info = bot.get_file(file_id.clone()).await?;
                    let file_format = format!("{}.dat", file_id);
                    let file_name = doc.file_name.clone().unwrap_or(file_format);
                    let path = format!("data/temp/{}", file_name);

                    let mut destination_file = TokioFile::create(path.clone()).await?;
                    bot.download_file(&file_info.path, &mut destination_file)
                        .await?;

                    let p = MessagePacket {
                        username: msg.chat.first_name().map(String::from),
                        text: path,
                        chat_id: msg.chat_id().unwrap_or(ChatId(1)),
                        time: Local::now().format("%H:%M").to_string(),
                        msgtype: FILE,
                        is_bot: false,
                        is_read: true,
                        msg_name: Some(file_name),
                    };
                    let _ = tx.send(p.clone()).await;
                    database.clone().add_message(&p).await;
                    player.play(SoundType::Message);
                } else {
                    bot.delete_message(msg.chat.id, msg.id).await?;
                }
                respond(())
            }
        })
        .await;
    }
}
