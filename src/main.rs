#![feature(io_read_to_string)]

use egui::{Color32, Pos2, Rect, Rounding, Sense, Vec2};
use egui_miniquad as egui_mq;
use gol::Board;
use miniquad as mq;
use serde_json::{from_reader, to_writer_pretty};
use std::{
    fs::File,
    io::{BufReader, BufWriter},
};

const BOARD_SCALE: f32 = 10.0;

struct Stage {
    egui_mq: egui_mq::EguiMq,
    filename: String,
    edit_mode: bool,
    resize: Option<[String; 2]>,
    resize_warning: bool,
    board: Option<Board>,
}

impl Stage {
    fn new(ctx: &mut mq::Context) -> Self {
        let board = Some(Board::new(30, 30));
        Self {
            egui_mq: egui_mq::EguiMq::new(ctx),
            filename: String::new(),
            edit_mode: false,
            resize: None,
            resize_warning: false,
            board,
        }
    }
}

impl mq::EventHandler for Stage {
    fn update(&mut self, _ctx: &mut mq::Context) {}

    fn draw(&mut self, mq_ctx: &mut mq::Context) {
        mq_ctx.clear(Some((1., 1., 1., 1.)), None, None);
        mq_ctx.begin_default_pass(mq::PassAction::clear_color(0.2, 0.2, 0.2, 1.0));
        mq_ctx.end_render_pass();

        self.egui_mq.run(mq_ctx, |_mq_ctx, egui_ctx| {
            egui::TopBottomPanel::top("top").show(egui_ctx, |ui| {
                ui.horizontal(|ui| {
                    let filename = &mut self.filename;
                    ui.label("Filename:");
                    ui.text_edit_singleline(filename);
                    if ui.button("Save").clicked() {
                        if let Ok(file) = File::create(&filename) {
                            let file = BufWriter::new(file);
                            if to_writer_pretty(file, &self.board).is_err() {
                                eprintln!("error writing board to {filename}");
                            }
                        } else {
                            eprintln!("could not write to {filename}");
                        }
                    }
                    if ui.button("Load").clicked() {
                        if let Ok(file) = File::open(&filename) {
                            let file = BufReader::new(file);
                            if let Ok(board) = from_reader(file) {
                                self.board.replace(board);
                            } else {
                                eprintln!("invalid board data in {filename}");
                            }
                        } else {
                            eprintln!("could not read from {filename}");
                        }
                    }
                });

                if ui.button("Resize").clicked() {
                    self.resize = Some([String::new(), String::new()]);
                }
            });

            egui::TopBottomPanel::bottom("bottom").show(egui_ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.toggle_value(&mut self.edit_mode, "Edit");
                    if ui.button("Next").clicked() {
                        if let Some(board) = self.board.as_ref() {
                            self.board = Some(board.next());
                        }
                    }
                    #[cfg(not(target_arch = "wasm32"))]
                    {
                        if ui.button("Quit").clicked() {
                            std::process::exit(0);
                        }
                    }
                });
            });

            egui::CentralPanel::default().show(egui_ctx, |ui| {
                egui::ScrollArea::both().show(ui, |ui| {
                    if let Some(board) = self.board.as_mut() {
                        let (response, painter) = ui.allocate_painter(
                            #[allow(clippy::cast_precision_loss)]
                            Vec2::new(
                                board.size()[0] as f32 * BOARD_SCALE,
                                board.size()[1] as f32 * BOARD_SCALE,
                            ),
                            Sense::click(),
                        );
                        painter.rect_filled(response.rect, Rounding::none(), Color32::WHITE);
                        let min = response.rect.min;
                        for ((j, i), _) in board.iter().filter(|(_, live)| *live) {
                            #[allow(clippy::cast_precision_loss)]
                            let pix_min = Pos2 {
                                x: min.x + BOARD_SCALE * i as f32,
                                y: min.y + BOARD_SCALE * j as f32,
                            };
                            #[allow(clippy::cast_precision_loss)]
                            let pix_max = Pos2 {
                                x: min.x + BOARD_SCALE * (i + 1) as f32,
                                y: min.y + BOARD_SCALE * (j + 1) as f32,
                            };
                            painter.rect_filled(
                                Rect::from_two_pos(pix_min, pix_max),
                                Rounding::none(),
                                Color32::BLACK,
                            );
                        }
                        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                        if self.edit_mode && response.clicked() &&
                            let Some(pos) = response.interact_pointer_pos() {
                                let pix = ((pos - min) / BOARD_SCALE).floor();
                                let x = pix[0] as usize;
                                let y = pix[1] as usize;
                                board[(y, x)] = !board[(y, x)];
                        }
                    }
                });
            });

            let mut close_window = false;
            if let Some([new_width, new_height]) = self.resize.as_mut() {
                egui::Window::new("Resize").show(egui_ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.text_edit_singleline(new_width);
                        ui.text_edit_singleline(new_height);
                    });
                    ui.horizontal(|ui| {
                        if ui.button("Cancel").clicked() {
                            close_window = true;
                            self.resize_warning = false;
                        }
                        if ui.button("Ok").clicked() {
                            if let (Ok(w), Ok(h)) =
                                (new_width.parse::<usize>(), new_height.parse::<usize>())
                            {
                                let new_board = self
                                    .board
                                    .as_ref()
                                    .map_or_else(|| Board::new(w, h), |b| b.resize(w, h));
                                self.board.replace(new_board);
                                close_window = true;
                                self.resize_warning = false;
                            } else {
                                self.resize_warning = true;
                            }
                        }
                    });
                    if self.resize_warning {
                        ui.label("Could not parse dimensions");
                    }
                });
            }
            if close_window {
                self.resize = None;
            }
        });

        self.egui_mq.draw(mq_ctx);
        mq_ctx.commit_frame();
    }

    fn mouse_motion_event(&mut self, _: &mut mq::Context, x: f32, y: f32) {
        self.egui_mq.mouse_motion_event(x, y);
    }

    fn mouse_wheel_event(&mut self, _: &mut mq::Context, dx: f32, dy: f32) {
        self.egui_mq.mouse_wheel_event(dx, dy);
    }

    fn mouse_button_down_event(
        &mut self,
        ctx: &mut mq::Context,
        mb: mq::MouseButton,
        x: f32,
        y: f32,
    ) {
        self.egui_mq.mouse_button_down_event(ctx, mb, x, y);
    }

    fn mouse_button_up_event(
        &mut self,
        ctx: &mut mq::Context,
        mb: mq::MouseButton,
        x: f32,
        y: f32,
    ) {
        self.egui_mq.mouse_button_up_event(ctx, mb, x, y);
    }

    fn char_event(
        &mut self,
        _ctx: &mut mq::Context,
        character: char,
        _keymods: mq::KeyMods,
        _repeat: bool,
    ) {
        self.egui_mq.char_event(character);
    }

    fn key_down_event(
        &mut self,
        ctx: &mut mq::Context,
        keycode: mq::KeyCode,
        keymods: mq::KeyMods,
        _repeat: bool,
    ) {
        self.egui_mq.key_down_event(ctx, keycode, keymods);
    }

    fn key_up_event(&mut self, _ctx: &mut mq::Context, keycode: mq::KeyCode, keymods: mq::KeyMods) {
        self.egui_mq.key_up_event(keycode, keymods);
    }
}

fn main() {
    let conf = mq::conf::Conf {
        window_title: "Conway's Game of Life".to_string(),
        high_dpi: true,
        ..Default::default()
    };
    mq::start(conf, |ctx| Box::new(Stage::new(ctx)));
}
