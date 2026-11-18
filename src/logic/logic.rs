use std::sync::{Arc, Mutex};

use crate::logic::db::Db;
use crate::logic::integrations::manager::manager::AccountManager;
use crate::logic::integrations::types::{MessageType::*, ReceivedMessage};
#[cfg(not(debug_assertions))]
use crate::utils::audio::set_max_vol;
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
            if packet.chat_id != 0 {
                let r = account_manager
                    .send_message(packet.account_id, packet.chat_id, &text)
                    .await;
                match r {
                    Ok(m) => {
                        db_for_gui.clone().add_message(&packet).await;
                        info!(
                            "Successful send msg! chat_id:{}, msg:{}",
                            packet.chat_id,
                            m.text().unwrap_or("occurred message")
                        );
                    }
                    Err(e) => {
                        error!("Failed to send message Error:{}", e);
                    }
                }
            } else {
                let new_packet = ReceivedMessage {
                    account_id: -1,
                    ..packet.clone()
                };
                db_for_gui.clone().add_message(&new_packet).await;
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
            if packet.chat_id != 0 {
                let r = account_manager
                    .send_document(packet.account_id, packet.chat_id, &path)
                    .await;

                match r {
                    Ok(m) => {
                        db_for_gui.clone().add_message(&packet).await;
                        info!(
                            "Successful send msg! chat_id:{}, msg:{}",
                            packet.chat_id,
                            m.text().unwrap_or("occurred message")
                        );
                    }
                    Err(e) => {
                        error!("Failed to send message Error:{}", e);
                    }
                }
            } else {
                let new_packet = ReceivedMessage {
                    account_id: -1,
                    ..packet.clone()
                };
                db_for_gui.clone().add_message(&new_packet).await;
            }
        }

        PHOTO {
            ref path,
            caption: None,
        } => {
            if packet.chat_id != 0 {
                let r = account_manager
                    .send_photo(packet.account_id, packet.chat_id, &path)
                    .await;

                match r {
                    Ok(m) => {
                        db_for_gui.clone().add_message(&packet).await;
                        info!(
                            "Successful send msg! chat_id:{}, msg:{}",
                            packet.chat_id,
                            m.text().unwrap_or("occurred message")
                        );
                    }
                    Err(e) => {
                        error!("Failed to send message Error:{}", e);
                    }
                }
            } else {
                let new_packet = ReceivedMessage {
                    account_id: -1,
                    ..packet.clone()
                };
                db_for_gui.clone().add_message(&new_packet).await;
            }
        }
        _ => {}
    }
    player.lock().unwrap().play_msg(SoundType::Send);
}
