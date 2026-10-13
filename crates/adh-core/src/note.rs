//! Types de base de L'Atlas des harmoniques : les 12 demi-tons de la gamme
//! tempérée.
//!
//! L'application historique utilisait des noms français en interne
//! (`DO, DOd, RE, REd, MI, FA, FAd, SOL, SOLd, LA, LAd, SI`) et des noms
//! internationaux à l'affichage (`C, C♯, D, D♯, E, F, F♯, G, G♯, A, A♯, B`).
//! Les deux sont conservés : [`Note::display`] pour l'affichage,
//! [`Note::display_fr`] pour l'aide-mémoire francophone, et
//! [`Note::sticker`] pour retrouver le nom du sprite PNG d'origine.

use serde::{Deserialize, Serialize};

/// Nombre de demi-tons de la gamme tempérée.
pub const NOTE_COUNT: usize = 12;

/// Un demi-ton de la gamme tempérée, nommé à partir de Do.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize)]
pub enum Note {
    /// Do
    C,
    /// Do♯ / Ré♭
    Cs,
    /// Ré
    D,
    /// Ré♯ / Mi♭
    Ds,
    /// Mi
    E,
    /// Fa
    F,
    /// Fa♯ / Sol♭
    Fs,
    /// Sol
    G,
    /// Sol♯ / La♭
    Gs,
    /// La
    A,
    /// La♯ / Si♭
    As,
    /// Si
    B,
}

impl Note {
    /// Tous les demi-tons, dans l'ordre ascendant à partir de Do.
    pub const ALL: [Note; NOTE_COUNT] = [
        Note::C,
        Note::Cs,
        Note::D,
        Note::Ds,
        Note::E,
        Note::F,
        Note::Fs,
        Note::G,
        Note::Gs,
        Note::A,
        Note::As,
        Note::B,
    ];

    /// Index du demi-ton (0 = Do … 11 = Si), identique à l'enum Qt `notes::name`.
    pub fn index(self) -> usize {
        self as usize
    }

    /// Demi-ton à partir d'un index, avec repli modulo 12.
    pub fn from_index(index: usize) -> Self {
        Self::ALL[index % NOTE_COUNT]
    }

    /// Nom affiché (international), identique à `harmonics::noteName` du C++.
    pub fn display(self) -> &'static str {
        match self {
            Note::C => "C",
            Note::Cs => "C♯",
            Note::D => "D",
            Note::Ds => "D♯",
            Note::E => "E",
            Note::F => "F",
            Note::Fs => "F♯",
            Note::G => "G",
            Note::Gs => "G♯",
            Note::A => "A",
            Note::As => "A♯",
            Note::B => "B",
        }
    }

    /// Nom francophone (notation latine).
    pub fn display_fr(self) -> &'static str {
        match self {
            Note::C => "Do",
            Note::Cs => "Do♯",
            Note::D => "Ré",
            Note::Ds => "Ré♯",
            Note::E => "Mi",
            Note::F => "Fa",
            Note::Fs => "Fa♯",
            Note::G => "Sol",
            Note::Gs => "Sol♯",
            Note::A => "La",
            Note::As => "La♯",
            Note::B => "Si",
        }
    }

    /// Nom du sprite de pastille d'origine (`do.png`, `reb.png`, …).
    ///
    /// L'app historique nomme les altérations du côté bémol : `Cs` est dessiné
    /// avec `reb.png`, `Ds` avec `mib.png`, etc.
    pub fn sticker(self) -> &'static str {
        match self {
            Note::C => "do",
            Note::Cs => "reb",
            Note::D => "re",
            Note::Ds => "mib",
            Note::E => "mi",
            Note::F => "fa",
            Note::Fs => "solb",
            Note::G => "sol",
            Note::Gs => "lab",
            Note::A => "la",
            Note::As => "sib",
            Note::B => "si",
        }
    }

    /// Monte de `semitones` demi-tons et renvoie la note obtenue ainsi que le
    /// nombre d'octaves franchies.
    pub fn add_semitones(self, semitones: u32) -> (Note, u32) {
        let total = self.index() as u32 + semitones;
        (
            Self::from_index((total % NOTE_COUNT as u32) as usize),
            total / NOTE_COUNT as u32,
        )
    }
}

impl std::fmt::Display for Note {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.display())
    }
}
