use rodio::*;

use std::cell::RefCell;
//use std::error::Error;
use std::io::{Cursor, ErrorKind, Read};
use std::path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time;
use std::time::Duration;

pub trait AudioContext {
    fn device(&self) -> &rodio::OutputStreamHandle;
}

pub struct RodioAudioContext {
    _stream: rodio::OutputStream,
    stream_handle: rodio::OutputStreamHandle,
}

impl RodioAudioContext {
    pub fn new() -> Result<Self, rodio::StreamError> {
        let (_stream, stream_handle) = rodio::OutputStream::try_default()?;
        Ok(Self {
            _stream,
            stream_handle,
        })
    }
}

impl AudioContext for RodioAudioContext {
    fn device(&self) -> &rodio::OutputStreamHandle {
        &self.stream_handle
    }
}

#[derive(Clone, Debug)]
pub struct SoundData(Arc<[u8]>);

impl SoundData {
    pub fn new<P: AsRef<path::Path>>(path: P) -> Result<Self, std::io::Error> {
        let path = path.as_ref();
        let file = &mut std::fs::File::open(path)?;
        SoundData::from_read(file)
    }

    pub fn from_bytes(data: &[u8]) -> Self {
        SoundData(Arc::from(data))
    }

    pub fn from_read<R>(reader: &mut R) -> Result<Self, std::io::Error>
    where
        R: Read,
    {
        let mut buffer = Vec::new();
        let _ = reader.read_to_end(&mut buffer)?;

        Ok(SoundData::from(buffer))
    }

    pub fn can_play(&self) -> bool {
        let cursor = Cursor::new(self.clone());
        rodio::Decoder::new(cursor).is_ok()
    }
}

impl From<Arc<[u8]>> for SoundData {
    #[inline]
    fn from(arc: Arc<[u8]>) -> Self {
        SoundData(arc)
    }
}

impl From<Vec<u8>> for SoundData {
    fn from(v: Vec<u8>) -> Self {
        SoundData(Arc::from(v))
    }
}

impl From<Box<[u8]>> for SoundData {
    fn from(b: Box<[u8]>) -> Self {
        SoundData(Arc::from(b))
    }
}

impl AsRef<[u8]> for SoundData {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

pub struct SourceState {
    data: Cursor<SoundData>,
    repeat: bool,
    fade_in: time::Duration,
    speed: f32,
    query_interval: time::Duration,
    play_time: Arc<AtomicUsize>,
}

impl SourceState {
    pub fn new(cursor: Cursor<SoundData>) -> Self {
        SourceState {
            data: cursor,
            repeat: false,
            fade_in: time::Duration::from_millis(0),
            speed: 1.0,
            query_interval: time::Duration::from_millis(100),
            play_time: Arc::new(AtomicUsize::new(0)),
        }
    }

    pub fn set_repeat(&mut self, repeat: bool) {
        self.repeat = repeat;
    }

    pub fn set_fade_in(&mut self, dur: time::Duration) {
        self.fade_in = dur;
    }

    pub fn set_speed(&mut self, ratio: f32) {
        self.speed = ratio;
    }

    pub fn repeat(&self) -> bool {
        self.repeat
    }

    pub fn elapsed(&self) -> time::Duration {
        let t = self.play_time.load(Ordering::SeqCst);
        time::Duration::from_micros(t as u64)
    }

    pub fn set_query_interval(&mut self, t: time::Duration) {
        self.query_interval = t;
    }
}

pub struct Source {
    sink: rodio::Sink,
    state: SourceState,
}

impl Source {
    pub fn new<P: AsRef<path::Path>>(
        audio_context: &dyn AudioContext,
        path: P,
    ) -> Result<Self, std::io::Error> {
        let path = path.as_ref();
        let data = SoundData::new(path)?;
        Source::from_data(audio_context, data)
    }

    pub fn from_data(
        audio_context: &dyn AudioContext,
        data: SoundData,
    ) -> Result<Self, std::io::Error> {
        if !data.can_play() {
            return Err(std::io::Error::new(
                ErrorKind::Other,
                "Couldn't play the audio",
            ));
        }
        let sink = rodio::Sink::try_new(audio_context.device());
        if sink.is_err() {
            return Err(std::io::Error::new(
                ErrorKind::Other,
                "Couldn't create the sink",
            ));
        }
        let cursor = Cursor::new(data);
        Ok(Source {
            sink: sink.unwrap(),
            state: SourceState::new(cursor),
        })
    }

    fn play_later(&self) -> Result<(), rodio::decoder::DecoderError> {
        use rodio::Source;
        let cursor = self.state.data.clone();
        let counter = self.state.play_time.clone();
        let period_mus = self.state.query_interval.as_secs() as usize * 1_000_000
            + self.state.query_interval.subsec_micros() as usize;

        //if self.state.repeat {
        let sound = rodio::Decoder::new(cursor)?
            .repeat_infinite()
            .speed(self.state.speed)
            .fade_in(self.state.fade_in)
            .periodic_access(self.state.query_interval, move |_| {
                let _ = counter.fetch_add(period_mus, Ordering::SeqCst);
            });
        self.sink.append(sound);
        //}

        Ok(())
    }

    fn set_repeat(&mut self, repeat: bool) {
        self.state.set_repeat(repeat)
    }

    fn set_fade_in(&mut self, dur: time::Duration) {
        self.state.set_fade_in(dur);
    }

    fn set_speed(&mut self, ratio: f32) {
        self.state.set_speed(ratio);
    }

    fn repeat(&self) -> bool {
        self.state.repeat
    }

    fn pause(&self) {
        self.sink.pause()
    }

    fn resume(&self) {
        self.sink.play()
    }

    fn stop(&mut self, audio_context: Box<dyn AudioContext>) -> Result<(), rodio::PlayError> {
        let volume = self.volume();
        let device = audio_context.device();
        self.sink = rodio::Sink::try_new(&device)?;
        self.state.play_time.store(0, Ordering::SeqCst);
        self.set_volume(volume);
        Ok(())
    }

    fn stopped(&self) -> bool {
        self.sink.empty()
    }

    fn volume(&self) -> f32 {
        self.sink.volume()
    }

    fn set_volume(&mut self, value: f32) {
        self.sink.set_volume(value)
    }

    fn paused(&self) -> bool {
        self.sink.is_paused()
    }

    fn playing(&self) -> bool {
        !self.paused() && !self.stopped()
    }

    fn elapsed(&self) -> time::Duration {
        self.state.elapsed()
    }

    fn set_query_interval(&mut self, t: time::Duration) {
        self.state.set_query_interval(t)
    }
}

pub struct AudioPlayer {
    source: Box<Source>,
}

impl AudioPlayer {
    pub fn new(ctx: &dyn AudioContext) -> Self {
        let source = Source::new(ctx, path::Path::new("test_audio/audio.mp3")).unwrap();
        if source.play_later().is_ok() {
            println!("sure");
        };
        source.resume();
        //thread::sleep(Duration::from_secs(2));
        AudioPlayer {
            source: Box::new(source),
        }
    }

    pub fn play(&mut self) {
        if !self.source.playing() {
            self.source.resume();
        } else {
            self.source.pause();
        }
    }
}
