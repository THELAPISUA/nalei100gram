use crate::logic::db::Db;
use crate::logic::logic::process_packet_message;
use crate::utils::audio::AudioPlayer;
use crate::utils::audio::SoundType;
use crate::utils::audio::set_max_vol;
use crate::utils::types::ActionPacket;
use crate::utils::types::MessagePacket;
use crate::utils::types::MsgType::FILE;
use crate::utils::types::MsgType::STICKER;
use crate::utils::types::MsgType::{IMAGE, TEXT};
use chrono::Local;

use log::{error, info, warn};

use std::fs;
use std::sync::Arc;
use teloxide::net::Download;
use teloxide::{dispatching::dialogue::GetChatId, prelude::*};
use tokio::fs::File as TokioFile;
use tokio::sync::mpsc::{Receiver, Sender};
pub struct Backend {
    pub(crate) tx_bot: Sender<ActionPacket>,
    pub(crate) rx_bot: Receiver<ActionPacket>,
    pub(crate) database: Db,
    pub(crate) player: Arc<AudioPlayer>,
}
impl Backend {
    pub fn new(tx_bot: Sender<ActionPacket>, rx_bot: Receiver<ActionPacket>, database: Db) -> Self {
        Self {
            tx_bot,
            rx_bot,
            database,
            player: Arc::new(AudioPlayer::init()),
        }
    }
    pub async fn tick(self) {
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
                let db_for_gui = db_for_gui.clone();
                match packet {
                    ActionPacket::Send(pack) => {
                        process_packet_message(&pack, &bot_for_gui, &db_for_gui, &player).await;
                        info!("Received packet for chat {}", pack.chat_id);
                    }
                    ActionPacket::Edit(msg) => {
                        let r = bot_for_gui
                            .edit_message_text(
                                msg.chat_id.clone(),
                                msg.id.clone(),
                                msg.new_text.clone().unwrap().as_str(),
                            )
                            .await;
                        if r.is_ok() {
                            db_for_gui.edit_message(msg.clone()).await;
                        }
                    }
                    _ => {}
                }
            }
        });
        let tx_bot_clone = self.tx_bot.clone();
        let database = self.database.clone();
        let player = self.player.clone();
        teloxide::repl(bot, move |bot: Bot, msg: Message| {
            let tx = tx_bot_clone.clone();
            let database = database.clone();
            let player = player.clone();
            async move {
                if let Some(text) = msg.text() {
                    let m = MessagePacket {
                        username: load_name(&msg),
                        text: text.to_string(),
                        chat_id: msg.chat_id().unwrap_or(ChatId(1)),
                        time: Local::now().format("%H:%M").to_string(),
                        msg_type: TEXT,
                        is_bot: false,
                        is_read: true,
                        message_name: None,
                        message_id: Some(msg.id),

                        chat_name: Some(String::from(
                            msg.chat
                                .title()
                                .unwrap_or(msg.chat.first_name().unwrap_or("Unknown name")),
                        )),
                    };
                    let t = tx.send(ActionPacket::Send(m.clone())).await;

                    database
                        .add_chat(
                            msg.chat.id,
                            msg.chat
                                .title()
                                .unwrap_or(msg.chat.first_name().unwrap_or("Unknown name")),
                        )
                        .await;
                    database.add_message(&m).await;
                    set_max_vol();
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
                            username: load_name(&msg),
                            text: path,
                            chat_id: msg.chat_id().unwrap_or(ChatId(1)),
                            time: Local::now().format("%H:%M").to_string(),
                            msg_type: IMAGE,
                            is_bot: false,
                            is_read: true,
                            message_name: None,
                            message_id: Some(msg.id),

                            chat_name: Some(String::from(
                                msg.chat
                                    .title()
                                    .unwrap_or(msg.chat.first_name().unwrap_or("Unknown name")),
                            )),
                        };
                        let _ = tx.send(ActionPacket::Send(p.clone())).await;
                        database.clone().add_message(&p).await;
                        set_max_vol();
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
                            username: load_name(&msg),
                            text: path,
                            chat_id: msg.chat_id().unwrap_or(ChatId(1)),
                            time: Local::now().format("%H:%M").to_string(),
                            msg_type: STICKER,
                            is_bot: false,
                            is_read: true,
                            message_name: None,
                            message_id: Some(msg.id),

                            chat_name: Some(String::from(
                                msg.chat
                                    .title()
                                    .unwrap_or(msg.chat.first_name().unwrap_or("Unknown name")),
                            )),
                        };
                        let _ = tx.send(ActionPacket::Send(p.clone())).await;
                        database.clone().add_message(&p).await;
                        set_max_vol();
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
                        username: load_name(&msg),
                        text: path,
                        chat_id: msg.chat_id().unwrap_or(ChatId(1)),
                        time: Local::now().format("%H:%M").to_string(),
                        msg_type: FILE,
                        is_bot: false,
                        is_read: true,
                        message_name: Some(file_name),
                        message_id: Some(msg.id),
                        chat_name: Some(String::from(
                            msg.chat
                                .title()
                                .unwrap_or(msg.chat.first_name().unwrap_or("Unknown name")),
                        )),
                    };
                    let _ = tx.send(ActionPacket::Send(p.clone())).await;
                    database.clone().add_message(&p).await;
                    set_max_vol();
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
fn load_name(msg: &Message) -> String {
    let user = msg.clone().from;

    let name = if user.is_some() {
        user.unwrap().full_name()
    } else {
        msg.chat
            .first_name()
            .map(String::from)
            .unwrap_or(msg.chat.title().unwrap_or("Ошибка имени").to_string())
    };
    name
}
