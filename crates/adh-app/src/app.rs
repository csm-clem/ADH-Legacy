//! Fenêtre principale de l'Atlas des harmoniques.
//!
//! L'application Qt séparait le modèle (`ModelADHCore`, `ModelHarmonicManager`,
//! `NotesMasking`, `PartielsMasking`) de la vue QML, reliés par 23 `Q_PROPERTY`
//! et leurs signaux. Ici `Atlas` est le modèle : l'UI egui le relit à chaque
//! image, il n'y a plus rien à notifier.

use adh_core::builtin::instruments;
use adh_core::masking::{NoteMask, PartialMask};
use adh_core::{Harmonic, HarmonicEngine, Instrument, Note, Partial, PartialInfo, Point};
use egui::{
    Align2, CentralPanel, Color32, Context, CornerRadius, FontId, Frame, Id, Margin, Panel,
    Painter, Pos2, Rect, Response, Sense, Stroke, StrokeKind, TextureHandle, Ui, Vec2, pos2,
};

use crate::i18n::{Lang, Tr};
use crate::scene::{thumbprint_size, sticker_size, View, PLECTRUM_LAYOUT, ZOOM_STEP};
use crate::settings::{CustomTuning, Settings};
use crate::textures::Store;

/// Uv complet d'une texture.
fn full_uv() -> Rect {
    Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0))
}

/// Couleur de fond de la scène (`color: "darkgray"` du QML).
const SCENE_BACKGROUND: Color32 = Color32::from_gray(160);

/// Application complète : moteur, cadrage, masques, visuels, réglages.
pub struct Atlas {
    instruments: Vec<Instrument>,
    engine: HarmonicEngine,
    view: View,
    note_mask: NoteMask,
    partial_mask: PartialMask,
    textures: Store,
    lang: Lang,
    settings: Settings,
    show_about: bool,
    dirty: bool,
    last_action: String,
    /// `ADH_DEBUG=1` affiche le résultat du hit-test sur la sortie d'erreur.
    debug: bool,
    /// `ADH_SCREENSHOT=/tmp/adh.png` enregistre une capture puis ferme la fenêtre.
    screenshot: Option<std::path::PathBuf>,
    /// Nombre d'images déjà dessinées, pour ne capturer qu'une scène complète.
    frames: u64,
    /// Accordages ajoutés par l'utilisateur, tous instruments confondus.
    custom_tunings: Vec<CustomTuning>,
    /// Fenêtre « Accordage personnalisé… ».
    show_custom: bool,
    /// Champs de la fenêtre « Accordage personnalisé… ».
    custom_name: String,
    custom_strings: String,
    /// Message d'erreur de cette fenêtre.
    custom_error: Option<String>,
    /// Zone de dessin de la dernière image, pour les actions de menu.
    area: Rect,
    /// Taille du canevas scène de l'instrument courant.
    scene_size: Vec2,
}

