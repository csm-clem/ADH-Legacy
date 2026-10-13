//! Modèle d'instrument.
//!
//! Remplace l'architecture « plugin partagé » de l'app Qt
//! (`QLibrary` + symbole `GetInstrument()` + 4 `CMakeLists.txt`) : un instrument
//! de l'Atlas est intégralement décrit par un fichier TOML. Ajouter un
//! instrument = ajouter un fichier, sans recompilation ni ABI partagée.

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::geometry::{
    BEGINING_OF_STRING_X, ENDING_OF_STRING_X, MAX_FRETS, MAX_STRINGS, REFERENCE_FRETS_X,
    SCENE_HEIGHT, SCENE_WIDTH,
};
use crate::tuning::{self, StringSetting, TuningError};

/// Un accordage tel qu'écrit dans le TOML.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TuningDef {
    /// Libellé affiché dans le menu « Accordages ».
    pub name: String,
    /// Jetons du grave vers l'aigu : `"E2 A2 D3 G3 B3 E4"`.
    pub tuning: String,
}

/// Instrument lu depuis le TOML, avant validation.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct InstrumentDef {
    /// Identifiant stable, ex. `"guitar6"`.
    pub id: String,
    /// Nom affiché, ex. `"Guitare 6 cordes"`.
    pub name: String,
    /// Nom du fichier image, ex. `"g6.jpg"`.
    pub image: String,
    /// Nombre de frettes jouables (1..=24).
    pub fret_count: u8,
    /// `[début Y, fin Y]` par corde, corde 0 = la plus aiguë (haut de l'image).
    pub strings: Vec<[f64; 2]>,
    /// Table des abscisses de frettes ; `None` = table de référence.
    #[serde(default)]
    pub fret_positions_x: Option<Vec<f64>>,
    /// Abscisses de début/fin de corde dans le repère de la photo.
    #[serde(default = "default_span")]
    pub string_span_x: [f64; 2],
    /// Taille du canevas dans lequel sont exprimées les coordonnées scène.
    ///
    /// Les photos de référence font 6666 × 1598 px. Une photo plus petite (la
    /// 12 cordes est fournie en 3333 × 799) est étirée sur ce canevas par
    /// l'afficheur : les harmoniques tombent au bon endroit au lieu de déborder
    /// de l'image, ce que l'application Qt ne faisait pas.
    #[serde(default = "default_scene_size")]
    pub scene_size: [f64; 2],
    /// Accordages proposés par l'instrument.
    pub tunings: Vec<TuningDef>,
}

fn default_span() -> [f64; 2] {
    [BEGINING_OF_STRING_X, ENDING_OF_STRING_X]
}

fn default_scene_size() -> [f64; 2] {
    [SCENE_WIDTH, SCENE_HEIGHT]
}

