//! Instruments livrés avec l'Atlas.
//!
//! Les définitions sont des fichiers TOML embarqués au moment de la compilation
//! (`include_str!`) : l'application n'a pas besoin de dossier de ressources à
//! côté de l'exécutable, ce qui supprime toute la logique de `_GetResourcesPath()`
//! et de déploiement des `.so`/`.dylib`/`.dll` d'instruments.

use crate::instrument::{Instrument, InstrumentError};

/// `(id, TOML)` de chaque instrument embarqué, dans l'ordre d'affichage du menu.
pub const BUILTIN_TOMLS: &[(&str, &str)] = &[
    ("bass4", include_str!("../assets/instruments/bass4.toml")),
    ("bass5", include_str!("../assets/instruments/bass5.toml")),
    ("bass6", include_str!("../assets/instruments/bass6.toml")),
    (
        "guitar6",
        include_str!("../assets/instruments/guitar6.toml"),
    ),
    (
        "guitar7",
        include_str!("../assets/instruments/guitar7.toml"),
    ),
    (
        "guitar8",
        include_str!("../assets/instruments/guitar8.toml"),
    ),
    (
        "guitar12",
        include_str!("../assets/instruments/guitar12.toml"),
    ),
];

/// Construit tous les instruments embarqués.
pub fn instruments() -> Result<Vec<Instrument>, InstrumentError> {
    BUILTIN_TOMLS
        .iter()
        .map(|(_, toml)| Instrument::from_toml(toml))
        .collect()
}

/// TOML brut d'un instrument, par id (`"guitar6"`, …).
pub fn toml_of(id: &str) -> Option<&'static str> {
    BUILTIN_TOMLS
        .iter()
        .find(|(instrument_id, _)| *instrument_id == id)
        .map(|(_, toml)| *toml)
}