impl Atlas {
    /// Reconstruit l'application depuis les réglages enregistrés.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        install_symbol_font(&cc.egui_ctx);
        let settings = Settings::load();
        let all = instruments().expect("instruments embarqués invalides");
        let custom_tunings = settings.custom_tunings.clone();
        let mut chosen = all
            .iter()
            .find(|instrument| instrument.id() == settings.instrument)
            .cloned()
            .unwrap_or_else(|| all[0].clone());
        let chosen_id = chosen.id().to_string();
        for custom in custom_tunings.iter().filter(|c| c.instrument == chosen_id) {
            if let Err(error) = chosen.add_tuning(&custom.name, &custom.raw) {
                eprintln!("accordage personnalisé ignoré : {error}");
            }
        }
        let mut engine = HarmonicEngine::new(chosen);
        if settings.tuning > 0 && settings.tuning < engine.instrument().tunings().len() {
            engine.set_tuning(settings.tuning);
        }
        Self {
            instruments: all,
            engine,
            view: View::from_saved(settings.zoom, settings.pan),
            note_mask: settings.note_mask(),
            partial_mask: settings.partial_mask(),
            textures: Store::new(),
            lang: settings.lang,
            settings,
            show_about: false,
            dirty: false,
            last_action: String::new(),
            debug: std::env::var("ADH_DEBUG").is_ok(),
            screenshot: std::env::var("ADH_SCREENSHOT")
                .map(std::path::PathBuf::from)
                .ok(),
            frames: 0,
            custom_tunings,
            show_custom: false,
            custom_name: String::new(),
            custom_strings: String::new(),
            custom_error: None,
            area: Rect::NOTHING,
            scene_size: Vec2::ZERO,
        }
    }

    /// Change d'instrument.
    fn select_instrument(&mut self, id: &str) {
        if let Some(found) = self.instruments.iter().find(|i| i.id() == id).cloned() {
            let mut instrument = found;
            for custom in self
                .custom_tunings
                .iter()
                .filter(|custom| custom.instrument == id)
                .cloned()
            {
                if let Err(error) = instrument.add_tuning(&custom.name, &custom.raw) {
                    eprintln!("accordage personnalisé ignoré : {error}");
                }
            }
            self.engine.set_instrument(instrument);
            self.settings.instrument = id.to_string();
            self.settings.tuning = 0;
            self.view = View::new();
            self.textures.forget_textures();
            self.last_action.clear();
            self.dirty = true;
        }
    }

    /// Change d'accordage.
    fn select_tuning(&mut self, index: usize) {
        if self.engine.set_tuning(index) {
            self.settings.tuning = index;
            self.last_action.clear();
            self.dirty = true;
        }
    }

    /// Bascule la visibilité d'une note (barre, médiator ou menu).
    fn toggle_note(&mut self, note: Note) {
        let visible = self.note_mask.toggle(note);
        self.last_action = format!(
            "{} {}",
            note.display(),
            if visible { "visible" } else { "masquée" }
        );
        self.dirty = true;
    }

    /// Bascule la visibilité d'un partiel.
    fn toggle_partial(&mut self, partial: Partial) {
        let visible = self.partial_mask.toggle(partial);
        self.last_action = format!(
            "{} {}",
            partial.ratio_label(),
            if visible { "visible" } else { "masqué" }
        );
        self.dirty = true;
    }

    /// Retire tous les doigts posés.
    fn clear_fingers(&mut self) {
        self.engine.clear_frets();
        self.last_action.clear();
    }

    /// Écrit les réglages sur le disque.
    fn persist(&mut self) {
        self.settings.capture_masks(&self.note_mask, &self.partial_mask);
        let (zoom, pan) = self.view.saved();
        self.settings.zoom = zoom;
        self.settings.pan = pan;
        self.settings.tuning = self.engine.tuning_index();
        self.settings.lang = self.lang;
        self.settings.custom_tunings = self.custom_tunings.clone();
        if let Err(error) = self.settings.save() {
            eprintln!("réglages non enregistrés : {error}");
        }
        self.dirty = false;
    }

    /// Texture d'une pastille de note.
    fn sticker_texture(&mut self, ctx: &Context, note: Note) -> TextureHandle {
        self.textures
            .texture(ctx, note.sticker(), adh_assets::note_sticker(note))
    }
}

impl eframe::App for Atlas {
    fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        let tr = self.lang.tr();
        if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
            self.show_about = false;
            self.show_custom = false;
        }
        self.menubar(ui, tr);
        self.partials_toolbar(ui, tr);
        self.notes_toolbar(ui, tr);
        self.statusbar(ui, tr);
        self.scene(ui);
        if self.show_about {
            self.about(ui, tr);
        }
        if self.show_custom {
            self.custom_window(ui, tr);
        }
        self.capture_screenshot(ui);
        if self.dirty {
            self.persist();
        }
    }

    fn save(&mut self, _storage: &mut dyn eframe::Storage) {
        self.persist();
    }

    fn persist_egui_memory(&self) -> bool {
        true
    }
}

