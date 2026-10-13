//! Constantes géométriques de la scène.
//!
//! L'Atlas travaille dans le repère pixel de la photo d'instrument « de
//! référence » (largeur de corde 234 → 6432 px). Ces valeurs étaient dupliquées
//! dans `ModelADHCore` et `ModelHarmonicManager` ; elles sont centralisées ici.

/// Corde la plus longue supportée par le moteur (guitare 12 cordes).
pub const MAX_STRINGS: usize = 12;

/// Nombre de positions de frettes disponibles dans la table de référence.
pub const REFERENCE_FRETS_COUNT: usize = 25;

/// Nombre maximal de frettes qu'un instrument peut déclarer.
pub const MAX_FRETS: usize = 24;

/// Abscisse du début des cordes (sillet / extrémité jouée).
pub const BEGINING_OF_STRING_X: f64 = 234.0;

/// Abscisse de la fin des cordes (chevalet).
pub const ENDING_OF_STRING_X: f64 = 6432.0;

/// Largeur du canevas de référence : les coordonnées scène sont exprimées dans
/// les pixels de la photo d'instrument « de référence » (6666 × 1598).
pub const SCENE_WIDTH: f64 = 6666.0;

/// Hauteur du canevas de référence.
pub const SCENE_HEIGHT: f64 = 1598.0;

/// Largeur de l'empreinte de doigt (`thumbprint.svg`).
pub const THUMBPRINT_WIDTH: f64 = 70.0;

/// Hauteur de l'empreinte de doigt (`thumbprint.svg`).
pub const THUMBPRINT_HEIGHT: f64 = 96.0;

/// Tolérance horizontale pour rattacher un clic à la frette suivante.
pub const FRET_HIT_MARGIN: f64 = 100.0;

/// Positions absolues des frettes dans le repère de référence.
///
/// L'index 0 vaut 0 et sert de sentinelle « aucun doigt posé ». Les index 1..=24
/// sont les frettes réelles. Un instrument peut fournir sa propre table via
/// `fret_positions_x` dans son TOML (corrige le défaut de l'original, qui
/// imposait cette table globale à tous les instruments quelle que soit leur
/// échelle).
/// Les coordonnées de la scène sont en `f64` pour reproduire exactement les
/// calculs `qreal` (double) du moteur C++ historique.
pub const REFERENCE_FRETS_X: [f64; REFERENCE_FRETS_COUNT] = [
    0.0, 486.0, 814.0, 1118.0, 1418.0, 1694.0, 1946.0, 2198.0, 2434.0, 2648.0, 2856.0, 3054.0,
    3254.0, 3414.0, 3586.0, 3738.0, 3894.0, 4028.0, 4160.0, 4276.0, 4396.0, 4515.0, 4624.0, 4710.0,
    4804.0,
];

/// Point du repère scène.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Point {
    /// Abscisse, en pixels du repère de référence.
    pub x: f64,
    /// Ordonnée, en pixels du repère de référence.
    pub y: f64,
}

impl Point {
    /// Construit un point.
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
}
