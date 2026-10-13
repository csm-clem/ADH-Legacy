//! État persistant de l'application.
//!
//! L'application Qt ne persistait rien. Le choix d'instrument, d'accordage, les
//! masques et le cadrage sont ici enregistrés dans un TOML du dossier de
//! configuration de l'utilisateur.

use std::io;
use std::path::PathBuf;

use adh_core::masking::{NoteMask, PartialMask};
use adh_core::{Note, Partial};
use serde::{Deserialize, Serialize};

use crate::i18n::Lang;

/// Instrument ouvert en l'absence de fichier de réglages.
pub const DEFAULT_INSTRUMENT: &str = "guitar6";

/// Accordage ajouté par l'utilisateur à un instrument.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CustomTuning {
    /// Instrument auquel il appartient (`"guitar6"`).
    pub instrument: String,
    /// Libellé affiché dans le menu « Accordages ».
    pub name: String,
    /// Jetons du grave vers l'aigu, comme dans le TOML (`"D2 A2 D3 G3"`).
    pub raw: String,
}

/// Ce que l'application se souvient entre deux lancements.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    /// Identifiant de l'instrument (`"guitar6"`).
    pub instrument: String,
    /// Index de l'accordage dans la liste de l'instrument.
    pub tuning: usize,
    /// Langue de l'interface.
    pub lang: Lang,
    /// Visibilité des 12 demi-tons, dans l'ordre `Note::ALL`.
    pub notes: Vec<bool>,
    /// Visibilité des 11 partiels, dans l'ordre `Partial::ALL`.
    pub partials: Vec<bool>,
    /// Niveau de zoom (`0.0` = ajuster à la fenêtre).
    pub zoom: f32,
    /// Décalage de la scène, en pixels écran.
    pub pan: [f32; 2],
    /// Accordages personnalisés, tous instruments confondus.
    pub custom_tunings: Vec<CustomTuning>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            instrument: DEFAULT_INSTRUMENT.to_string(),
            tuning: 0,
            lang: Lang::default(),
            notes: Note::ALL.iter().map(|_| true).collect(),
            partials: Partial::ALL.iter().map(|_| true).collect(),
            zoom: 0.0,
            pan: [0.0, 0.0],
            custom_tunings: Vec::new(),
        }
    }
}

impl Settings {
    /// Masque de notes correspondant aux réglages.
    pub fn note_mask(&self) -> NoteMask {
        let mut mask = NoteMask::all_visible();
        for (index, note) in Note::ALL.iter().enumerate() {
            mask.set(*note, self.notes.get(index).copied().unwrap_or(true));
        }
        mask
    }

    /// Masque de partiels correspondant aux réglages.
    pub fn partial_mask(&self) -> PartialMask {
        let mut mask = PartialMask::all_visible();
        for (index, partial) in Partial::ALL.iter().enumerate() {
            mask.set(*partial, self.partials.get(index).copied().unwrap_or(true));
        }
        mask
    }

    /// Enregistre l'état courant des masques.
    pub fn capture_masks(&mut self, notes: &NoteMask, partials: &PartialMask) {
        self.notes = Note::ALL.iter().map(|note| notes.is_visible(*note)).collect();
        self.partials = Partial::ALL
            .iter()
            .map(|partial| partials.is_visible(*partial))
            .collect();
    }

    /// Chemin du fichier de réglages.
    pub fn path() -> PathBuf {
        config_dir().join("settings.toml")
    }

    /// Charge les réglages, ou fournit les valeurs par défaut.
    pub fn load() -> Self {
        let path = Self::path();
        match std::fs::read_to_string(&path) {
            Ok(text) => toml::from_str::<Settings>(&text).unwrap_or_else(|error| {
                eprintln!("réglages ignorés ({}): {}", path.display(), error);
                Settings::default()
            }),
            Err(_) => Settings::default(),
        }
    }

    /// Écrit les réglages sur le disque.
    pub fn save(&self) -> io::Result<()> {
        let path = Self::path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&path, toml::to_string_pretty(self).unwrap_or_default())
    }
}

/// Dossier de configuration de l'utilisateur.
fn config_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("XDG_CONFIG_HOME") {
        if !dir.is_empty() {
            return PathBuf::from(dir).join("adh");
        }
    }
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .unwrap_or_else(|_| ".".to_string());
    match std::env::consts::OS {
        "macos" => PathBuf::from(home).join("Library/Application Support/adh"),
        "windows" => PathBuf::from(home).join("AppData/Roaming/adh"),
        _ => PathBuf::from(home).join(".config/adh"),
    }
}
