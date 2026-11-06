use std::sync::{Arc, Mutex};

use crate::logic::db::Db;
use crate::logic::integrations::manager::manager::AccountManager;
use crate::logic::integrations::types::{MessageType::*, ReceivedMessage};
use crate::utils::audio::{AudioPlayer, SoundType};

use log::{error, info};

pub(crate) async fn process_packet_message(
    packet: &ReceivedMessage,
    account_manager: &Arc<AccountManager>,
    db_for_gui: &Db,
    player: &Arc<Mutex<AudioPlayer>>,
) {
    #[cfg(not(debug_assertions))]
    set_max_vol();
    match packet.t {
        TEXT { ref text } => {
            let r = account_manager
                .send_message(packet.account_id, packet.chat_id, &text)
                .await;

            match r {
                Ok(m) => {
                    let chat_id = m.chat.id.0 as i64;
                    let new_packet = ReceivedMessage {
                        id: packet.id,
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
        AUDIO {
            ref path,
            name: _,
            caption: _,
        }
        | FILE {
            ref path,
            name: _,
            caption: _,
        } => {
            let r = account_manager
                .send_document(packet.account_id, packet.chat_id, &path)
                .await;

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

        PHOTO {
            ref path,
            caption: None,
        } => {
            let r = account_manager
                .send_photo(packet.account_id, packet.chat_id, &path)
                .await;

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
    player.lock().unwrap().play_msg(SoundType::Send);
}
