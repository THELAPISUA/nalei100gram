use log::error;
use rodio::{Decoder, DeviceSinkBuilder, MixerDeviceSink, Player};
use std::io::Cursor;

pub enum SoundType {
    Send,
    Message,
}
pub struct AudioPlayer {
    _stream_handle: MixerDeviceSink,
    player: Player,
    sound_send: &'static [u8],
    sound_message: &'static [u8],
}

impl AudioPlayer {
    pub fn init() -> Self {
        let _stream_handle =
            DeviceSinkBuilder::open_default_sink().expect("Не вдалося відкрити аудіо-пристрій");
        let player = Player::connect_new(&_stream_handle.mixer());

        let sound_send = include_bytes!("../assets/send.mp3");
        let sound_message = include_bytes!("../assets/message.mp3");
        Self {
            _stream_handle,
            player,
            sound_send,
            sound_message,
        }
    }

    pub fn play(&self, sound: SoundType) {
        let bytes = match sound {
            SoundType::Send => self.sound_send,
            SoundType::Message => self.sound_message,
        };

        let cursor = Cursor::new(bytes);

        match Decoder::new(cursor) {
            Ok(source) => {
                self.player.append(source);
            }
            Err(err) => {
                error!("Помилка декодування вбудованого аудіо: {:?}", err);
            }
        }
    }
}
