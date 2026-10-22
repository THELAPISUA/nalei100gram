use std::sync::Arc;

use crate::utils::audio::{AudioPlayer, SoundType};
use crate::utils::types::MsgType::*;
use crate::{logic::db::Db, utils::types::MessagePacket};
use log::{error, info};

use teloxide::dispatching::dialogue::GetChatId;
use teloxide::net::Download; // Важно для работы .download_file()
use teloxide::prelude::Requester;
use teloxide::prelude::*;
use teloxide::{Bot, types::InputFile};
use tokio::fs::{self, File as TokioFile};

pub(crate) async fn process_packet_message(
    packet: &MessagePacket,
    bot_for_gui: &Bot,
    db_for_gui: &Db,
    player: &Arc<AudioPlayer>,
) {
    match packet.msg_type {
        TEXT => {
            let r = bot_for_gui
                .send_message(packet.chat_id, packet.text.clone())
                .await;
            player.play(SoundType::Send);

            match r {
                Ok(m) => {
                    let chat_id = m.chat.id.0 as i64;
                    let new_packet = MessagePacket {
                        message_id: Some(m.id),
                        ..packet.clone()
                    };
                    db_for_gui.clone().add_message(&new_packet).await;
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
        FILE => {
            let r = bot_for_gui
                .send_document(packet.chat_id, InputFile::file(packet.text.clone()))
                .await;

            player.play(SoundType::Send);

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
        IMAGE => {
            let r = bot_for_gui
                .send_photo(packet.chat_id, InputFile::file(packet.text.clone()))
                .await;

            player.play(SoundType::Send);

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
        _ => {}
    }
}

pub async fn get_user_avatar(bot: Bot, msg: Message, is_user: bool) -> Option<String> {
    if is_user {
        let user = msg.from?;

        let photos = bot.get_user_profile_photos(user.id).await.ok()?;

        photos.photos.last()?.first().map(|p| p.file.id.clone())
    } else {
        let chat = bot.get_chat(msg.chat.id).await.ok()?;
        chat.photo.map(|photo| photo.small_file_id)
    }
}
pub async fn get_avatar(
    bot: Bot,
    msg: Message,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let chat = &msg.chat;

    let (file_id, owner_id) = if chat.is_private() {
        // Особистий чат: аватарка користувача, який написав повідомлення.
        let user = match msg.from.as_ref() {
            Some(user) => user,
            None => return Ok(()),
        };

        let photos = bot.get_user_profile_photos(user.id).await?;

        let file_id = match photos.photos.last().and_then(|photos| photos.last()) {
            Some(photo) => photo.file.id.clone(),
            None => return Ok(()),
        };

        (file_id, format!("{}", user.id.0))
    } else {
        // Група або супергрупа: аватарка саме чату.
        let chat = bot.get_chat(chat.id).await?;

        let photo = match chat.photo {
            Some(photo) => photo,
            None => return Ok(()),
        };

        (photo.big_file_id, format!("{}", msg.chat.id.0))
    };

    fs::create_dir_all("data/temp").await?;

    let path = format!("data/temp/{}.jpg", owner_id);
    let file_info = bot.get_file(file_id).await?;
    let mut destination_file = TokioFile::create(&path).await?;

    bot.download_file(&file_info.path, &mut destination_file)
        .await?;

    Ok(())
}
