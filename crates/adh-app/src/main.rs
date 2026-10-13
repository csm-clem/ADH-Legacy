//! Point d'entrée de L'Atlas des harmoniques.
//!
//! L'application Qt ouvrait une `ApplicationWindow` de 1920 × 1080 ; la fenêtre
//! egui s'adapte à l'écran et se souvient de sa taille.

mod app;
mod i18n;
mod links;
mod scene;
mod settings;
mod textures;

use eframe::egui;

fn main() -> eframe::Result {
    let viewport = egui::ViewportBuilder::default()
        .with_inner_size([1280.0, 820.0])
        .with_min_inner_size([720.0, 420.0])
        .with_title(i18n::FR.title);
    // Le logo embarqué sert d'icône de fenêtre ; l'application démarre quand même
    // si le PNG est illisible.
    let viewport = match eframe::icon_data::from_png_bytes(adh_assets::LOGO_PNG) {
        Ok(icon) => viewport.with_icon(icon),
        Err(error) => {
            eprintln!("icône du logo ignorée : {}", error);
            viewport
        }
    };

    let options = eframe::NativeOptions {
        viewport,
        renderer: eframe::Renderer::Glow,
        persist_window: true,
        ..Default::default()
    };

    eframe::run_native(
        "L'Atlas des harmoniques",
        options,
        Box::new(|cc| Ok(Box::new(app::Atlas::new(cc)))),
    )
}
