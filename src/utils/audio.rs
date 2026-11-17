use log::error;
use rodio::{Decoder, DeviceSinkBuilder, MixerDeviceSink, Player, buffer::SamplesBuffer};
use std::{
    collections::HashMap,
    fs::File,
    io::Cursor,
    num::{NonZeroU16, NonZeroU32},
    time::Duration,
};
use symphonia::core::{
    audio::SampleBuffer,
    codecs::{CodecRegistry, DecoderOptions},
    formats::FormatOptions,
    io::MediaSourceStream,
    meta::MetadataOptions,
    probe::Hint,
};
use symphonia_adapter_libopus::OpusDecoder;
#[cfg(not(debug_assertions))]
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
        let Some(path) = self.active_path.as_ref() else {
            return;
        };

        let position = self.player_audio.get_pos();
        let playing = !self.player_audio.is_paused() && !self.player_audio.empty();

        if let Some(state) = self.states.get_mut(path) {
            state.position = position.min(state.duration);
            state.playing = playing;
        }
    }
    pub fn play_msg(&self, sound: SoundType) {
        let bytes = match sound {
            SoundType::Send => self.sound_send,
            SoundType::Message => self.sound_message,
        };
        self.player_msg.clear();

        let cursor = Cursor::new(bytes);

        match Decoder::new(cursor) {
            Ok(source) => {
                self.player_msg.append(source);
                self.player_msg.play();
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

        let mss = MediaSourceStream::new(Box::new(sound_file), Default::default());

        let probed = match symphonia::default::get_probe().format(
            &Hint::new(),
            mss,
            &FormatOptions::default(),
            &MetadataOptions::default(),
        ) {
            Ok(probed) => probed,
            Err(err) => {
                error!("Помилка визначення формату: {:?}", err);
                self.active_path = None;
                return;
            }
        };

        let mut format = probed.format;

        let (track_id, codec_params) = {
            let Some(track) = format.default_track() else {
                error!("Аудіотрек не знайдено");
                self.active_path = None;
                return;
            };

            (track.id, track.codec_params.clone())
        };

        let mut codec_registry = CodecRegistry::new();
        symphonia::default::register_enabled_codecs(&mut codec_registry);
        codec_registry.register_all::<OpusDecoder>();

        let mut decoder = match codec_registry.make(&codec_params, &DecoderOptions::default()) {
            Ok(decoder) => decoder,
            Err(err) => {
                error!("Помилка створення декодера: {:?}", err);
                self.active_path = None;
                return;
            }
        };

        let mut all_samples: Vec<f32> = Vec::new();
        let mut audio_spec = None;

        loop {
            let packet = match format.next_packet() {
                Ok(packet) => packet,
                Err(_) => break,
            };

            if packet.track_id() != track_id {
                continue;
            }

            let decoded = match decoder.decode(&packet) {
                Ok(decoded) => decoded,
                Err(err) => {
                    error!("Помилка декодування пакета: {:?}", err);
                    continue;
                }
            };

            let spec = *decoded.spec();
            audio_spec.get_or_insert(spec);

            let mut buffer = SampleBuffer::<f32>::new(decoded.capacity() as u64, spec);

            buffer.copy_interleaved_ref(decoded);
            all_samples.extend_from_slice(buffer.samples());
        }

        let Some(spec) = audio_spec else {
            error!("Не вдалося декодувати аудіо");
            self.active_path = None;
            return;
        };

        let channels = match NonZeroU16::new(spec.channels.count() as u16) {
            Some(value) => value,
            None => return,
        };

        let sample_rate = match NonZeroU32::new(spec.rate) {
            Some(value) => value,
            None => return,
        };

        let duration = Duration::from_secs_f64(
            all_samples.len() as f64 / channels.get() as f64 / sample_rate.get() as f64,
        );

        self.states.insert(
            path.to_string(),
            AudioState {
                playing: true,
                position: Duration::ZERO,
                duration,
            },
        );

        let source = SamplesBuffer::new(channels, sample_rate, all_samples);

        self.player_audio.append(source);
        self.player_audio.play();
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
#[cfg(not(debug_assertions))]
pub fn set_max_vol() {
    let device = AudioDevice::from_default().unwrap();

    let _ = device.set_vol(100);
    let _ = device.set_mute(false);
}
