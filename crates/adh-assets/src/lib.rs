//! Visuels de L'Atlas des harmoniques, embarqués dans le binaire.
//!
//! Remplace le `.qrc` Qt (`Image.qrc`) et ses préfixes `qrc:/harmonics/…` :
//! les images sont incluses au moment de la compilation, l'exécutable reste
//! autonome sur les trois systèmes.
//!
//! **Licence** : ces visuels, réalisés par les auteurs de l'Atlas, sont placés
//! sous la même licence que le code (`MIT OR Apache-2.0`). Voir
//! `LICENSE-assets.md` à la racine du dépôt.

use adh_core::note::Note;

/// Photo d'instrument, par nom de fichier tel que déclaré dans le TOML.
pub const INSTRUMENT_IMAGES: &[(&str, &[u8])] = &[
    ("b4.jpg", include_bytes!("../assets/b4.jpg")),
    ("b5.jpg", include_bytes!("../assets/b5.jpg")),
    ("b6.jpg", include_bytes!("../assets/b6.jpg")),
    ("g6.jpg", include_bytes!("../assets/g6.jpg")),
    ("g7.jpg", include_bytes!("../assets/g7.jpg")),
    ("g8.jpg", include_bytes!("../assets/g8.jpg")),
    ("g12.jpg", include_bytes!("../assets/g12.jpg")),
];

/// Pastille d'harmonique par note (`do.png`, `reb.png`, …).
pub const NOTE_STICKERS: &[(Note, &[u8])] = &[
    (Note::C, include_bytes!("../assets/do.png")),
    (Note::Cs, include_bytes!("../assets/reb.png")),
    (Note::D, include_bytes!("../assets/re.png")),
    (Note::Ds, include_bytes!("../assets/mib.png")),
    (Note::E, include_bytes!("../assets/mi.png")),
    (Note::F, include_bytes!("../assets/fa.png")),
    (Note::Fs, include_bytes!("../assets/solb.png")),
    (Note::G, include_bytes!("../assets/sol.png")),
    (Note::Gs, include_bytes!("../assets/lab.png")),
    (Note::A, include_bytes!("../assets/la.png")),
    (Note::As, include_bytes!("../assets/sib.png")),
    (Note::B, include_bytes!("../assets/si.png")),
];

/// Médiator cliquable par note, utilisé par la barre « Afficher les notes ».
pub const PLECTRUMS: &[(Note, &[u8])] = &[
    (Note::C, include_bytes!("../assets/mediator_do.png")),
    (Note::Cs, include_bytes!("../assets/mediator_reb.png")),
    (Note::D, include_bytes!("../assets/mediator_re.png")),
    (Note::Ds, include_bytes!("../assets/mediator_mib.png")),
    (Note::E, include_bytes!("../assets/mediator_mi.png")),
    (Note::F, include_bytes!("../assets/mediator_fa.png")),
    (Note::Fs, include_bytes!("../assets/mediator_solb.png")),
    (Note::G, include_bytes!("../assets/mediator_sol.png")),
    (Note::Gs, include_bytes!("../assets/mediator_lab.png")),
    (Note::A, include_bytes!("../assets/mediator_la.png")),
    (Note::As, include_bytes!("../assets/mediator_sib.png")),
    (Note::B, include_bytes!("../assets/mediator_si.png")),
];