impl Atlas {
    /// `ADH_SCREENSHOT=/tmp/adh.png` : enregistre la fenêtre en PNG puis la ferme.
    ///
    /// `eframe` sait déjà le faire avec `EFRAME_SCREENSHOT_TO`, mais son backend
    /// glow lit le tampon après son échange : la capture ressort noire ici. La
    /// commande de viewport, elle, est traitée juste après le dessin de l'image.
    fn capture_screenshot(&mut self, ui: &mut Ui) {
        let Some(path) = self.screenshot.clone() else {
            return;
        };
        self.frames += 1;
        // Deux images : la scène et ses textures doivent être en place.
        if self.frames < 2 {
            return;
        }
        let reply = ui.input(|i| {
            i.events.iter().find_map(|event| match event {
                egui::Event::Screenshot { image, .. } => Some(image.clone()),
                _ => None,
            })
        });
        let Some(shot) = reply else {
            ui.ctx().send_viewport_cmd(egui::ViewportCommand::Screenshot(
                egui::UserData::default(),
            ));
            return;
        };
        self.screenshot = None;
        let bytes: Vec<u8> = shot
            .pixels
            .iter()
            .flat_map(|color| color.to_srgba_unmultiplied())
            .collect();
        let frame = image::RgbaImage::from_raw(shot.size[0] as u32, shot.size[1] as u32, bytes);
        match frame {
            Some(frame) => match frame.save_with_format(&path, image::ImageFormat::Png) {
                Ok(()) => println!("capture écrite : {}", path.display()),
                Err(error) => eprintln!("capture impossible : {error}"),
            },
            None => eprintln!(
                "capture impossible : {} pixels pour {} × {}",
                shot.pixels.len(),
                shot.size[0],
                shot.size[1]
            ),
        }
        ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
    }

    /// Barre de menus : Instruments, Accordages, Affichage, Aide.
    fn menubar(&mut self, ui: &mut Ui, tr: &'static Tr) {
        let current_instrument = self.engine.instrument().id().to_string();
        let instruments: Vec<(String, String)> = self
            .instruments
            .iter()
            .map(|i| (i.id().to_string(), i.name().to_string()))
            .collect();
        let tunings: Vec<(usize, String)> = self
            .engine
            .instrument()
            .tunings()
            .iter()
            .enumerate()
            .map(|(index, tuning)| (index, format!("{} : {}", tuning.name, tuning.raw)))
            .collect();
        let current_tuning = self.engine.tuning_index();

        Panel::top("menubar").show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.menu_button(tr.instruments, |ui| {
                    for (id, name) in &instruments {
                        if ui.selectable_label(current_instrument == *id, name).clicked() {
                            self.select_instrument(id);
                            ui.close();
                        }
                    }
                });
                ui.menu_button(tr.tunings, |ui| {
                    egui::ScrollArea::vertical()
                        .max_height(420.0)
                        .auto_shrink([false, true])
                        .show(ui, |ui| {
                            for (index, label) in &tunings {
                                if ui.selectable_label(current_tuning == *index, label).clicked() {
                                    self.select_tuning(*index);
                                    ui.close();
                                }
                            }
                        });
                    ui.separator();
                    if ui.button(tr.custom_tuning).clicked() {
                        self.open_custom_window();
                        ui.close();
                    }
                });
                ui.menu_button(tr.view, |ui| {
                    if ui.button(tr.fit).clicked() {
                        self.view.fit(self.area, self.scene_size);
                        ui.close();
                    }
                    if ui.button(tr.zoom_in).clicked() {
                        self.view.zoom_step(self.area, self.scene_size, true);
                        ui.close();
                    }
                    if ui.button(tr.zoom_out).clicked() {
                        self.view.zoom_step(self.area, self.scene_size, false);
                        ui.close();
                    }
                    if ui.button(tr.zoom_full).clicked() {
                        self.view.actual_size(self.area, self.scene_size);
                        ui.close();
                    }
                    ui.separator();
                    if ui.button(tr.clear_fingers).clicked() {
                        self.clear_fingers();
                        ui.close();
                    }
                });
                ui.menu_button(tr.help, |ui| {
                    let links: [(&str, &str); 6] = [
                        (tr.tutorial, "https://lecompositeur.com/adh-tuto"),
                        (
                            tr.notations,
                            "https://lecompositeur.com/adh-notation-internationale",
                        ),
                        (
                            tr.serie,
                            "https://lecompositeur.com/adh-serie-harmoniques",
                        ),
                        (
                            tr.relative,
                            "https://lecompositeur.com/adh-notation-relative-partiels",
                        ),
                        (
                            tr.frequencies,
                            "https://lecompositeur.com/adh-frequence-notes",
                        ),
                        (tr.donation, "https://lecompositeur.com/adh-don"),
                    ];
                    for (label, url) in links {
                        if ui.button(label).clicked() {
                            crate::links::open(url);
                            ui.close();
                        }
                    }
                    ui.separator();
                    if ui.button(tr.about).clicked() {
                        self.show_about = true;
                        ui.close();
                    }
                    ui.menu_button(tr.language, |ui| {
                        for lang in [Lang::Fr, Lang::En] {
                            if ui.selectable_label(self.lang == lang, lang.label()).clicked() {
                                self.lang = lang;
                                self.dirty = true;
                                ui.close();
                            }
                        }
                    });
                });
            });
        });
    }
}

