#![warn(clippy::all, rust_2018_idioms)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use mochido::TemplateApp;

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
fn main() {
    // Log to stdout (if you run with `RUST_LOG=debug`).
    tracing_subscriber::fmt::init();

    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "eframe template",
        native_options,
        Box::new(|cc| Box::new(TemplateApp::new(cc))),
    );
}

// when compiling to web using trunk.
#[cfg(target_arch = "wasm32")]
fn main() {
    // Make sure panics are logged using `console.error`.
    console_error_panic_hook::set_once();

    // Redirect tracing to console.log and friends:
    tracing_wasm::set_as_global_default();

    let web_options = eframe::WebOptions::default();

    wasm_bindgen_futures::spawn_local(async {
        eframe::start_web(
            "the_canvas_id", // hardcode it
            web_options,
            Box::new(|cc| Box::new(eframe_template::TemplateApp::new(cc))),
        )
        .await
        .expect("failed to start eframe");
    });
}

// struct Mochido {
//     duration: Duration,
//     state: State,
//     audio: AudioPlayer,
//     audio_ctx: Box<dyn AudioContext>,
// }

// enum State {
//     Idle,
//     Ticking { last_tick: Instant },
// }

// #[derive(Debug, Clone)]
// enum Message {
//     Toggle,
//     Reset,
//     Tick(Instant),
// }

// impl Application for Mochido {
//     type Executor = executor::Default;
//     type Message = Message;
//     type Flags = ();

//     fn new(_flags: ()) -> (Mochido, Command<Message>) {
//         let audio_ctx: Box<dyn AudioContext> = Box::new(audio::RodioAudioContext::new().unwrap());
//         let audio = AudioPlayer::new(audio_ctx.as_ref());
//         (
//             Mochido {
//                 duration: Duration::default(),
//                 state: State::Idle,
//                 toggle: button::State::new(),
//                 reset: button::State::new(),
//                 audio,
//                 audio_ctx,
//             },
//             Command::none(),
//         )
//     }

//     fn title(&self) -> String {
//         String::from("Mochido")
//     }

//     fn update(&mut self, message: Message, _clipboard: &mut Clipboard) -> Command<Message> {
//         match message {
//             Message::Toggle => match self.state {
//                 State::Idle => {
//                     self.state = State::Ticking {
//                         last_tick: Instant::now(),
//                     };
//                     self.audio.play();
//                 }
//                 State::Ticking { .. } => {
//                     self.state = State::Idle;
//                     self.audio.play();
//                 }
//             },
//             Message::Tick(now) => match &mut self.state {
//                 State::Ticking { last_tick } => {
//                     self.duration += now - *last_tick;
//                     *last_tick = now;
//                 }
//                 _ => {}
//             },
//             Message::Reset => {
//                 self.duration = Duration::default();
//                 self.audio.stop(self.audio_ctx.as_ref());
//                 self.audio.play();
//                 self.state = State::Idle;
//             }
//         }

//         Command::none()
//     }

//     fn subscription(&self) -> Subscription<Message> {
//         match self.state {
//             State::Idle => Subscription::none(),
//             State::Ticking { .. } => time::every(Duration::from_millis(10)).map(Message::Tick),
//         }
//     }

//     fn view(&mut self) -> Element<Message> {
//         const MINUTE: u64 = 60;
//         const HOUR: u64 = 60 * MINUTE;

//         let playtime = self.audio.play_time();
//         let duration = if let Some(total_seconds) = self.audio.total_time() {
//             let tsec = total_seconds.as_secs();
//             let seconds = playtime.as_secs();
//             Text::new(format!(
//                 "{:0>2}:{:0>2}:{:0>2}.{:0>2} of {:0>2}:{:0>2}:{:0>2}.{:0>2}",
//                 seconds / HOUR,
//                 (seconds % HOUR) / MINUTE,
//                 seconds % MINUTE,
//                 playtime.subsec_millis() / 10,
//                 tsec / HOUR,
//                 (tsec % HOUR) / MINUTE,
//                 tsec % MINUTE,
//                 total_seconds.subsec_millis() / 10,
//             ))
//             .size(40)
//         } else {
//             let seconds = playtime.as_secs();
//             Text::new(format!(
//                 "{:0>2}:{:0>2}:{:0>2}.{:0>2}",
//                 seconds / HOUR,
//                 (seconds % HOUR) / MINUTE,
//                 seconds % MINUTE,
//                 playtime.subsec_millis() / 10,
//             ))
//             .size(40)
//         };
//         let button = |state, label, style| {
//             Button::new(
//                 state,
//                 Text::new(label).horizontal_alignment(HorizontalAlignment::Center),
//             )
//             .min_width(80)
//             .padding(10)
//             .style(style)
//         };

//         let toggle_button = {
//             let (label, color) = match self.state {
//                 State::Idle => ("Start", style::Button::Primary),
//                 State::Ticking { .. } => ("Stop", style::Button::Destructive),
//             };

//             button(&mut self.toggle, label, color).on_press(Message::Toggle)
//         };

//         let reset_button =
//             button(&mut self.reset, "Reset", style::Button::Secondary).on_press(Message::Reset);

//         let controls = Row::new()
//             .spacing(20)
//             .push(toggle_button)
//             .push(reset_button);

//         let content = Column::new()
//             .align_items(Align::Center)
//             .spacing(20)
//             .push(duration)
//             .push(controls);

//         Container::new(content)
//             .width(Length::Fill)
//             .height(Length::Fill)
//             .center_x()
//             .center_y()
//             .into()
//     }
// }

// mod style {
//     use iced::{button, Background, Color, Vector};

//     pub enum Button {
//         Primary,
//         Secondary,
//         Destructive,
//     }

//     impl button::StyleSheet for Button {
//         fn active(&self) -> button::Style {
//             button::Style {
//                 background: Some(Background::Color(match self {
//                     Button::Primary => Color::from_rgb(0.11, 0.42, 0.87),
//                     Button::Secondary => Color::from_rgb(0.5, 0.5, 0.5),
//                     Button::Destructive => Color::from_rgb(0.8, 0.2, 0.2),
//                 })),
//                 border_radius: 12.0,
//                 shadow_offset: Vector::new(1.0, 1.0),
//                 text_color: Color::WHITE,
//                 ..button::Style::default()
//             }
//         }
//     }
// }
