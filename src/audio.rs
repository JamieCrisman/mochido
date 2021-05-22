use rodio::*;

use std::cell::RefCell;
use std::io::BufReader;
use std::sync::Arc;
//use std::thread;
//use std::time::Duration;

pub struct AudioPlayer {
    stream: Arc<RefCell<OutputStream>>,
    handle: Arc<RefCell<OutputStreamHandle>>,
    sink: Option<Arc<RefCell<Sink>>>,
    // file: Arc<RefCell<std::fs::File>>,
    source: Option<Arc<RefCell<rodio::Decoder<BufReader<std::fs::File>>>>>,
}

impl AudioPlayer {
    pub fn new() -> Self {
        let (stream, handle) = rodio::OutputStream::try_default().unwrap();
        AudioPlayer {
            stream: Arc::new(RefCell::new(stream)),
            handle: Arc::new(RefCell::new(handle)),
            sink: None,
            source: None,
            // file: Arc::new(RefCell::new(file)),
        }
    }

    pub fn play(&mut self) {
        if let Some(s) = &self.sink {
            let ss = s.borrow();
            if ss.is_paused() {
                ss.play();
            } else {
                ss.pause();
            }
        } else {
            // self.stream.play();
            let sink = rodio::Sink::try_new(&self.handle.borrow()).unwrap();
            let file = std::fs::File::open("test_audio/audio.mp3").unwrap();
            sink.append(rodio::Decoder::new(BufReader::new(file)).unwrap());
            // let beep1 = self.handle.borrow().play_once(self.master_buff).unwrap();
            //beep1.set_volume(0.2);

            self.sink = Some(Arc::new(RefCell::new(sink)));
            // thread::sleep(Duration::from_millis(1500));
        }
    }
}