impl Atlas {
    /// Barre « Afficher les partiels ».
    fn partials_toolbar(&mut self, ui: &mut Ui, tr: &'static Tr) {
        Panel::top("partials").show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(tr.show_partials);
                if ui.button(tr.all).clicked() {
                    self.partial_mask.show_all();
                    self.dirty = true;
                }
                if ui.button(tr.none).clicked() {
                    self.partial_mask.hide_all();
                    self.dirty = true;
                }
                ui.separator();
                egui::ScrollArea::horizontal().show(ui, |ui| {
                    for partial in Partial::ALL {
                        let visible = self.partial_mask.is_visible(partial);
                        let button = ui
                            .selectable_label(visible, partial.ratio_label())
                            .on_hover_text(partial.interval_name_fr());
                        if button.clicked() {
                            self.toggle_partial(partial);
                        }
                    }
                });
            });
        });
    }

    /// Barre « Afficher les notes ».
    fn notes_toolbar(&mut self, ui: &mut Ui, tr: &'static Tr) {
        Panel::top("notes").show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(tr.show_notes);
                if ui.button(tr.all).clicked() {
                    self.note_mask.show_all();
                    self.dirty = true;
                }
                if ui.button(tr.none).clicked() {
                    self.note_mask.hide_all();
                    self.dirty = true;
                }
                ui.separator();
                egui::ScrollArea::horizontal().show(ui, |ui| {
                    for note in Note::ALL {
                        let visible = self.note_mask.is_visible(note);
                        let button = ui
                            .selectable_label(visible, note.display())
                            .on_hover_text(note.display_fr());
                        if button.clicked() {
                            self.toggle_note(note);
                        }
                    }
                });
            });
        });
    }

    /// Barre d'état : instrument, accordage, doigts posés, zoom.
    fn statusbar(&mut self, ui: &mut Ui, tr: &'static Tr) {
        let instrument = self.engine.instrument();
        let tuning = self.engine.tuning();
        let strings = instrument.string_count();
        let down: Vec<String> = (0..strings)
            .filter(|string| self.engine.fret(*string) > 0)
            .map(|string| format!("C{}:{}", string + 1, self.engine.fret(string)))
            .collect();
        let percent = (self.view.zoom() * 100.0).round() as i32;
        let action = self.last_action.clone();

        Panel::bottom("status").show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(instrument.name());
                ui.separator();
                ui.label(format!("{} : {}", tr.tuning, tuning.name));
                ui.separator();
                ui.label(format!("{} {}", strings, tr.strings));
                ui.separator();
                if down.is_empty() {
                    ui.label(format!("0 {}", tr.frets));
                } else {
                    ui.label(format!("{} {} — {}", down.len(), tr.frets, down.join(" ")));
                }
                if !action.is_empty() {
                    ui.separator();
                    ui.strong(action);
                }
                ui.with_layout(
                    egui::Layout::right_to_left(egui::Align::Center),
                    |ui| {
                        if self.view.is_fit() {
                            ui.label(format!("{} {} % — {}", tr.zoom, percent, tr.fitted));
                        } else {
                            ui.label(format!("{} {} %", tr.zoom, percent));
                        }
                    },
                );
            });
        });
    }
}

