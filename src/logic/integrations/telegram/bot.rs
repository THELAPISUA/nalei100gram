use async_trait::async_trait;

use log::error;
use teloxide::RequestError;
use teloxide::net::Download;
use teloxide::{
    Bot,
    requests::Requester,
    respond,
    types::{ChatId, InputFile, Message, MessageId, True, UserId},
};
use tokio::{fs, sync::mpsc::Sender};

use crate::logic::integrations::{traits::AccauntIntegration, types::ReceivedMessage};

pub struct TelegramBot {
    bot: Bot,
}
impl TelegramBot {
    pub fn new(token: &str) -> Self {
        Self {
            bot: Bot::new(token),
        }
    }
}

#[async_trait]
impl AccauntIntegration for TelegramBot {
    async fn start(&self, tx: Sender<ReceivedMessage>, id: i64) {
        let bot = self.bot.clone();
        let account_id = id;
        tokio::spawn(async move {
            teloxide::repl(bot, move |_: Bot, msg: Message| {
                let tx = tx.clone();

                async move {
                    if let Some(text) = msg.text() {
                        let m = ReceivedMessage {
                            username: msg.chat.first_name().unwrap_or("Err").to_string(),
                            id: msg.id.0,
                            t: crate::logic::integrations::types::MessageType::TEXT {
                                text: text.to_string(),
                            },
                            date: msg.date.format("%H:%M").to_string(),
                            chat_id: msg.chat.id.0,
                            readed: true,
                            sender_is_you: false,
                            account_id,
                        };
                        let t = tx.send(m).await;
                        match t {
                            Ok(()) => {
                                log::info!("Seccesful send paket to ui!");
                            }
                            Err(e) => {
                                log::error!("Failed send packet to ui, err: {}", e);
                            }
                        }
                    }
                    respond(())
                }
            })
            .await;
        });
    }
    async fn send_message(
        &self,
        chat_id: i64,
        text: &str,
    ) -> anyhow::Result<Message, RequestError> {
        let r = self.bot.send_message(ChatId(chat_id), text).await;
        r
    }
    async fn send_document(
        &self,
        chat_id: i64,
        path: &str,
    ) -> anyhow::Result<Message, RequestError> {
        let r = self
            .bot
            .send_document(ChatId(chat_id), InputFile::file(path))
            .await;
        r
    }
    async fn send_photo(&self, chat_id: i64, path: &str) -> anyhow::Result<Message, RequestError> {
        let r = self
            .bot
            .send_photo(ChatId(chat_id), InputFile::file(path))
            .await;
        r
    }
    async fn edit_message_text(
        &self,
        chat_id: i64,
        message_id: i32,
        text: &str,
    ) -> anyhow::Result<Message, RequestError> {
        let r = self
            .bot
            .edit_message_text(ChatId(chat_id), MessageId(message_id), text)
            .await;
        r
    }
    async fn delete_message(&self, chat_id: i64, message_id: i32) -> anyhow::Result<True> {
        let r = self
            .bot
            .delete_message(ChatId(chat_id), MessageId(message_id))
            .await
            .unwrap();

        Ok(r)
    }
    async fn get_info(&self) -> String {
        let me = self.bot.get_me().await.unwrap();
        if let Err(e) = get_avatar_by_user_id(self.bot.clone(), me.id.0).await {
            error!("Помилка завантаження аватарки: {e}");
        }
        me.full_name()
    }
}

pub async fn get_avatar_by_user_id(
    bot: Bot,
    id: u64,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let photos = bot.get_user_profile_photos(UserId(id)).await?;

    let file_id = match photos.photos.last().and_then(|photos| photos.last()) {
        Some(photo) => photo.file.id.clone(),
        None => return Ok(()),
    };

    let file_info = bot.get_file(file_id).await?;
    fs::create_dir_all("data/temp").await?;

    let temp_path = format!("data/temp/{}_original", "my");
    let output_path = format!("data/temp/{}.png", "my");

    let mut destination_file = fs::File::create(&temp_path).await?;

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

    let mut destination_file = fs::File::create(&temp_path).await?;

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