/// Erreurs possibles d'une définition d'instrument.
#[derive(Debug, Error)]
pub enum InstrumentError {
    /// Le TOML n'a pas pu être lu.
    #[error("TOML invalide : {0}")]
    Toml(#[from] toml::de::Error),
    /// Un libellé d'accordage est incorrect.
    #[error("accordage invalide : {0}")]
    Tuning(#[from] TuningError),
    /// Le TOML est lisible mais incohérent (cordes/accordages/frettes).
    #[error("instrument {id} : {detail}")]
    Invalid {
        /// Identifiant de l'instrument fautif.
        id: String,
        /// Détail de l'incohérence.
        detail: String,
    },
}

/// Accordage validé : libellé + réglage résolu par corde.
#[derive(Clone, Debug)]
pub struct Tuning {
    /// Libellé affiché.
    pub name: String,
    /// Chaîne d'origine, telle qu'écrite dans le TOML.
    pub raw: String,
    /// Un élément par corde, index 0 = corde la plus aiguë.
    pub settings: Vec<StringSetting>,
}

/// Instrument validé, prêt à être utilisé par le moteur.
#[derive(Clone, Debug)]
pub struct Instrument {
    id: String,
    name: String,
    image: String,
    fret_count: u8,
    fret_x: Vec<f64>,
    strings: Vec<[f64; 2]>,
    span_x: [f64; 2],
    scene_size: [f64; 2],
    tunings: Vec<Tuning>,
}

impl Instrument {
    /// Construit un instrument depuis son TOML.
    pub fn from_toml(text: &str) -> Result<Self, InstrumentError> {
        Self::from_def(toml::from_str::<InstrumentDef>(text)?)
    }

    /// Construit un instrument depuis une définition déjà désérialisée.
    pub fn from_def(def: InstrumentDef) -> Result<Self, InstrumentError> {
        let invalid = |detail: String| InstrumentError::Invalid {
            id: def.id.clone(),
            detail,
        };

        if def.strings.is_empty() {
            return Err(invalid("aucune corde déclarée".into()));
        }
        if def.strings.len() > MAX_STRINGS {
            return Err(invalid(format!(
                "{} cordes déclarées, maximum {}",
                def.strings.len(),
                MAX_STRINGS
            )));
        }
        if def.fret_count == 0 || def.fret_count as usize > MAX_FRETS {
            return Err(invalid(format!(
                "fret_count = {}, attendu entre 1 et {}",
                def.fret_count, MAX_FRETS
            )));
        }
        if def.tunings.is_empty() {
            return Err(invalid("aucun accordage déclaré".into()));
        }

        let fret_x = match &def.fret_positions_x {
            Some(table) => {
                if table.len() <= def.fret_count as usize {
                    return Err(invalid(format!(
                        "fret_positions_x : {} entrées, il en faut au moins {}",
                        table.len(),
                        def.fret_count as usize + 1
                    )));
                }
                table.clone()
            }
            None => REFERENCE_FRETS_X.to_vec(),
        };

        let mut tunings = Vec::with_capacity(def.tunings.len());
        for def_tuning in &def.tunings {
            let settings = tuning::to_string_settings(&def_tuning.tuning)?;
            if settings.len() != def.strings.len() {
                return Err(invalid(format!(
                    "accordage {:?} décrit {} corde(s) pour un instrument de {} corde(s)",
                    def_tuning.name,
                    settings.len(),
                    def.strings.len()
                )));
            }
            tunings.push(Tuning {
                name: def_tuning.name.clone(),
                raw: def_tuning.tuning.clone(),
                settings,
            });
        }

        Ok(Self {
            id: def.id,
            name: def.name,
            image: def.image,
            fret_count: def.fret_count,
            fret_x,
            strings: def.strings,
            span_x: def.string_span_x,
            scene_size: def.scene_size,
            tunings,
        })
    }

    /// Identifiant stable (`"guitar6"`).
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Nom affiché dans le menu « Instruments ».
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Nom du fichier de la photo d'instrument.
    pub fn image(&self) -> &str {
        &self.image
    }

    /// Nombre de frettes jouables.
    pub fn fret_count(&self) -> u8 {
        self.fret_count
    }

    /// Nombre de cordes.
    pub fn string_count(&self) -> usize {
        self.strings.len()
    }

    /// `[début Y, fin Y]` d'une corde.
    pub fn string_span_y(&self, string: usize) -> [f64; 2] {
        self.strings[string.min(self.strings.len() - 1)]
    }

    /// `[début Y, fin Y]` de toutes les cordes.
    pub fn strings(&self) -> &[[f64; 2]] {
        &self.strings
    }

    /// Abscisses de début et de fin des cordes.
    pub fn span_x(&self) -> [f64; 2] {
        self.span_x
    }

    /// Taille du canevas de référence dans lequel s'expriment les coordonnées
    /// scène. L'afficheur étire la photo sur ce canevas, ce qui permet à une
    /// photo fournie dans une résolution différente (12 cordes : 3333 × 799)
    /// de recevoir les harmoniques aux bonnes abscisses.
    pub fn scene_size(&self) -> [f64; 2] {
        self.scene_size
    }

    /// Accordages validés de l'instrument.
    pub fn tunings(&self) -> &[Tuning] {
        &self.tunings
    }

    /// Ajoute un accordage personnalisé, validé comme ceux du TOML.
    ///
    /// `raw` s'écrit du grave vers l'aigu (`"D2 A2 D3 G3"`) et doit décrire
    /// exactement autant de cordes que l'instrument. Un nom vide est remplacé par
    /// la chaîne elle-même.
    pub fn add_tuning(&mut self, name: &str, raw: &str) -> Result<(), InstrumentError> {
        let raw = raw.trim();
        let settings = tuning::to_string_settings(raw)?;
        if settings.len() != self.strings.len() {
            return Err(InstrumentError::Invalid {
                id: self.id.clone(),
                detail: format!(
                    "accordage {raw:?} décrit {} corde(s) pour un instrument de {} corde(s)",
                    settings.len(),
                    self.strings.len()
                ),
            });
        }
        let name = name.trim();
        self.tunings.push(Tuning {
            name: if name.is_empty() {
                raw.to_string()
            } else {
                name.to_string()
            },
            raw: raw.to_string(),
            settings,
        });
        Ok(())
    }

    /// Abscisse d'une frette (0 = sentinelle « pas de doigt »).
    pub fn fret_x(&self, fret: usize) -> f64 {
        self.fret_x[fret.min(self.fret_x.len() - 1)]
    }
}
