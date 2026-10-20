use std::sync::Arc;

use crate::utils::audio::{AudioPlayer, SoundType};
use crate::utils::types::MsgType::*;
use crate::{
    logic::{backend::Backend, db::Db},
    utils::types::MessagePacket,
};
use log::{error, info};
use teloxide::prelude::Requester;
use teloxide::{Bot, types::InputFile};

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
