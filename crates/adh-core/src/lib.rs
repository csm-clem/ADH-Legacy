//! # adh-core
//!
//! Moteur de L'Atlas des harmoniques, sans aucune dépendance GUI.
//!
//! Tout ce que savait faire la couche modèle Qt (calcul des fréquences,
//! détection de la note tempérée la plus proche, écart en cents, positions des
//! pastilles, hit-test d'un clic, accordages, filtres de visibilité) tient ici
//! et est testable sans ouvrir de fenêtre.
//!
//! ## Exemple
//!
//! ```
//! use adh_core::{HarmonicEngine, Note, Partial, builtin::instruments};
//!
//! let guitar6 = instruments()
//!     .unwrap()
//!     .into_iter()
//!     .find(|i| i.id() == "guitar6")
//!     .unwrap();
//! let engine = HarmonicEngine::new(guitar6);
//!
//! // Corde de Mi aigu (corde 0), harmonique 1/2 : une octave au-dessus.
//! let info = engine.partial_info(0, Partial::Half);
//! assert_eq!(info.note, Note::E);
//! assert!((info.frequency - 659.26).abs() < 0.01);
//! assert!(info.cents.abs() <= 1);
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod frequency;
pub mod geometry;
pub mod harmonics;
pub mod instrument;
pub mod masking;
pub mod note;
pub mod parity;
pub mod partial;
pub mod tuning;

#[cfg(feature = "builtin")]
pub mod builtin;

pub use frequency::{FrequencyTable, LA_FREQUENCY};
pub use geometry::Point;
pub use harmonics::{FretAction, Harmonic, HarmonicEngine, PartialInfo};
pub use instrument::{Instrument, InstrumentDef, InstrumentError, Tuning, TuningDef};
pub use masking::{NoteMask, PartialMask};
pub use note::Note;
pub use partial::Partial;
