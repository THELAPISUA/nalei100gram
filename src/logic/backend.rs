use crate::logic::integrations::manager::manager::AccountManager;
use crate::logic::integrations::types::ActionPacket;

use crate::logic::{db::Db, integrations::types::ReceivedMessage};

use crate::logic::logic::process_packet_message;
use crate::utils::audio::{AudioPlayer, SoundType};

use log::info;

use std::sync::{Arc, Mutex};

use tokio::sync::mpsc::{Receiver, Sender};
pub struct Backend {
    pub(crate) tx_bot: Sender<ActionPacket>,
    pub(crate) rx_bot: Receiver<ActionPacket>,
    pub(crate) database: Db,
    pub(crate) player: Arc<Mutex<AudioPlayer>>,
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
        player: Arc<Mutex<AudioPlayer>>,
    ) -> Self {
        Self {
            tx_bot,
            rx_bot,
            database,
            player,
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
                        info!("Received edit request for chat {}", chat_id);
                        if chat_id != 0 {
                            let r = self
                                .account_manager
                                .edit_message_text(accaunt_id, chat_id, id, &new_text)
                                .await;
                            match r {
                                Ok(_) => {
                                    db_for_gui
                                        .edit_message(accaunt_id, chat_id, id, &new_text)
                                        .await;
                                }
                                Err(e) => {
                                    info!(
                                        "Cannot to edit massage id:{}, chat_id:{}, accaunt_id:{} Error:{}",
                                        id, chat_id, accaunt_id, e
                                    )
                                }
                            }
                        } else {
                            db_for_gui.edit_message(-1, chat_id, id, &new_text).await;
                        }
                    }
                    ActionPacket::Delete {
                        id,
                        chat_id,
                        account_id,
                    } => {
                        if chat_id != 0 {
                            let r = self
                                .account_manager
                                .delete_message(account_id, chat_id, id)
                                .await;
                            if r.is_ok() {
                                db_for_gui.delete_message(account_id, chat_id, id).await;
                            }
                        } else {
                            db_for_gui.delete_message(-1, chat_id, id).await;
                        }
                    }
                    _ => {}
                }
            }
        });
        let mut reciver = self.reciver;
        while let Some(packet) = reciver.recv().await {
            #[cfg(not(debug_assertions))]
            set_max_vol();
            self.player.lock().unwrap().play_msg(SoundType::Message);
            let t = self.tx_bot.send(ActionPacket::Send(packet.clone())).await;
            let name = if packet.chat_name == "" {
                packet.username.clone()
            } else {
                packet.chat_name.clone()
            };
            self.database
                .add_chat(packet.account_id, packet.chat_id, &name)
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
    }
}
