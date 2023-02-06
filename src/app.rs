use egui::Key;

use self::audio::AudioPlayer;

mod audio;
mod slider;

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct TemplateApp {
    #[serde(skip)]
    playback_speed: f32,

    #[serde(skip)]
    cur_pos: f32,

    picked_path: Option<String>,

    marks: Vec<f32>,

    #[serde(skip)]
    audio: AudioPlayer,
}

impl Default for TemplateApp {
    fn default() -> Self {
        let audio = AudioPlayer::new();
        Self {
            playback_speed: 1.0,
            picked_path: None,
            cur_pos: 0.0,
            marks: vec![],
            audio,
        }
    }
}

impl TemplateApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            let mut r: Self = eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
            if let Some(path) = r.picked_path.as_ref() {
                // try to load previous file
                _ = r.audio.load(path.as_str());
            }
            return r;
        }

        Default::default()
    }
}

impl eframe::App for TemplateApp {
    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let Self {
            playback_speed,
            picked_path,
            cur_pos,
            marks,
            audio,
        } = self;

        if audio.is_playing() {
            if let Some(total) = audio.total_time() {
                *cur_pos = audio.play_time().as_secs_f32() / total.as_secs_f32();
            }
            ctx.request_repaint();
        }
        // Examples of how to create different panels and windows.
        // Pick whichever suits you.
        // Tip: a good default choice is to just keep the `CentralPanel`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        #[cfg(not(target_arch = "wasm32"))] // no File->Quit on web pages!
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("File").clicked() {
                        if let Some(path) = rfd::FileDialog::new().pick_file() {
                            *picked_path = Some(path.display().to_string());
                            match audio.load(path.display().to_string().as_str()) {
                                Ok(_) => {},
                                Err(_) => {},
                            }
                        }
                    }
                    if ui.button("Quit").clicked() {
                        _frame.close();
                    }
                });
                egui::warn_if_debug_build(ui);
            });
        });

        egui::SidePanel::left("side_panel").show(ctx, |ui| {
            ui.heading("Side Panel");

            if let Some(pp) = picked_path {
                if !pp.is_empty() {
                    ui.horizontal(|ui| {
                        ui.label("file: ");
                        ui.text_edit_singleline(pp);
                    });
                }
            }

            if let Some(source) = audio.source.as_mut() {
                ui.add(
                    egui::Slider::from_get_set(0.5..=3.0, |v: Option<f64>| {
                        if let Some(v) = v {
                            *playback_speed = eframe::emath::Numeric::from_f64(v);
                            source.set_speed(*playback_speed);
                        }
                        eframe::emath::Numeric::to_f64(*playback_speed)
                    })
                    .text("Playback")
                    .logarithmic(true),
                );

                ui.with_layout(
                    egui::Layout::top_down_justified(egui::Align::Center),
                    |ui| {
                        ui.label("Playback Speed");
                        if ui.button("Half").clicked() {
                            *playback_speed = 0.5;
                            source.set_speed(0.5);
                        }
                        if ui.button("Regular").clicked() {
                            *playback_speed = 1.0;
                            source.set_speed(1.0);
                        }
                        if ui.button("Double").clicked() {
                            source.set_speed(2.0);
                            *playback_speed = 2.0;
                        }
                    },
                );
            }
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            if ctx.input().key_pressed(Key::Space) {
                audio.toggle_play();
            }
            if ctx.input().key_pressed(Key::ArrowLeft) {
                if marks.is_empty() {
                    *cur_pos = 0.0;
                }
                // if we're playing we'll have a small offset to jump past something if it's
                // "too close"
                let pos = *cur_pos + if audio.is_playing() { -0.05 } else { 0.0 };
                if let Some(val) = marks.iter().rev().find(|i| *i < &pos) {
                    *cur_pos = *val;
                } else {
                    *cur_pos = 0.0;
                }
                audio.scrub_to(*cur_pos);
            }
            if ctx.input().key_pressed(Key::ArrowRight) {
                if marks.is_empty() {
                    // TODO: or whatever max should be
                    *cur_pos = 1.0;
                }
                if let Some(val) = marks.iter().find(|i| *i > cur_pos) {
                    *cur_pos = *val;
                } else {
                    // TODO: or whatever max should be
                    *cur_pos = 1.0;
                }
                audio.scrub_to(*cur_pos);
            }
            // if ctx.input(|i| i.key_pressed(Key::Space)) {
            //     audio.toggle_play();
            // }
            // The central panel the region left after adding TopPanel's and SidePanel's
            ui.style_mut().spacing.slider_width = ui.max_rect().width();
            ui.vertical_centered_justified(|ui| {
                // let slider = egui::Slider::new(cur_pos, 0.0..=1.0).show_value(true);
                // let slider = slider::Slider::new(cur_pos, 0.0..=1.0);
                let slider = slider::Slider::from_get_set(0.0..=1.0, |v: Option<f64>| {
                    if let Some(v) = v {
                        *cur_pos = eframe::emath::Numeric::from_f64(v);
                        audio.scrub_to(*cur_pos);
                    }
                    eframe::emath::Numeric::to_f64(*cur_pos)
                });
                // ui.add(egui::Slider::new(cur_pos, 0.0..=1.0).show_value(true));
                ui.add(slider);
            });

            ui.spacing_mut().item_spacing.y = 10.0;
            ui.horizontal_top(|ui| {
                if ui.button("Mark").clicked() {
                    if marks.contains(cur_pos) {
                        return;
                    }
                    marks.push(*cur_pos);
                    marks.sort_by(|a, b| a.partial_cmp(b).unwrap());
                    // TODO: sort
                }
                if ui.button("Prev").clicked() {
                    if marks.is_empty() {
                        *cur_pos = 0.0;
                    }
                    // if we're playing we'll have a small offset to jump past something if it's
                    // "too close"
                    let pos = *cur_pos + if audio.is_playing() { -0.05 } else { 0.0 };
                    if let Some(val) = marks.iter().rev().find(|i| *i < &pos) {
                        *cur_pos = *val;
                    } else {
                        *cur_pos = 0.0;
                    }
                    audio.scrub_to(*cur_pos);
                }
                if audio.is_playing() {
                    if ui.button("Pause").clicked() {
                        audio.toggle_play();
                    }
                } else if ui.button("Play").clicked() {
                    audio.toggle_play();
                }
                if ui.button("Next").clicked() {
                    if marks.is_empty() {
                        // TODO: or whatever max should be
                        *cur_pos = 1.0;
                    }
                    if let Some(val) = marks.iter().find(|i| *i > cur_pos) {
                        *cur_pos = *val;
                    } else {
                        // TODO: or whatever max should be
                        *cur_pos = 1.0;
                    }
                    audio.scrub_to(*cur_pos);
                }
            });

            if !marks.is_empty() {
                // ui.spacing_mut().item_spacing.y = 20.0;
                ui.with_layout(
                    egui::Layout::top_down_justified(egui::Align::Center),
                    |ui| {
                        for ind in 0..marks.len() {
                            if ind >= marks.len() {
                                return;
                            }
                            ui.horizontal_top(|ui| {
                                ui.label(format!("{}: {}", ind, marks.get(ind).expect("no")));
                                if ui.button("Jump").clicked() {
                                    *cur_pos = *marks.get(ind).expect("can't jump");
                                    audio.scrub_to(*cur_pos);
                                }
                                if ui.button("Delete").clicked() {
                                    marks.remove(ind);
                                    ctx.request_repaint();
                                }
                            });
                        }
                    },
                );
            }
        });

        if false {
            egui::Window::new("Window").show(ctx, |ui| {
                ui.label("Windows can be moved by dragging them.");
                ui.label("They are automatically sized based on contents.");
                ui.label("You can turn on resizing and scrolling if you like.");
                ui.label("You would normally choose either panels OR windows.");
            });
        }
    }

    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }
}
