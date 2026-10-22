use std::sync::Arc;

use crate::utils::audio::{AudioPlayer, SoundType};
use crate::utils::types::MsgType::*;
use crate::{logic::db::Db, utils::types::MessagePacket};
use eframe::egui::WidgetType::Image;
use eframe::wgpu::naga::back::spv::Capability::ImageBuffer;
use image::RgbaImage;
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

    let file_info = bot.get_file(file_id).await?;
    fs::create_dir_all("data/temp").await?;

    let temp_path = format!("data/temp/{}_original", owner_id);
    let output_path = format!("data/temp/{}.png", owner_id);

    let mut destination_file = TokioFile::create(&temp_path).await?;

    bot.download_file(&file_info.path, &mut destination_file)
        .await?;

    drop(destination_file);

    let bytes = fs::read(&temp_path).await?;

    let mut img = image::load_from_memory(&bytes)?.to_rgba8();

    make_circle(&mut img);

    img.save(&output_path)?;

    fs::remove_file(&temp_path).await?;

    Ok(())
}
fn make_circle(image: &mut image::RgbaImage) {
    let (w, h) = image.dimensions();
    let radius = w.min(h) as f32 / 2.0;
    let cx = w as f32 / 2.0;
    let cy = h as f32 / 2.0;

    for y in 0..h {
        for x in 0..w {
            let dx = x as f32 + 0.5 - cx;
            let dy = y as f32 + 0.5 - cy;

            if dx * dx + dy * dy > radius * radius {
                image.get_pixel_mut(x, y).0[3] = 0;
            }
        }
    }
}
