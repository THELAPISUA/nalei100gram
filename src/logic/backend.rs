use crate::logic::integrations::manager::manager::AccountManager;
use crate::logic::integrations::types::ActionPacket;
use crate::logic::integrations::types::MessageType::TEXT;
use crate::logic::{db::Db, integrations::types::ReceivedMessage};

use crate::logic::logic::process_packet_message;
use crate::utils::audio::{AudioPlayer, SoundType, set_max_vol};

use log::info;

use std::sync::Arc;

use tokio::sync::mpsc::{Receiver, Sender};
pub struct Backend {
    pub(crate) tx_bot: Sender<ActionPacket>,
    pub(crate) rx_bot: Receiver<ActionPacket>,
    pub(crate) database: Db,
    pub(crate) player: Arc<AudioPlayer>,
    pub(crate) reciver: Receiver<ReceivedMessage>,
    account_manager: Arc<AccountManager>,
}
impl Backend {
    pub fn new(
        tx_bot: Sender<ActionPacket>,
        rx_bot: Receiver<ActionPacket>,
        database: Db,
        account_manager: Arc<AccountManager>,
        reciver: Receiver<ReceivedMessage>,
    ) -> Self {
        Self {
            tx_bot,
            rx_bot,
            database,
            player: Arc::new(AudioPlayer::init()),
            account_manager: account_manager,
            reciver,
        }
    }
    pub async fn tick(self) {
        self.account_manager.start_all_accounts().await;
        let names = self.account_manager.get_info().await;

        for (name, id) in names {
            self.tx_bot
                .try_send(ActionPacket::AccountInfo(name, id))
                .unwrap();
        }
        let player = self.player.clone();
        let db_for_gui = self.database.clone();
        let mut rx_bot = self.rx_bot;

        tokio::spawn(async move {
            while let Some(packet) = rx_bot.recv().await {
                let db_for_gui = db_for_gui.clone();
                match packet {
                    ActionPacket::Send(pack) => {
                        process_packet_message(&pack, &self.account_manager, &db_for_gui, &player)
                            .await;
                        info!("Received packet for chat {}", pack.chat_id);
                    }
                    ActionPacket::Edit {
                        accaunt_id,
                        chat_id,
                        id,
                        new_text,
                    } => {
                        let r = self
                            .account_manager
                            .edit_message_text(accaunt_id, chat_id, id, &new_text)
                            .await;
                        if r.is_ok() {
                            db_for_gui
                                .edit_message(accaunt_id, chat_id, id, &new_text)
                                .await;
                        }
                    }
                    ActionPacket::Delete {
                        id,
                        chat_id,
                        account_id,
                    } => {
                        let r = self
                            .account_manager
                            .delete_message(account_id, chat_id, id)
                            .await;
                        if r.is_ok() {
                            db_for_gui.delete_message(account_id, chat_id, id).await;
                        }
                    }
                    _ => {}
                }
            }
        });
        let mut reciver = self.reciver;
        while let Some(packet) = reciver.recv().await {
            set_max_vol();
            self.player.play(SoundType::Message);
            match packet.t.clone() {
                TEXT { text } => {
                    let t = self.tx_bot.send(ActionPacket::Send(packet.clone())).await;

                    self.database
                        .add_chat(packet.account_id, packet.chat_id, &packet.username)
                        .await;
                    self.database.add_message(&packet).await;

                    match t {
                        Ok(()) => {
                            log::info!("Seccesful send paket to ui!");
                        }
                        Err(e) => {
                            log::error!("Failed send packet to ui, err: {}", e);
                        }
                    }
                }
                _ => {}
            }
        }

        // let tx_bot_clone = self.tx_bot.clone();
        // let database = self.database.clone();
        // let player = self.player.clone();
        // teloxide::repl(bot, move |bot: Bot, msg: Message| {
        //     let tx = tx_bot_clone.clone();
        //     let database = database.clone();
        //     let player = player.clone();

        //     async move {
        //
        //         } else if let Some(imgs) = msg.photo() {
        //             if let Err(e) = get_avatar(bot.clone(), msg.clone()).await {
        //                 eprintln!("Помилка завантаження аватарки: {e}");
        //             }

        //             if let Some(best_photo) = imgs.iter().max_by_key(|p| p.width) {
        //                 let file_id = &best_photo.file.id;

        //                 info!("Успішно знайдено file_id: {}", file_id);

        //                 let file_info = bot.get_file(file_id.clone()).await?;

        //                 let path = format!("data/temp/{}.jpg", file_id);

        //                 let mut destination_file = TokioFile::create(path.clone()).await?;

        //                 bot.download_file(&file_info.path, &mut destination_file)
        //                     .await?;
        //                 let p = MessagePacket {
        //                     username: load_name(&msg),
        //                     text: path,
        //                     chat_id: msg.chat_id().unwrap_or(ChatId(1)),
        //                     time: Local::now().format("%H:%M").to_string(),
        //                     msg_type: IMAGE,
        //                     is_bot: false,
        //                     is_read: true,
        //                     message_name: None,
        //                     message_id: Some(msg.id),

        //                     chat_name: Some(String::from(
        //                         msg.chat
        //                             .title()
        //                             .unwrap_or(msg.chat.first_name().unwrap_or("Unknown name")),
        //                     )),
        //                 };
        //                 let _ = tx.send(ActionPacket::Send(p.clone())).await;
        //                 database
        //                     .add_chat(
        //                         msg.chat.id,
        //                         msg.chat
        //                             .title()
        //                             .unwrap_or(msg.chat.first_name().unwrap_or("Unknown name")),
        //                     )
        //                     .await;
        //                 database.clone().add_message(&p).await;
        //                 set_max_vol();
        //                 player.play(SoundType::Message);
        //             } else {
        //                 warn!("Список PhotoSize порожній!");
        //             }
        //         } else if let Some(stiker) = msg.sticker() {
        //             if let Err(e) = get_avatar(bot.clone(), msg.clone()).await {
        //                 eprintln!("Помилка завантаження аватарки: {e}");
        //             }

        //             if stiker.is_static() {
        //                 let file_id = &stiker.file.id;

        //                 let file_info = bot.get_file(file_id.clone()).await?;

        //                 let path = format!("data/temp/{}.webp", file_id);

        //                 let mut destination_file = TokioFile::create(path.clone()).await?;
        //                 bot.download_file(&file_info.path, &mut destination_file)
        //                     .await?;
        //                 let p = MessagePacket {
        //                     username: load_name(&msg),
        //                     text: path,
        //                     chat_id: msg.chat_id().unwrap_or(ChatId(1)),
        //                     time: Local::now().format("%H:%M").to_string(),
        //                     msg_type: STICKER,
        //                     is_bot: false,
        //                     is_read: true,
        //                     message_name: None,
        //                     message_id: Some(msg.id),

        //                     chat_name: Some(String::from(
        //                         msg.chat
        //                             .title()
        //                             .unwrap_or(msg.chat.first_name().unwrap_or("Unknown name")),
        //                     )),
        //                 };
        //                 let _ = tx.send(ActionPacket::Send(p.clone())).await;
        //                 database
        //                     .add_chat(
        //                         msg.chat.id,
        //                         msg.chat
        //                             .title()
        //                             .unwrap_or(msg.chat.first_name().unwrap_or("Unknown name")),
        //                     )
        //                     .await;
        //                 database.clone().add_message(&p).await;
        //                 set_max_vol();
        //                 player.play(SoundType::Message);
        //             } else {
        //                 bot.delete_message(msg.chat.id, msg.id).await?;
        //             }
        //         } else if let Some(doc) = msg.document() {
        //             if let Err(e) = get_avatar(bot.clone(), msg.clone()).await {
        //                 eprintln!("Помилка завантаження аватарки: {e}");
        //             }
        //             let file_id = &doc.file.id;

        //             let file_info = bot.get_file(file_id.clone()).await?;
        //             let file_format = format!("{}.dat", file_id);
        //             let file_name = doc.file_name.clone().unwrap_or(file_format);
        //             let path = format!("data/temp/{}", file_name);

        //             let mut destination_file = TokioFile::create(path.clone()).await?;
        //             bot.download_file(&file_info.path, &mut destination_file)
        //                 .await?;

        //             let p = MessagePacket {
        //                 username: load_name(&msg),
        //                 text: path,
        //                 chat_id: msg.chat_id().unwrap_or(ChatId(1)),
        //                 time: Local::now().format("%H:%M").to_string(),
        //                 msg_type: FILE,
        //                 is_bot: false,
        //                 is_read: true,
        //                 message_name: Some(file_name),
        //                 message_id: Some(msg.id),
        //                 chat_name: Some(String::from(
        //                     msg.chat
        //                         .title()
        //                         .unwrap_or(msg.chat.first_name().unwrap_or("Unknown name")),
        //                 )),
        //             };
        //             let _ = tx.send(ActionPacket::Send(p.clone())).await;
        //             database
        //                 .add_chat(
        //                     msg.chat.id,
        //                     msg.chat
        //                         .title()
        //                         .unwrap_or(msg.chat.first_name().unwrap_or("Unknown name")),
        //                 )
        //                 .await;
        //             database.clone().add_message(&p).await;
        //             set_max_vol();
        //             player.play(SoundType::Message);
        //         } else {
        //             bot.delete_message(msg.chat.id, msg.id).await?;
        //         }
        //         respond(())
        //     }
        // })
        // .await;
    }
}
