//! Ouverture d'un lien externe.
//!
//! L'application Qt utilisait `Qt.openUrlExternally`. La version `egui` de
//! l'Atlas passe par le lanceur natif, ce qui fonctionne de la même façon sur
//! Linux, macOS et Windows sans dépendre d'un feature additionnel de winit.

/// Ouvre `url` dans le navigateur par défaut du système.
pub fn open(url: &str) {
    let spawned = match std::env::consts::OS {
        "macos" => std::process::Command::new("open").arg(url).spawn(),
        "windows" => std::process::Command::new("cmd")
            .args(["/C", "start", "", url])
            .spawn(),
        _ => std::process::Command::new("xdg-open").arg(url).spawn(),
    };
    if let Err(error) = spawned {
        eprintln!("impossible d'ouvrir {} : {}", url, error);
    }
}