impl Atlas {
    /// Dessine la scène : photo, empreintes, pastilles, médiators, infobulle,
    /// puis traite la molette, le glisser, les raccourcis et le clic sur une corde.
    fn scene(&mut self, ui: &mut Ui) {
        let [width, height] = self.engine.instrument().scene_size();
        let scene_size = Vec2::new(width as f32, height as f32);

        CentralPanel::default_margins()
            .frame(
                Frame::central_panel(ui.style())
                    .fill(SCENE_BACKGROUND)
                    .inner_margin(Margin::same(0)),
            )
            .show(ui, |ui| {
                let (area, response) =
                    ui.allocate_exact_size(ui.available_size(), Sense::click_and_drag());
                self.area = area;
                self.scene_size = scene_size;
                self.view.apply_fit(area, scene_size);

                let ctx = ui.ctx().clone();
                let painter = ctx.layer_painter(ui.layer_id());

                self.draw_photo(ui, &ctx, &painter, area, scene_size);
                self.draw_fingers(&ctx, &painter, area, scene_size);
                let hovered = self.draw_stickers(&ctx, &painter, area, scene_size);
                let plectrum_hit = self.draw_plectrums(ui, &ctx, &painter, area, scene_size);
                self.handle_scene_input(ui, &ctx, &response, area, scene_size, plectrum_hit);

                if let Some(info) = hovered {
                    let anchor = ctx.pointer_hover_pos().unwrap_or(area.center());
                    draw_tooltip(&painter, anchor, &info);
                }
            });
    }

    /// Photo de l'instrument, étirée sur le canevas de référence.
    fn draw_photo(&mut self, ui: &Ui, ctx: &Context, painter: &Painter, area: Rect, scene_size: Vec2) {
        let name = self.engine.instrument().image().to_string();
        let bytes = adh_assets::instrument_image(&name).unwrap_or_default();
        let photo = self.textures.texture(ctx, &name, bytes);
        painter.rect(
            area,
            CornerRadius::ZERO,
            SCENE_BACKGROUND,
            Stroke::NONE,
            StrokeKind::Middle,
        );
        painter.image(photo.id(), self.view.image_rect(area, scene_size), full_uv(), Color32::WHITE);
        let _ = ui;
    }

    /// Empreintes de doigt des cordes où une frette est posée.
    fn draw_fingers(&mut self, ctx: &Context, painter: &Painter, area: Rect, scene: Vec2) {
        let thumbprint = self
            .textures
            .texture(ctx, "thumbprint", adh_assets::THUMBPRINT_PNG);
        for string in 0..self.engine.instrument().string_count() {
            let fret = self.engine.fret(string);
            if fret == 0 {
                continue;
            }
            let point = self.engine.finger_position(string, fret);
            let rect = self.view.scene_rect(area, scene, point, thumbprint_size());
            painter.image(thumbprint.id(), rect, full_uv(), Color32::WHITE);
        }
    }
}

