//! Les 11 partiels (harmoniques) dessinées par l'Atlas.
//!
//! L'enum Qt `partials::name` allait de `DEMI` (1/2) à `DOUZIEME` (1/12).
//! Le « 1/1 » (la corde entière) n'est pas une harmonique : l'index 0 correspond
//! donc au demi de la corde, et la valeur du partiel vaut `index + 2`.

use serde::{Deserialize, Serialize};

/// Nombre de partiels gérés par l'Atlas.
pub const PARTIAL_COUNT: usize = 11;

/// Un partiel (harmonique) de la corde, de 1/2 à 1/12.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize)]
pub enum Partial {
    /// 1/2 — octave
    Half,
    /// 1/3 — quinte de l'octave
    Third,
    /// 1/4 — 2e octave
    Quarter,
    /// 1/5 — tierce majeure majeure de la 3e octave
    Fifth,
    /// 1/6 — 3e quinte
    Sixth,
    /// 1/7 — 3e septième mineure
    Seventh,
    /// 1/8 — 3e octave
    Eighth,
    /// 1/9 — 4e seconde majeure
    Ninth,
    /// 1/10 — 4e tierce majeure
    Tenth,
    /// 1/11 — 4e quarte augmentée
    Eleventh,
    /// 1/12 — 4e quinte
    Twelfth,
}

impl Partial {
    /// Tous les partiels, du plus grave (1/2) au plus aigu (1/12).
    pub const ALL: [Partial; PARTIAL_COUNT] = [
        Partial::Half,
        Partial::Third,
        Partial::Quarter,
        Partial::Fifth,
        Partial::Sixth,
        Partial::Seventh,
        Partial::Eighth,
        Partial::Ninth,
        Partial::Tenth,
        Partial::Eleventh,
        Partial::Twelfth,
    ];

    /// Index de l'enum (0 = 1/2 … 10 = 1/12).
    pub fn index(self) -> usize {
        self as usize
    }

    /// Partiel à partir d'un index, avec repli modulo 11.
    pub fn from_index(index: usize) -> Self {
        Self::ALL[index % PARTIAL_COUNT]
    }

    /// Dénominateur réel du partiel : 2 pour [`Partial::Half`], 12 pour [`Partial::Twelfth`].
    pub fn value(self) -> u32 {
        self.index() as u32 + 2
    }

    /// Libellé « 1/N » affiché dans l'infobulle.
    pub fn ratio_label(self) -> String {
        format!("1/{}", self.value())
    }

    /// Nom de l'intervalle par rapport à la corde, identique à
    /// `harmonics::partialOctaveName` du C++.
    pub fn interval_name_fr(self) -> &'static str {
        match self {
            Partial::Half => "1ère octave (P1)",
            Partial::Third => "2e quinte (P5)",
            Partial::Quarter => "2e octave (P1)",
            Partial::Fifth => "3e tierce majeure (M3)",
            Partial::Sixth => "3e quinte (P5)",
            Partial::Seventh => "3e septième mineure (m7)",
            Partial::Eighth => "3ère octave (P1)",
            Partial::Ninth => "4e seconde majeure (M2)",
            Partial::Tenth => "4e tierce majeure (M3)",
            Partial::Eleventh => "4e quarte augmentée (A4)",
            Partial::Twelfth => "4e quinte (P5)",
        }
    }

    /// Libellé complet d'infobulle, identique à `t_partialInfo::getPartial()`.
    pub fn partial_label(self) -> String {
        format!(
            "Partiel {}, {}",
            self.ratio_label(),
            self.interval_name_fr()
        )
    }

    /// Occurrences de ce partiel sur la corde, avant dédoublonnage.
    ///
    /// Un partiel 1/N se joue N-1 fois sur la corde (tous les multiples de 1/N
    /// strictement inférieurs à 1).
    pub fn occurrences(self) -> u32 {
        self.value() - 1
    }

    /// Occurrences réellement affichées, après suppression des harmoniques
    /// qui en masquent d'autres (ex. 1/4 recouvre 1/2).
    pub fn visible_occurrences(self) -> u32 {
        (1..self.value())
            .filter(|h| !is_masked(self.value(), *h))
            .count() as u32
    }
}

/// Une harmonique « simple » (numérateur `numerator`) est masquée par le partiel
/// `partial_value` si elle est réductible à un partiel déjà dessiné : le 1/4 ne
/// doit pas faire réapparaître le 1/2, le 1/6 ni le 1/3, etc.
///
/// Condition reproduite ligne à ligne depuis
/// `ModelHarmonicManager::ComputeAllHarmonics()` : les `% n == 0` sont gardés
/// tels quels pour rester identiques au C++ d'origine.
#[allow(clippy::manual_is_multiple_of)]
pub fn is_masked(partial_value: u32, numerator: u32) -> bool {
    (partial_value == 4 && numerator == 2)
        || (partial_value == 6 && (numerator % 2 == 0 || numerator == 3))
        || (partial_value == 8 && numerator % 2 == 0)
        || (partial_value == 9 && numerator % 3 == 0)
        || (partial_value == 10 && (numerator % 2 == 0 || numerator == 5))
        || (partial_value == 12 && (numerator % 2 == 0 || numerator % 3 == 0))
}

impl std::fmt::Display for Partial {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.ratio_label())
    }
}
