use std::sync::Arc;

use crate::logic::integrations::manager::manager::AccountManager;
use crate::utils::audio::{AudioPlayer, SoundType};
use crate::utils::types::MsgType::*;
use crate::{logic::db::Db, utils::types::MessagePacket};

use log::{error, info};

pub(crate) async fn process_packet_message(
    packet: &MessagePacket,
    account_manager: &Arc<AccountManager>,
    db_for_gui: &Db,
    player: &Arc<AudioPlayer>,
) {
    match packet.msg_type {
        TEXT => {
            let r = account_manager
                .send_message(packet.account_id, packet.chat_id, packet.text.as_str())
                .await;
            player.play(SoundType::Send);

            match r {
                Ok(m) => {
                    let chat_id = m.chat.id.0 as i64;
                    let new_packet = MessagePacket {
                        message_id: Some(m.id.0),
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
                    error!("Failed to send message Error:{}", e);
                }
            }
        }
        FILE => {
            let r = account_manager
                .send_document(packet.account_id, packet.chat_id, packet.text.as_str())
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
                    error!("Failed to send message Error:{}", e);
                }
            }
        }
        IMAGE => {
            let r = account_manager
                .send_photo(packet.account_id, packet.chat_id, packet.text.as_str())
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
                    error!("Failed to send message Error:{}", e);
                }
            }
        }
        _ => {}
    }
}
