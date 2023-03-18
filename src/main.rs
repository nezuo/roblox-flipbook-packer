#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod export;

use std::path::PathBuf;

use eframe::{CreationContext, Frame, NativeOptions};
use egui::{Button, CentralPanel, Context, TextureHandle, TextureOptions, Ui, Vec2};

fn main() -> Result<(), eframe::Error> {
    eframe::run_native(
        "Roblox Flipbook Packer",
        NativeOptions::default(),
        Box::new(|cc| Box::new(App::new(cc))),
    )
}

struct App {
    rows: u32,
    columns: u32,
    textures: Vec<TextureHandle>,
    image_path: Option<PathBuf>,
    sequence_path: Option<Vec<PathBuf>>,
}

impl App {
    fn new(_: &CreationContext<'_>) -> Self {
        App {
            rows: 1,
            columns: 1,
            textures: Vec::new(),
            image_path: None,
            sequence_path: None,
        }
    }
}

fn load_image_from_path(path: &std::path::Path) -> Result<egui::ColorImage, image::ImageError> {
    let image = image::io::Reader::open(path)?.decode()?;
    let size = [image.width() as _, image.height() as _];
    let image_buffer = image.to_rgba8();
    let pixels = image_buffer.as_flat_samples();

    Ok(egui::ColorImage::from_rgba_unmultiplied(
        size,
        pixels.as_slice(),
    ))
}

impl App {
    fn render_import(&mut self, ui: &mut Ui) {
        ui.group(|ui| {
            ui.heading("Import");
            ui.separator();

            ui.horizontal(|ui| {
                if ui.button("Import").clicked() {
                    self.image_path = rfd::FileDialog::new().pick_file();

                    if self.image_path.is_some() {
                        self.sequence_path = None;

                        let texture = ui.ctx().load_texture(
                            "",
                            load_image_from_path(self.image_path.as_ref().unwrap()).unwrap(),
                            TextureOptions::LINEAR,
                        );

                        self.textures = vec![texture];
                    }
                }

                if ui.button("Import Sequence").clicked() {
                    self.sequence_path = rfd::FileDialog::new().pick_files();

                    if self.sequence_path.is_some() {
                        self.image_path = None;

                        self.textures = Vec::new();

                        let paths = self.sequence_path.as_ref().unwrap();

                        for path in paths {
                            let texture = ui.ctx().load_texture(
                                "",
                                load_image_from_path(path).unwrap(),
                                TextureOptions::LINEAR,
                            );

                            self.textures.push(texture);
                        }
                    }
                }
            });

            if !self.textures.is_empty() {
                ui.horizontal(|ui| {
                    for texture in &self.textures {
                        let size = texture.size_vec2();

                        let smaller = size.x.min(size.y) as f64;
                        let higher = size.x.max(size.y) as f64;

                        let size = if size.x > size.y {
                            Vec2::new(128.0, (128.0 * smaller / higher) as f32)
                        } else {
                            Vec2::new((128.0 * smaller / higher) as f32, 128.0)
                        };

                        ui.image(texture, size);
                    }
                });
            }

            if self.image_path.is_some() {
                ui.horizontal(|ui| {
                    ui.label("Rows:");
                    ui.add(
                        egui::DragValue::new(&mut self.rows)
                            .speed(0.1)
                            .clamp_range(1..=256),
                    );
                });

                ui.horizontal(|ui| {
                    ui.label("Columns:");
                    ui.add(
                        egui::DragValue::new(&mut self.columns)
                            .speed(0.1)
                            .clamp_range(1..=256),
                    );
                });
            }
        });
    }

    fn render_export(&mut self, ui: &mut Ui) {
        ui.group(|ui| {
            ui.heading("Export");
            ui.separator();

            ui.horizontal(|ui| {
                let pack_enabled = self.image_path.is_some() || self.sequence_path.is_some();

                if ui.add_enabled(pack_enabled, Button::new("Pack")).clicked() {
                    let sequence = match &self.image_path {
                        Some(image_path) => export::image_to_frames(
                            self.rows,
                            self.columns,
                            image_path.to_path_buf(),
                        ),
                        None => export::sequence_to_frames(
                            self.sequence_path.as_ref().unwrap().to_vec(),
                        ),
                    };

                    export::export_packed(sequence);
                };

                if ui
                    .add_enabled(self.image_path.is_some(), Button::new("Export to Sequence"))
                    .clicked()
                {
                    export::export_sequence(
                        self.rows,
                        self.columns,
                        self.image_path.as_ref().unwrap().clone(),
                    );
                };
            });
        });
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &Context, _: &mut Frame) {
        ctx.set_visuals(egui::Visuals::dark());

        CentralPanel::default().show(ctx, |ui| {
            self.render_import(ui);
            ui.add_space(6.0);
            self.render_export(ui);
        });
    }
}