impl Atlas {
    /// Pastilles d'harmoniques visibles, et détection de la pastille survolée.
    fn draw_stickers(
        &mut self,
        ctx: &Context,
        painter: &Painter,
        area: Rect,
        scene_size: Vec2,
    ) -> Option<PartialInfo> {
        let harmonics: Vec<Harmonic> = self
            .engine
            .visible_harmonics(self.note_mask, self.partial_mask)
            .copied()
            .collect();
        let mut hovered: Option<PartialInfo> = None;
        let pointer = ctx.input(|i| i.pointer.interact_pos());

        for harmonic in &harmonics {
            // Comme dans le QML (`sticker.scale = 1 / image.scale`), une pastille
            // garde sa taille écran quelle que soit l'échelle d'affichage.
            let center = self
                .view
                .screen_of(area, scene_size, Point::new(harmonic.x, harmonic.y));
            let rect = Rect::from_center_size(center, sticker_size());
            if !area.intersects(rect) {
                continue;
            }
            let sticker = self.sticker_texture(ctx, harmonic.note);
            painter.image(sticker.id(), rect, full_uv(), Color32::WHITE);
            // La pastille n'est pas un widget : elle ne doit pas voler le clic
            // destiné à la corde. Le survol est donc testé à la main.
            if pointer.is_some_and(|position| rect.contains(position)) {
                hovered = Some(self.engine.partial_info(harmonic.string, harmonic.partial));
            }
        }

        hovered
    }

    /// Médiators de la barre des notes. Un clic n'est pris en compte que sur un
    /// pixel non transparent, comme le `getImageData(...).data[3] > 0` du Canvas QML.
    fn draw_plectrums(
        &mut self,
        ui: &mut Ui,
        ctx: &Context,
        painter: &Painter,
        area: Rect,
        scene_size: Vec2,
    ) -> bool {
        let mut consumed = false;

        for (note, x, y) in PLECTRUM_LAYOUT {
            let key = format!("mediator_{}", note.sticker());
            let bytes = adh_assets::plectrum(note);
            let size = {
                let decoded = self.textures.decoded(&key, bytes);
                Vec2::new(decoded.width as f32, decoded.height as f32)
            };
            let rect = self.view.scene_rect_anchored(area, scene_size, x, y, size);
            if !area.intersects(rect) {
                continue;
            }
            let visible = self.note_mask.is_visible(note);
            let tint = if visible {
                Color32::WHITE
            } else {
                Color32::from_white_alpha((adh_assets::PLECTRUM_HIDDEN_OPACITY * 255.0) as u8)
            };
            let texture = self.textures.texture(ctx, &key, bytes);
            painter.image(texture.id(), rect, full_uv(), tint);

            let click = ui.interact(rect, Id::new(("plectrum", note.index())), Sense::click());
            if click.clicked() {
                let position = click.interact_pointer_pos().unwrap_or(rect.center());
                let pixel_x = ((position.x - rect.min.x) / rect.width().max(1.0) * size.x) as u32;
                let pixel_y = ((position.y - rect.min.y) / rect.height().max(1.0) * size.y) as u32;
                let alpha = self.textures.decoded(&key, bytes).alpha_at(pixel_x, pixel_y);
                if alpha > adh_assets::HIT_ALPHA_THRESHOLD {
                    self.toggle_note(note);
                    consumed = true;
                }
            }
        }

        consumed
    }
}

