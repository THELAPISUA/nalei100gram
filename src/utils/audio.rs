use log::error;
use rodio::{Decoder, DeviceSinkBuilder, MixerDeviceSink, Player, Source};
use std::{collections::HashMap, fs::File, io::Cursor, time::Duration};
use volumecontrol::AudioDevice;
pub enum SoundType {
    Send,
    Message,
}
pub struct AudioPlayer {
    player_msg: Player,
    player_audio: Player,
    sound_send: &'static [u8],
    sound_message: &'static [u8],
    _stream_handle: MixerDeviceSink,
    states: HashMap<String, AudioState>,
    active_path: Option<String>,
}

#[derive(Clone, Debug)]
pub struct AudioState {
    pub playing: bool,
    pub position: Duration,
    pub duration: Duration,
}

impl AudioPlayer {
    pub fn init() -> Self {
        let _stream_handle: MixerDeviceSink =
            DeviceSinkBuilder::open_default_sink().expect("Не вдалося відкрити аудіо-пристрій");
        let player_msg = Player::connect_new(&_stream_handle.mixer());
        let player_audio = Player::connect_new(&_stream_handle.mixer());

        let sound_send = include_bytes!("../../assets/send.mp3");
        let sound_message = include_bytes!("../../assets/message.mp3");
        Self {
            player_msg,
            sound_send,
            player_audio,
            sound_message,
            _stream_handle,
            states: HashMap::new(),
            active_path: None,
        }
    }
    pub fn tick(&mut self) {
        let Some(path) = &self.active_path else {
            return;
        };

        let position = self.player_audio.get_pos();

        if let Some(state) = self.states.get_mut(path) {
            state.position = position;
            state.playing = !self.player_audio.is_paused() && !self.player_audio.empty();
        }
    }
    pub fn play_msg(&self, sound: SoundType) {
        let bytes = match sound {
            SoundType::Send => self.sound_send,
            SoundType::Message => self.sound_message,
        };

        let cursor = Cursor::new(bytes);

        match Decoder::new(cursor) {
            Ok(source) => {
                self.player_msg.append(source);
            }
            Err(err) => {
                error!("Помилка декодування вбудованого аудіо: {:?}", err);
            }
        }
    }
    pub fn play_sound(&mut self, path: &str) {
        self.player_audio.stop();
        self.player_audio.clear();
        self.states.clear();

        self.active_path = Some(path.to_string());

        let sound_file = match File::open(path) {
            Ok(file) => file,
            Err(err) => {
                error!("Помилка відкриття аудіо: {:?}", err);
                self.active_path = None;
                return;
            }
        };

        match Decoder::try_from(sound_file) {
            Ok(source) => {
                let duration = source.total_duration().unwrap_or(Duration::ZERO);

                self.states.insert(
                    path.to_string(),
                    AudioState {
                        playing: true,
                        position: Duration::ZERO,
                        duration,
                    },
                );

                self.player_audio.append(source);
                self.player_audio.play();
            }
            Err(err) => {
                error!("Помилка декодування аудіо: {:?}", err);
                self.active_path = None;
            }
        }
    }
    pub fn seek(&self, position: Duration) {
        if let Err(err) = self.player_audio.try_seek(position) {
            error!("Помилка перемотування: {:?}", err);
        }
    }
    pub fn toggle(&self) {
        if self.player_audio.is_paused() {
            self.player_audio.play();
        } else {
            self.player_audio.pause();
        }
    }
    pub fn get_state(&self, path: &String) -> Option<&AudioState> {
        self.states.get(path)
    }
}

pub fn set_max_vol() {
    let device = AudioDevice::from_default().unwrap();

    let _ = device.set_vol(100);
    let _ = device.set_mute(false);
}
