//! Textes de l'interface, français et anglais.
//!
//! L'application historique était traduite via `qsTr`. Les libellés musicaux
//! restent identiques dans les deux langues : les pastilles affichent la
//! notation internationale (`C♯`, `F♯`…) et l'infobulle garde le nom d'intervalle
//! français, qui est le vocabulaire de l'Atlas.

use serde::{Deserialize, Serialize};

/// Langue de l'interface.
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug, Serialize, Deserialize)]
pub enum Lang {
    /// Français (langue d'origine de l'Atlas).
    #[default]
    Fr,
    /// Anglais.
    En,
}

/// Jeu de libellés.
pub struct Tr {
    /// Titre de la fenêtre.
    pub title: &'static str,
    /// Menu « Instruments ».
    pub instruments: &'static str,
    /// Menu « Accordages ».
    pub tunings: &'static str,
    /// Menu « Affichage ».
    pub view: &'static str,
    /// Menu « Aide ».
    pub help: &'static str,
    /// Action « Ajuster à la fenêtre ».
    pub fit: &'static str,
    /// Action « Zoom avant ».
    pub zoom_in: &'static str,
    /// Action « Zoom arrière ».
    pub zoom_out: &'static str,
    /// Action « Échelle 1:1 ».
    pub zoom_full: &'static str,
    /// Action « Retirer tous les doigts ».
    pub clear_fingers: &'static str,
    /// Barre « Afficher les notes ».
    pub show_notes: &'static str,
    /// Barre « Afficher les partiels ».
    pub show_partials: &'static str,
    /// Bouton « Toutes ».
    pub all: &'static str,
    /// Bouton « Aucune ».
    pub none: &'static str,
    /// Choix de la langue.
    pub language: &'static str,
    /// Lien tutoriel.
    pub tutorial: &'static str,
    /// Lien notations latine / anglophone.
    pub notations: &'static str,
    /// Lien série des harmoniques.
    pub serie: &'static str,
    /// Lien notation relative / partiels.
    pub relative: &'static str,
    /// Lien fréquences des notes.
    pub frequencies: &'static str,
    /// Lien soutien.
    pub donation: &'static str,
    /// Entrée « À propos ».
    pub about: &'static str,
    /// Corps de la boîte « À propos ».
    pub about_body: &'static str,
    /// Libellé « cordes ».
    pub strings: &'static str,
    /// Libellé « doigts posés ».
    pub frets: &'static str,
    /// Libellé du niveau de zoom.
    pub zoom: &'static str,
    /// Libellé « accordage ».
    pub tuning: &'static str,
    /// Le cadrage est ajusté à la fenêtre.
    pub fitted: &'static str,
    /// Action « Accordage personnalisé… ».
    pub custom_tuning: &'static str,
    /// Champ « Nom » de l'accordage personnalisé.
    pub custom_name: &'static str,
    /// Champ « Cordes » de l'accordage personnalisé.
    pub custom_strings: &'static str,
    /// Aide : ordre et exemple des jetons.
    pub custom_hint: &'static str,
    /// Bouton « Ajouter ».
    pub custom_add: &'static str,
    /// Bouton « Supprimer ».
    pub custom_remove: &'static str,
    /// Accordages personnalisés de l'instrument.
    pub custom_list: &'static str,
}

/// Libellés français, repris mot pour mot de l'application Qt.
pub const FR: Tr = Tr {
    title: "L'atlas des harmoniques",
    instruments: "Instruments",
    tunings: "Accordages",
    view: "Affichage",
    help: "Aide",
    fit: "Ajuster à la fenêtre",
    zoom_in: "Zoom avant",
    zoom_out: "Zoom arrière",
    zoom_full: "Échelle 1:1",
    clear_fingers: "Retirer tous les doigts",
    show_notes: "Afficher les notes",
    show_partials: "Afficher les partiels",
    all: "Toutes",
    none: "Aucune",
    language: "Langue",
    tutorial: "Tutoriel",
    notations: "Notations latine / anglophone",
    serie: "Série des harmoniques",
    relative: "Notation relative / partiels",
    frequencies: "Fréquences des notes (en hertz)",
    donation: "Cette application vous plait ?",
    about: "A propos de l'Atlas des harmoniques…",
    about_body: "L'Atlas des harmoniques montre, sur une photo d'instrument, \
                 quelle note produit chaque point de pincement et à combien de \
                 cents elle se situe de la note tempérée la plus proche.\n\n\
                 Moteur et interface réécrits en Rust (egui / eframe).\n\
                 Licence du code et des visuels : MIT OR Apache-2.0.\n\
                 https://lecompositeur.com/adh",
    strings: "cordes",
    frets: "doigts posés",
    zoom: "Zoom",
    tuning: "Accordage",
    fitted: "ajusté",
    custom_tuning: "Accordage personnalisé…",
    custom_name: "Nom",
    custom_strings: "Cordes",
    custom_hint: "Du grave vers l'aigu, par exemple « D2 A2 D3 G3 B3 D4 ».",
    custom_add: "Ajouter",
    custom_remove: "Supprimer",
    custom_list: "Accordages personnalisés",
};

/// Libellés anglais.
pub const EN: Tr = Tr {
    title: "The atlas of harmonics",
    instruments: "Instruments",
    tunings: "Tunings",
    view: "View",
    help: "Help",
    fit: "Fit to window",
    zoom_in: "Zoom in",
    zoom_out: "Zoom out",
    zoom_full: "Actual size",
    clear_fingers: "Remove all fingers",
    show_notes: "Show notes",
    show_partials: "Show partials",
    all: "All",
    none: "None",
    language: "Language",
    tutorial: "Tutorial",
    notations: "Latin / English notation",
    serie: "Harmonic series",
    relative: "Relative notation / partials",
    frequencies: "Note frequencies (hertz)",
    donation: "Do you like this app?",
    about: "About the atlas of harmonics…",
    about_body: "The atlas of harmonics shows, on a photograph of an instrument, \
                 which note each plucking point produces and how many cents away \
                 it is from the nearest equal-tempered note.\n\n\
                 Engine and interface rewritten in Rust (egui / eframe).\n\
                 Code and artwork licence: MIT OR Apache-2.0.\n\
                 https://lecompositeur.com/adh",
    strings: "strings",
    frets: "fingers down",
    zoom: "Zoom",
    tuning: "Tuning",
    fitted: "fitted",
    custom_tuning: "Custom tuning…",
    custom_name: "Name",
    custom_strings: "Strings",
    custom_hint: "Low to high, for instance “D2 A2 D3 G3 B3 D4”.",
    custom_add: "Add",
    custom_remove: "Remove",
    custom_list: "Custom tunings",
};

impl Lang {
    /// Jeu de libellés de cette langue.
    pub fn tr(self) -> &'static Tr {
        match self {
            Lang::Fr => &FR,
            Lang::En => &EN,
        }
    }

    /// Nom affiché de la langue.
    pub fn label(self) -> &'static str {
        match self {
            Lang::Fr => "Français",
            Lang::En => "English",
        }
    }
}