impl Atlas {
    /// Molette, glisser, raccourcis clavier et clic sur une corde.
    fn handle_scene_input(
        &mut self,
        ui: &mut Ui,
        ctx: &Context,
        response: &Response,
        area: Rect,
        scene_size: Vec2,
        plectrum_hit: bool,
    ) {
        let scroll = ui.input(|i| i.smooth_scroll_delta().y);
        if scroll.abs() > 0.5 {
            let factor = if scroll > 0.0 {
                1.0 + ZOOM_STEP
            } else {
                1.0 - ZOOM_STEP
            };
            let anchor = ctx.pointer_hover_pos().unwrap_or(area.center());
            self.view.zoom_at(area, scene_size, factor, anchor);
        }

        let delta = response.drag_delta();
        if delta.length_sq() > 0.0 {
            self.view.pan_by(area, scene_size, delta);
        }

        let (zoom_in, zoom_out, fit, clear) = ui.input(|i| {
            (
                i.key_pressed(egui::Key::Plus) || i.key_pressed(egui::Key::Equals),
                i.key_pressed(egui::Key::Minus),
                i.key_pressed(egui::Key::F),
                i.key_pressed(egui::Key::C),
            )
        });
        if zoom_in {
            self.view.zoom_step(area, scene_size, true);
        }
        if zoom_out {
            self.view.zoom_step(area, scene_size, false);
        }
        if fit {
            self.view.fit(area, scene_size);
        }
        if clear {
            self.clear_fingers();
        }

        if response.clicked() && !plectrum_hit {
            if self.debug {
                eprintln!("clic scène reçu à {:?}", response.interact_pointer_pos());
            }
            if let Some(position) = response.interact_pointer_pos() {
                let point = self.view.scene_of(area, scene_size, position);
                if self.debug {
                    eprintln!(
                        "scène ({}, {}) → hit_test {:?}",
                        point.x,
                        point.y,
                        self.engine.hit_test(point.x, point.y)
                    );
                }
                if let Some((string, fret)) = self.engine.hit_test(point.x, point.y) {
                    match self.engine.toggle_fret(string, fret) {
                        adh_core::FretAction::Show { string, .. } => {
                            self.last_action =
                                format!("corde {} : case {}", string + 1, self.engine.fret(string));
                        }
                        adh_core::FretAction::Hide { string } => {
                            self.last_action = format!("corde {} : à vide", string + 1);
                        }
                        adh_core::FretAction::Nothing => self.last_action.clear(),
                    }
                }
            }
        }
    }

    /// Ouvre la fenêtre « Accordage personnalisé… », pré-remplie avec
    /// l'accordage courant.
    fn open_custom_window(&mut self) {
        let tuning = self.engine.tuning();
        self.custom_strings = tuning.raw.clone();
        self.custom_name = String::new();
        self.custom_error = None;
        self.show_custom = true;
    }

    /// Reconstruit les accordages de l'instrument courant : ceux du TOML suivis
    /// des accordages personnalisés, dans l'ordre où ils ont été ajoutés.
    fn rebuild_tunings(&mut self, select: Option<usize>) {
        let id = self.engine.instrument().id().to_string();
        let previous = self.engine.tuning_index();
        let Some(base) = self.instruments.iter().find(|i| i.id() == id).cloned() else {
            return;
        };
        let mut instrument = base;
        for custom in self
            .custom_tunings
            .iter()
            .filter(|custom| custom.instrument == id)
            .cloned()
        {
            if let Err(error) = instrument.add_tuning(&custom.name, &custom.raw) {
                eprintln!("accordage personnalisé ignoré : {error}");
            }
        }
        let count = instrument.tunings().len();
        self.engine.set_instrument(instrument);
        let chosen = select.unwrap_or(previous).min(count.saturating_sub(1));
        self.engine.set_tuning(chosen);
        self.settings.tuning = chosen;
        self.dirty = true;
    }

    /// Fenêtre « Accordage personnalisé… » : nom, cordes, validation, liste.
    fn custom_window(&mut self, ui: &mut Ui, tr: &'static Tr) {
        let string_count = self.engine.instrument().string_count();
        let instrument_id = self.engine.instrument().id().to_string();
        let listed: Vec<(usize, String, String)> = self
            .custom_tunings
            .iter()
            .enumerate()
            .filter(|(_, custom)| custom.instrument == instrument_id)
            .map(|(index, custom)| (index, custom.name.clone(), custom.raw.clone()))
            .collect();
        let mut add = false;
        let mut remove: Option<usize> = None;

        egui::Window::new(tr.custom_tuning)
            .open(&mut self.show_custom)
            .collapsible(false)
            .resizable(false)
            .show(ui.ctx(), |ui| {
                ui.horizontal(|ui| {
                    ui.label(tr.custom_name);
                    ui.text_edit_singleline(&mut self.custom_name);
                });
                ui.horizontal(|ui| {
                    ui.label(tr.custom_strings);
                    ui.text_edit_singleline(&mut self.custom_strings);
                });
                ui.label(format!("{} ({} {})", tr.custom_hint, string_count, tr.strings));
                if let Some(error) = &self.custom_error {
                    ui.colored_label(Color32::RED, error);
                }
                if ui.button(tr.custom_add).clicked() {
                    add = true;
                }
                if !listed.is_empty() {
                    ui.separator();
                    ui.strong(tr.custom_list);
                    for (index, name, raw) in &listed {
                        ui.horizontal(|ui| {
                            ui.label(format!("{name} : {raw}"));
                            if ui.small_button(tr.custom_remove).clicked() {
                                remove = Some(*index);
                            }
                        });
                    }
                }
            });

        if let Some(index) = remove {
            self.custom_tunings.remove(index);
            self.rebuild_tunings(None);
        }
        if add {
            let name = self.custom_name.trim().to_string();
            let raw = self.custom_strings.trim().to_string();
            // Validé par le moteur lui-même, sur une copie de l'instrument.
            match self.engine.instrument().clone().add_tuning(&name, &raw) {
                Ok(()) => {
                    let index = self.engine.instrument().tunings().len();
                    self.custom_tunings.push(CustomTuning {
                        instrument: instrument_id,
                        name,
                        raw,
                    });
                    self.custom_error = None;
                    self.custom_name = String::new();
                    self.rebuild_tunings(Some(index));
                }
                Err(error) => {
                    self.custom_error = Some(error.to_string());
                }
            }
        }
    }

