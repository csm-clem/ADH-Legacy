//! Table des fréquences du tempérament égal, portée par `LA_FREQUENCY` (440 Hz).
//!
//! Portage exact de `ModelHarmonicManager::ComputeTableOfFrequencies()` :
//! l'octave 0 de l'Atlas commence 45 demi-tons sous le La 440, ce qui place le
//! Do de l'octave 0 à 32,703 Hz (le La de l'octave 3 est exactement 440 Hz).

use crate::note::NOTE_COUNT;

/// Diapason de référence.
pub const LA_FREQUENCY: f64 = 440.0;

/// Nombre d'octaves couvertes par l'Atlas (0..=8).
pub const OCTAVE_COUNT: usize = 9;

/// Fréquence d'un demi-ton, en ratio.
pub const SEMITONE_RATIO: f64 = 1.059_463_094_359_831;

/// Table octave × demi-ton des fréquences du tempérament égal.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FrequencyTable {
    data: [[f64; NOTE_COUNT]; OCTAVE_COUNT],
}

impl FrequencyTable {
    /// Construit la table `440 * 2^(n/12)` pour n = -45 … 62.
    pub fn new() -> Self {
        let ratio = 2f64.powf(1.0 / 12.0);
        let mut offset = -45.0;
        let mut data = [[0.0; NOTE_COUNT]; OCTAVE_COUNT];
        // Les index ont un sens musical (octave, puis demi-ton) : ils sont gardés.
        #[allow(clippy::needless_range_loop)]
        for octave in 0..OCTAVE_COUNT {
            #[allow(clippy::needless_range_loop)]
            for note in 0..NOTE_COUNT {
                data[octave][note] = LA_FREQUENCY * ratio.powf(offset);
                offset += 1.0;
            }
        }
        Self { data }
    }

    /// Fréquence d'une note dans une octave, avec bornage de sécurité
    /// (l'original C++ accédait aux tableaux sans contrôle).
    pub fn get(&self, octave: usize, note: usize) -> f64 {
        self.data[octave.min(OCTAVE_COUNT - 1)][note % NOTE_COUNT]
    }

    /// Fréquence la plus haute de la table (Si octave 8).
    pub fn max(&self) -> f64 {
        self.data[OCTAVE_COUNT - 1][NOTE_COUNT - 1]
    }

    /// Fréquence la plus basse de la table (Do octave 0).
    pub fn min(&self) -> f64 {
        self.data[0][0]
    }
}

impl Default for FrequencyTable {
    fn default() -> Self {
        Self::new()
    }
}
