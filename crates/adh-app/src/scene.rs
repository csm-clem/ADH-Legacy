//! Cadrage de la scène : zoom, déplacement, conversion écran ↔ repère scène.
//!
//! Le repère scène est celui de la photo de référence (6666 × 1598 px) dans
//! lequel le moteur calcule les positions des harmoniques. `View` ne fait que
//! transposer ce repère à l'écran, avec les mêmes règles que le `Flickable` de
//! l'application Qt : ajustement à la fenêtre, zoom par pas de 10 %, contenu
//! maintenu dans les bords.

use adh_core::Note;
use adh_core::Point;
use egui::{Pos2, Rect, Vec2};

/// Taille d'une pastille à l'écran, en pixels.
///
/// L'application Qt dessinait chaque pastille avec `scale = 1 / scale_image` :
/// une pastille gardait donc 16 × 26 px quelle que soit la taille d'affichage.
pub fn sticker_size() -> Vec2 {
    Vec2::new(16.0, 26.0)
}

/// Emprise de l'empreinte de doigt, en unités scène (`thumbprint.svg` : 70 × 96).
pub fn thumbprint_size() -> Vec2 {
    Vec2::new(70.0, 96.0)
}

/// Pas de zoom d'un cran (comme `zoomStep` du QML).
pub const ZOOM_STEP: f32 = 0.1;

/// Zoom maximal au-delà de la taille réelle de la photo.
pub const MAX_ZOOM_FACTOR: f32 = 4.0;

/// Position et ordre de dessin d'un médiator, en unités scène.
///
/// Repris de `Plectrums.qml`, dont les coordonnées étaient doublées pour la
/// photo 6666 × 1598. L'ordre suit les `z` du QML, pour que les recouvrements
/// soient les mêmes.
pub const PLECTRUM_LAYOUT: [(Note, f32, f32); 12] = [
    (Note::A, 98.0, 1278.0),
    (Note::As, 254.0, 1284.0),
    (Note::Cs, 868.0, 1356.0),
    (Note::Ds, 1142.0, 1294.0),
    (Note::D, 1008.0, 1234.0),
    (Note::C, 682.0, 1292.0),
    (Note::B, 510.0, 1266.0),
    (Note::E, 1334.0, 1272.0),
    (Note::F, 1540.0, 1336.0),
    (Note::Gs, 2118.0, 1194.0),
    (Note::G, 1960.0, 1286.0),
    (Note::Fs, 1752.0, 1298.0),
];

/// Cadrage courant de la scène.
#[derive(Clone, Copy, Debug)]
pub struct View {
    /// Rapport pixels scène → pixels écran.
    zoom: f32,
    /// Décalage du centre de la scène, en pixels écran.
    pan: Vec2,
    /// `true` tant que l'utilisateur n'a pas zoomé : la scène est ajustée.
    fit_active: bool,
}

impl Default for View {
    fn default() -> Self {
        Self::new()
    }
}

impl View {
    /// Cadrage « ajusté à la fenêtre ».
    pub fn new() -> Self {
        Self {
            zoom: 0.0,
            pan: Vec2::ZERO,
            fit_active: true,
        }
    }

    /// Restaure un cadrage enregistré. `zoom <= 0` relance l'ajustement.
    pub fn from_saved(zoom: f32, pan: [f32; 2]) -> Self {
        Self {
            zoom,
            pan: Vec2::new(pan[0], pan[1]),
            fit_active: zoom <= 0.0,
        }
    }

    /// Zoom et décalage à enregistrer.
    pub fn saved(&self) -> (f32, [f32; 2]) {
        if self.fit_active {
            (0.0, [0.0, 0.0])
        } else {
            (self.zoom, [self.pan.x, self.pan.y])
        }
    }

    /// Zoom qui fait tenir toute la scène dans `area`.
    pub fn fit_zoom(area: Rect, scene: Vec2) -> f32 {
        (area.width() / scene.x).min(area.height() / scene.y).min(1.0)
    }

    /// Applique l'ajustement si l'utilisateur n'a pas zoomé.
    pub fn apply_fit(&mut self, area: Rect, scene: Vec2) {
        if self.fit_active || self.zoom <= 0.0 {
            self.zoom = Self::fit_zoom(area, scene).max(1.0e-4);
            self.pan = Vec2::ZERO;
        }
        self.clamp(area, scene);
    }