/// Empreinte de doigt posé sur une frette (SVG d'origine).
pub const THUMBPRINT_SVG: &[u8] = include_bytes!("../assets/thumbprint.svg");
/// Empreinte de doigt rastérisée, dessinée à 70 × 96 unités scène.
pub const THUMBPRINT_PNG: &[u8] = include_bytes!("../assets/thumbprint.png");
/// Logo vectoriel de l'application.
pub const LOGO_SVG: &[u8] = include_bytes!("../assets/logo.svg");
/// Logo rastérisé, utilisé comme icône de fenêtre.
pub const LOGO_PNG: &[u8] = include_bytes!("../assets/logo.png");
/// Image d'accueil affichée avant le choix d'un instrument.
pub const START_IMAGE: &[u8] = include_bytes!("../assets/start.png");
/// Image du dialogue « À propos ».
pub const ABOUT_IMAGE: &[u8] = include_bytes!("../assets/about.png");
/// Icône Windows.
pub const ICON_ICO: &[u8] = include_bytes!("../assets/logo.ico");
/// Icône macOS.
pub const ICON_ICNS: &[u8] = include_bytes!("../assets/ADH.icns");
/// Police de repli pour ♪ ♭ ♮ ♯ : sous-ensemble de Noto Sans Symbols (OFL 1.1),
/// renommé « ADH Symbols ». Les polices egui n'ont pas ces glyphes.
pub const FONT_SYMBOLS: &[u8] = include_bytes!("../assets/fonts/adh-symbols.ttf");

/// Octet de transparence au-delà duquel un médiator accepte le clic
/// (équivalent du `imageData.data[3] > 0` du `Canvas` QML).
pub const HIT_ALPHA_THRESHOLD: u8 = 1;

/// Opacité d'un médiator dont la note est masquée (`MIN_PLECTRUM_OPACITY`).
pub const PLECTRUM_HIDDEN_OPACITY: f32 = 0.35;


/// Photo d'un instrument, par nom de fichier.
pub fn instrument_image(name: &str) -> Option<&'static [u8]> {
    INSTRUMENT_IMAGES
        .iter()
        .find(|(file, _)| *file == name)
        .map(|(_, bytes)| *bytes)
}

/// Pastille d'une note.
pub fn note_sticker(note: Note) -> &'static [u8] {
    NOTE_STICKERS
        .iter()
        .find(|(n, _)| *n == note)
        .map(|(_, bytes)| *bytes)
        .unwrap_or_else(|| NOTE_STICKERS[0].1)
}

/// Médiator d'une note.
pub fn plectrum(note: Note) -> &'static [u8] {
    PLECTRUMS
        .iter()
        .find(|(n, _)| *n == note)
        .map(|(_, bytes)| *bytes)
        .unwrap_or_else(|| PLECTRUMS[0].1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tous_les_visuels_sont_embarques() {
        assert_eq!(INSTRUMENT_IMAGES.len(), 7);
        assert_eq!(NOTE_STICKERS.len(), 12);
        assert_eq!(PLECTRUMS.len(), 12);
        for (_, bytes) in INSTRUMENT_IMAGES {
            assert!(!bytes.is_empty());
        }
        for (_, bytes) in NOTE_STICKERS {
            assert!(!bytes.is_empty());
        }
        for (_, bytes) in PLECTRUMS {
            assert!(!bytes.is_empty());
        }
        assert!(!THUMBPRINT_SVG.is_empty());
        assert!(!THUMBPRINT_PNG.is_empty());
        assert!(!LOGO_SVG.is_empty());
        assert!(!LOGO_PNG.is_empty());
        assert!(!START_IMAGE.is_empty());
        assert!(!ABOUT_IMAGE.is_empty());
        assert!(!ICON_ICO.is_empty());
        assert!(!ICON_ICNS.is_empty());
        assert!(!FONT_SYMBOLS.is_empty());
    }

    #[test]
    fn chaque_photo_declaree_dans_un_toml_existe() {
        for instrument in adh_core::builtin::instruments().unwrap() {
            assert!(
                instrument_image(instrument.image()).is_some(),
                "image {} manquante pour {}",
                instrument.image(),
                instrument.id()
            );
        }
    }

    #[test]
    fn pastilles_et_mediateurs_couvrent_les_douze_notes() {
        for note in Note::ALL {
            assert!(!note_sticker(note).is_empty(), "{}", note.sticker());
            assert!(!plectrum(note).is_empty(), "{}", note.sticker());
        }
    }
}
