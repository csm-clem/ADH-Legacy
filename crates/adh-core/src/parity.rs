//! Empreinte de la table complète des harmoniques.
//!
//! Héritée de la vérification contre le moteur Qt historique : les deux
//! implémentations devaient produire la même empreinte sur le même jeu
//! d'instruments, d'accordages et de frettes. Elle sert aujourd'hui de garde-fou
//! contre toute régression du moteur Rust.

use crate::harmonics::{Harmonic, HarmonicEngine, PartialInfo};
use crate::instrument::Instrument;

const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x1000_0000_01b3;
const GOLDEN_RATIO: u64 = 0x9e37_79b9_7f4a_7c15;

/// Empreinte FNV-1a de toutes les harmoniques calculées pour `instruments`,
/// corde par corde, accordage par accordage, pour chaque frette de `frets`.
///
/// Les valeurs flottantes sont arrondies au centième avant d'être mélangées,
/// ce qui rend l'empreinte insensible au formatage mais sensible à tout écart
/// de calcul.
pub fn checksum(instruments: &[Instrument], frets: &[u8]) -> u64 {
    let mut hash = FNV_OFFSET;
    let mut mix = |value: i64| {
        hash ^= (value as u64).wrapping_add(GOLDEN_RATIO);
        hash = hash.wrapping_mul(FNV_PRIME);
    };

    for instrument in instruments {
        for tuning_index in 0..instrument.tunings().len() {
            let mut engine = HarmonicEngine::new(instrument.clone());
            engine.set_tuning(tuning_index);
            for &fret in frets {
                let fret = fret.min(instrument.fret_count());
                for string in 0..instrument.string_count() {
                    if fret > 0 {
                        engine.toggle_fret(string, fret);
                    }
                    for harmonic in engine.harmonics(string) {
                        let info = engine.partial_info(string, harmonic.partial);
                        mix(i64::from(harmonic.partial.value()));
                        mix((harmonic.x * 100.0).round() as i64);
                        mix((harmonic.y * 100.0).round() as i64);
                        mix((info.frequency * 100.0).round() as i64);
                        mix(i64::from(info.note.index() as u32));
                        mix(i64::from(info.octave));
                        mix(i64::from(info.cents));
                        mix((info.offset_hz * 100.0).round() as i64);
                    }
                    if fret > 0 {
                        engine.toggle_fret(string, fret);
                    }
                }
            }
        }
    }
    hash
}

/// Ligne CSV d'une harmonique.
///
/// Format partagé par le dump Rust et les dumps de référence gelés dans
/// `tests/golden/` : deux décimales sur les positions et les fréquences.
pub fn csv_row(
    instrument_id: &str,
    tuning_index: usize,
    string: usize,
    fret: u8,
    harmonic: &Harmonic,
    info: &PartialInfo,
) -> String {
    format!(
        "{},{},{},{},{},{},{:.2},{:.2},{:.2},{},{},{},{:.2}",
        instrument_id,
        tuning_index,
        string,
        fret,
        harmonic.partial.ratio_label(),
        harmonic.partial.value(),
        harmonic.x,
        harmonic.y,
        info.frequency,
        info.note.display(),
        info.octave,
        info.cents,
        info.offset_hz
    )
}

/// En-tête des dumps CSV.
pub const CSV_HEADER: &str =
    "instrument,tuning,string,fret,partial,partial_value,x,y,frequency,note,octave,cents,offset_hz";