    /// Boîte « À propos ».
    fn about(&mut self, ui: &mut Ui, tr: &'static Tr) {
        let texture = self
            .textures
            .texture(ui.ctx(), "about", adh_assets::ABOUT_IMAGE);
        let body = tr.about_body.to_string();
        egui::Window::new(tr.about)
            .collapsible(false)
            .resizable(false)
            .show(ui.ctx(), |ui| {
                ui.image(&texture);
                ui.label(body);
            });
    }
}

/// Infobulle d'une harmonique : note en grand, fréquence, écart, partiel —
/// même contenu que `HarmonicTooltip.ui.qml`.
fn draw_tooltip(painter: &Painter, at: Pos2, info: &PartialInfo) {
    let rect = Rect::from_min_size(at + Vec2::new(14.0, 14.0), Vec2::new(360.0, 118.0));
    painter.rect(
        rect,
        CornerRadius::same(6),
        Color32::from_black_alpha(205),
        Stroke::new(1.0, Color32::from_gray(40)),
        StrokeKind::Middle,
    );
    painter.text(
        rect.min + Vec2::new(16.0, 24.0),
        Align2::LEFT_CENTER,
        format!("{}{}", info.note.display(), info.octave),
        FontId::proportional(46.0),
        Color32::WHITE,
    );
    let column = Pos2::new(rect.min.x + 140.0, rect.min.y);
    painter.text(
        column + Vec2::new(0.0, 26.0),
        Align2::LEFT_CENTER,
        info.frequency_label(),
        FontId::proportional(19.0),
        Color32::WHITE,
    );
    painter.text(
        column + Vec2::new(0.0, 56.0),
        Align2::LEFT_CENTER,
        info.offset_label(),
        FontId::proportional(16.0),
        Color32::from_gray(220),
    );
    painter.text(
        column + Vec2::new(0.0, 86.0),
        Align2::LEFT_CENTER,
        info.partial_label(),
        FontId::proportional(16.0),
        Color32::from_gray(180),
    );
}

/// egui n'embarque pas les glyphes ♪ ♭ ♮ ♯ : sans ce repli, « C♯ » s'affiche
/// « C□ ». Le sous-ensemble embarqué par `adh-assets` fournit ces quatre signes.
fn install_symbol_font(ctx: &Context) {
    let mut fonts = egui::FontDefinitions::default();
    fonts.font_data.insert(
        "adh_symbols".to_owned(),
        std::sync::Arc::new(egui::FontData::from_static(adh_assets::FONT_SYMBOLS)),
    );
    for family in [egui::FontFamily::Proportional, egui::FontFamily::Monospace] {
        if let Some(list) = fonts.families.get_mut(&family) {
            list.push("adh_symbols".to_owned());
        }
    }
    ctx.set_fonts(fonts);
}