    /// Rectangle écran de la photo.
    pub fn image_rect(&self, area: Rect, scene: Vec2) -> Rect {
        Rect::from_center_size(area.center() + self.pan, scene * self.zoom)
    }

    /// Zoom courant.
    pub fn zoom(&self) -> f32 {
        self.zoom
    }

    /// Vrai si la scène est ajustée à la fenêtre.
    pub fn is_fit(&self) -> bool {
        self.fit_active
    }

    /// Repère scène → écran.
    pub fn screen_of(&self, area: Rect, scene: Vec2, point: Point) -> Pos2 {
        let rect = self.image_rect(area, scene);
        Pos2::new(
            rect.min.x + point.x as f32 * self.zoom,
            rect.min.y + point.y as f32 * self.zoom,
        )
    }

    /// Écran → repère scène.
    pub fn scene_of(&self, area: Rect, scene: Vec2, pos: Pos2) -> Point {
        let rect = self.image_rect(area, scene);
        Point::new(
            (pos.x - rect.min.x) as f64 / self.zoom as f64,
            (pos.y - rect.min.y) as f64 / self.zoom as f64,
        )
    }

    /// Rectangle écran d'un objet de taille `size` en unités scène, centré sur `point`.
    pub fn scene_rect(&self, area: Rect, scene: Vec2, point: Point, size: Vec2) -> Rect {
        Rect::from_center_size(self.screen_of(area, scene, point), size * self.zoom)
    }

    /// Rectangle écran d'un objet de taille `size` en unités scène, ancré en haut à gauche.
    pub fn scene_rect_anchored(&self, area: Rect, scene: Vec2, x: f32, y: f32, size: Vec2) -> Rect {
        let rect = self.image_rect(area, scene);
        let min = Pos2::new(rect.min.x + x * self.zoom, rect.min.y + y * self.zoom);
        Rect::from_min_size(min, size * self.zoom)
    }

    /// Zoom d'un cran autour d'un point écran : la scène sous la souris ne bouge pas.
    pub fn zoom_at(&mut self, area: Rect, scene: Vec2, factor: f32, anchor: Pos2) {
        let anchor_point = self.scene_of(area, scene, anchor);
        let fit = Self::fit_zoom(area, scene);
        self.zoom = (self.zoom * factor).clamp(fit, fit * MAX_ZOOM_FACTOR);
        self.fit_active = false;
        self.pan += anchor - self.screen_of(area, scene, anchor_point);
        self.clamp(area, scene);
    }

    /// Zoom d'un cran autour du centre de la zone.
    pub fn zoom_step(&mut self, area: Rect, scene: Vec2, forward: bool) {
        let factor = if forward {
            1.0 + ZOOM_STEP
        } else {
            1.0 - ZOOM_STEP
        };
        self.zoom_at(area, scene, factor, area.center());
    }

    /// Ramène la scène dans la fenêtre.
    pub fn fit(&mut self, area: Rect, scene: Vec2) {
        self.fit_active = true;
        self.apply_fit(area, scene);
    }

    /// Échelle 1:1 (`zoomFull` du QML).
    pub fn actual_size(&mut self, area: Rect, scene: Vec2) {
        self.zoom = 1.0;
        self.fit_active = false;
        self.pan = Vec2::ZERO;
        self.clamp(area, scene);
    }

    /// Déplace la scène.
    pub fn pan_by(&mut self, area: Rect, scene: Vec2, delta: Vec2) {
        self.pan += delta;
        self.clamp(area, scene);
    }

    /// `returnToBounds()` du QML : pas de marge vide, contenu jamais hors fenêtre.
    fn clamp(&mut self, area: Rect, scene: Vec2) {
        let slack = (area.size() - scene * self.zoom) / 2.0;
        self.pan.x = self.pan.x.clamp(slack.x.min(-slack.x), slack.x.max(-slack.x));
        self.pan.y = self.pan.y.clamp(slack.y.min(-slack.y), slack.y.max(-slack.y));
    }
}
