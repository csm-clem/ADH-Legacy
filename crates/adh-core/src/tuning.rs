//! Analyse des chaînes d'accordage.
//!
//! Convention héritée de l'app Qt : la chaîne est écrite **du grave vers
//! l'aigu** (`"E2 A2 D3 G3 B3 E4"`), l'octave du libellé est décrémentée pour
//! obtenir l'octave interne de l'Atlas, puis la liste est **inversée** pour que
//! la corde 0 soit la plus aiguë (celle du haut sur la photo).
//!
//! L'original (`ModelInstrumentManager::_GetTuningFromString`) cherchait les
//! noms de notes avec `QString::contains()` et lisait l'octave à l'index 2 :
//! cela cassait dès qu'une octave dépassait 9 ou qu'un nom en contenait un
//! autre. Le parseur ci-dessous est strict et testé.

use crate::note::Note;
use thiserror::Error;

/// Note + octave interne d'une corde (octave déjà décrémentée).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StringSetting {
    /// Note de la corde à vide.
    pub note: Note,
    /// Octave interne de l'Atlas (libellé − 1).
    pub octave: u32,
}

/// Erreurs possibles d'un libellé d'accordage.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum TuningError {
    /// Chaîne vide.
    #[error("accordage vide")]
    Empty,
    /// Jeton qui n'est pas de la forme `C4`, `F♯3`, `Eb2`…
    #[error("jeton d'accordage invalide : {token:?}")]
    InvalidToken {
        /// Jeton fautif tel qu'écrit.
        token: String,
    },
    /// Octave hors de la plage couverte par la table de fréquences.
    #[error("octave hors limites (0..=9) dans {token:?}")]
    OctaveOutOfRange {
        /// Jeton fautif tel qu'écrit.
        token: String,
    },
}

/// Analyse un jeton isolé : `C4`, `F♯3`, `Eb2`, `B♭1`, `g#0`…
pub fn parse_token(token: &str) -> Result<(Note, u32), TuningError> {
    let token = token.trim();
    let invalid = || TuningError::InvalidToken {
        token: token.to_owned(),
    };
    if token.is_empty() {
        return Err(invalid());
    }

    let mut chars = token.chars();

    // 1. lettre de la note (A..G, majuscule ou minuscule)
    let letter = match chars.next() {
        Some(c) => c.to_ascii_uppercase(),
        None => return Err(invalid()),
    };
    let base = match letter {
        'C' => Note::C,
        'D' => Note::D,
        'E' => Note::E,
        'F' => Note::F,
        'G' => Note::G,
        'A' => Note::A,
        'B' => Note::B,
        _ => return Err(invalid()),
    };

    // 2. altération optionnelle : ♯ # ♭ b
    let mut note = base;
    let mut consumed_accidental = false;
    if let Some(c) = chars.clone().next() {
        match c {
            '♯' | '#' => {
                note = Note::from_index(base.index() + 1);
                consumed_accidental = true;
            }
            '♭' => {
                note = Note::from_index(base.index() + 11);
                consumed_accidental = true;
            }
            // Le 'b' minuscule n'est accepté que s'il ne peut pas être la
            // lettre de la note elle-même (impossible ici : la lettre est déjà
            // consommée à l'étape 1).
            'b' => {
                note = Note::from_index(base.index() + 11);
                consumed_accidental = true;
            }
            _ => {}
        }
        if consumed_accidental {
            chars.next();
        }
    }

    // 3. octave : 1 à 2 chiffres, puis fin du jeton
    let rest: String = chars.collect();
    if rest.is_empty() || !rest.chars().all(|c| c.is_ascii_digit()) {
        return Err(invalid());
    }
    let octave: u32 = rest.parse().map_err(|_| invalid())?;
    if octave > 9 {
        return Err(TuningError::OctaveOutOfRange {
            token: token.to_owned(),
        });
    }

    Ok((note, octave))
}

/// Analyse une chaîne complète et renvoie les notes dans l'ordre du libellé
/// (grave → aigu), octaves du libellé.
pub fn parse_tuning(text: &str) -> Result<Vec<(Note, u32)>, TuningError> {
    let tokens: Vec<&str> = text.split_whitespace().collect();
    if tokens.is_empty() {
        return Err(TuningError::Empty);
    }
    tokens.iter().map(|t| parse_token(t)).collect()
}

/// Convertit une chaîne d'accordage en réglages de cordes indexés corde 0 → N-1
/// (corde 0 = la plus aiguë, en haut de l'image).
pub fn to_string_settings(text: &str) -> Result<Vec<StringSetting>, TuningError> {
    Ok(parse_tuning(text)?
        .into_iter()
        .map(|(note, octave)| StringSetting {
            note,
            octave: if octave > 0 { octave - 1 } else { 0 },
        })
        .rev()
        .collect())
}
