//! Moteur des harmoniques : fréquences, notes obtenues, écarts en cents et
//! positions à l'écran.
//!
//! Portage fidèle de `ModelHarmonicManager` + de la hit-test de
//! `ModelADHCore::userClickedOnScene`. Les `Q_PROPERTY`/signaux n'ont pas
//! d'équivalent : l'UI relit cet état à chaque image.

use crate::frequency::{FrequencyTable, OCTAVE_COUNT};
use crate::geometry::{Point, FRET_HIT_MARGIN, THUMBPRINT_HEIGHT, THUMBPRINT_WIDTH};
use crate::instrument::{Instrument, Tuning};
use crate::note::{Note, NOTE_COUNT};
use crate::partial::{self, Partial, PARTIAL_COUNT};

/// Détail d'un partiel sur une corde donnée : c'est le contenu de l'infobulle.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PartialInfo {
    /// Partiel concerné.
    pub partial: Partial,
    /// Fréquence réelle de l'harmonique, arrondie au centième de Hz.
    pub frequency: f64,
    /// Note tempérée la plus proche.
    pub note: Note,
    /// Octave de cette note (numérotation interne de l'Atlas).
    pub octave: u32,
    /// Écart avec la note tempérée, en cents (tronqué, comme l'original).
    pub cents: i32,
    /// Écart avec la note tempérée, en Hz (arrondi au centième).
    pub offset_hz: f64,
}

impl Default for PartialInfo {
    fn default() -> Self {
        Self {
            partial: Partial::Half,
            frequency: 0.0,
            note: Note::C,
            octave: 0,
            cents: 0,
            offset_hz: 0.0,
        }
    }
}

impl PartialInfo {
    /// Ex. `329.63Hz` (`t_partialInfo::getFrequency`).
    pub fn frequency_label(&self) -> String {
        format!("{}Hz", self.frequency)
    }

    /// Ex. `1 cents de B4 (1.12Hz)` (`t_partialInfo::getOctave`).
    pub fn offset_label(&self) -> String {
        format!(
            "{} cents de {}{} ({}Hz)",
            self.cents,
            self.note.display(),
            self.octave,
            self.offset_hz
        )
    }

    /// Ex. `Partiel 1/3, 2e quinte (P5)` (`t_partialInfo::getPartial`).
    pub fn partial_label(&self) -> String {
        self.partial.partial_label()
    }
}

/// Une pastille à dessiner : position + note obtenue.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Harmonic {
    /// Corde (0 = la plus aiguë).
    pub string: usize,
    /// Note tempérée la plus proche de l'harmonique.
    pub note: Note,
    /// Partiel dessiné.
    pub partial: Partial,
    /// Abscisse de la pastille, en pixels du repère scène.
    pub x: f64,
    /// Ordonnée de la pastille, en pixels du repère scène.
    pub y: f64,
}

/// Conséquence d'un clic sur la scène, à répercuter dans l'UI.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FretAction {
    /// Poser le doigt : afficher l'empreinte en (x, y).
    Show {
        /// Corde concernée.
        string: usize,
        /// Abscisse de l'empreinte.
        x: f64,
        /// Ordonnée de l'empreinte.
        y: f64,
    },
    /// Retirer le doigt de cette corde.
    Hide {
        /// Corde concernée.
        string: usize,
    },
    /// Clic hors d'une corde : rien à faire.
    Nothing,
}

/// État complet de l'Atlas pour un instrument et un accordage donnés.
#[derive(Clone, Debug)]
pub struct HarmonicEngine {
    instrument: Instrument,
    tuning_index: usize,
    table: FrequencyTable,
    /// Frette posée par corde (0 = corde à vide).
    frets: Vec<u8>,
    notes: Vec<Note>,
    octaves: Vec<u32>,
    harmonics: Vec<Vec<Harmonic>>,
    infos: Vec<Vec<PartialInfo>>,
}

impl HarmonicEngine {
    /// Construit le moteur pour un instrument ; l'accordage 0 est sélectionné.
    pub fn new(instrument: Instrument) -> Self {
        let mut engine = Self {
            instrument,
            tuning_index: 0,
            table: FrequencyTable::new(),
            frets: Vec::new(),
            notes: Vec::new(),
            octaves: Vec::new(),
            harmonics: Vec::new(),
            infos: Vec::new(),
        };
        engine.reset_strings();
        engine.apply_tuning();
        engine.recompute_all();
        engine
    }

    /// Instrument actuellement chargé.
    pub fn instrument(&self) -> &Instrument {
        &self.instrument
    }

    /// Table des fréquences utilisée.
    pub fn table(&self) -> &FrequencyTable {
        &self.table
    }

    /// Index de l'accordage sélectionné.
    pub fn tuning_index(&self) -> usize {
        self.tuning_index
    }

    /// Accordage sélectionné.
    pub fn tuning(&self) -> &Tuning {
        &self.instrument.tunings()[self.tuning_index]
    }

    /// Frette actuellement posée sur une corde (0 = corde à vide).
    pub fn fret(&self, string: usize) -> u8 {
        self.frets[string.min(self.frets.len() - 1)]
    }

    /// Note de la corde à vide.
    pub fn note(&self, string: usize) -> Note {
        self.notes[string.min(self.notes.len() - 1)]
    }

    /// Octave interne de la corde à vide.
    pub fn octave(&self, string: usize) -> u32 {
        self.octaves[string.min(self.octaves.len() - 1)]
    }

    /// Fréquence de la corde à vide (frette non incluse).
    pub fn open_frequency(&self, string: usize) -> f64 {
        self.table
            .get(self.octave(string) as usize, self.note(string).index())
    }

    /// Fréquence réelle de la corde, frette posée incluse.
    pub fn sounding_frequency(&self, string: usize) -> f64 {
        let (note, octave_step) = self.note(string).add_semitones(self.fret(string) as u32);
        self.table
            .get((self.octave(string) + octave_step) as usize, note.index())
    }

    fn reset_strings(&mut self) {
        let count = self.instrument.string_count();
        self.frets = vec![0; count];
        self.notes = vec![Note::C; count];
        self.octaves = vec![0; count];
        self.harmonics = vec![Vec::new(); count];
        self.infos = vec![vec![PartialInfo::default(); PARTIAL_COUNT]; count];
    }

    fn apply_tuning(&mut self) {
        let settings = &self.instrument.tunings()[self.tuning_index].settings;
        for (index, setting) in settings.iter().enumerate() {
            if index >= self.notes.len() {
                break;
            }
            self.notes[index] = setting.note;
            self.octaves[index] = setting.octave;
        }
    }

    /// Change d'instrument : remet l'accordage à 0 et retire tous les doigts.
    pub fn set_instrument(&mut self, instrument: Instrument) {
        self.instrument = instrument;
        self.tuning_index = 0;
        self.reset_strings();
        self.apply_tuning();
        self.recompute_all();
    }

    /// Change d'accordage. `false` si l'index est invalide ou déjà sélectionné.
    pub fn set_tuning(&mut self, index: usize) -> bool {
        if index >= self.instrument.tunings().len() || index == self.tuning_index {
            return false;
        }
        self.tuning_index = index;
        self.apply_tuning();
        self.recompute_all();
        true
    }

    /// Retire tous les doigts (les harmoniques restent affichées).
    pub fn clear_frets(&mut self) {
        for fret in self.frets.iter_mut() {
            *fret = 0;
        }
        self.recompute_all();
    }

    fn recompute_all(&mut self) {
        for string in 0..self.instrument.string_count() {
            self.recompute_string(string);
        }
    }

    /// Recalcule toutes les pastilles d'une corde (`AddHarmonicsOnString`).
    fn recompute_string(&mut self, string: usize) {
        self.harmonics[string].clear();
        let [begin_x, end_x] = self.instrument.span_x();
        let [begin_y, end_y] = self.instrument.string_span_y(string);

        for &partial in &Partial::ALL {
            let value = partial.value();
            let fret = self.frets[string];

            // Corde à vide : la pastille part du sillet. Doigt posé : elle part
            // de la frette, décalée de la largeur de l'empreinte.
            let left_margin = if fret == 0 {
                begin_x
            } else {
                self.instrument.fret_x(fret as usize) + THUMBPRINT_WIDTH - 10.0
            };
            let interval = (end_x - left_margin) / value as f64;
            let mut x = left_margin + interval;

            // Un partiel 1/N apparaît N-1 fois sur la corde ; les occurrences
            // réductibles à un partiel déjà dessiné sont ignorées.
            for numerator in (1..value).rev() {
                if !partial::is_masked(value, numerator) {
                    let y = (x / end_x) * (end_y - begin_y) + begin_y;
                    let info = self.compute_partial_info(string, partial);
                    self.infos[string][partial.index()] = info;
                    self.harmonics[string].push(Harmonic {
                        string,
                        note: info.note,
                        partial,
                        x,
                        y,
                    });
                }
                x += interval;
            }
        }
    }

    /// Fréquence du partiel, note tempérée la plus proche, cents et écart en Hz.
    ///
    /// Reproduit `ModelHarmonicManager::ComputeOneHarmonic`, y compris la
    /// troncature de `static_cast<int>` sur les cents.
    ///
    /// Comportement hérité de l'original, conservé pour la parité : l'octave
    /// affichée est celle de l'entrée de table immédiatement au-dessus de la
    /// fréquence (`maxOctave = octave + 1`). Une harmonique qui tombe pile sur
    /// une note tempérée est donc annoncée une octave trop haut — l'harmonique
    /// 1/2 du Mi aigu est libellée « E5 » alors qu'elle vaut 659,26 Hz
    /// (E5 scientifique, soit l'octave 4 dans la numérotation interne). Les
    /// cents et la fréquence, eux, sont exacts. Corriger ce décalage casserait
    /// la parité avec les captures d'écran et les traductions existantes : à
    /// faire, si on le veut, dans une version majeure.
    fn compute_partial_info(&self, string: usize, partial: Partial) -> PartialInfo {
        let fret = self.fret(string) as u32;
        let (shifted_note, octave_step) = self.note(string).add_semitones(fret);
        let shifted_octave = self.octave(string) + octave_step;
        let base = self
            .table
            .get(shifted_octave as usize, shifted_note.index());
        let frequency = base * partial.value() as f64;

        let mut min_octave = 0usize;
        let mut max_octave = 0usize;
        let mut min_note = 0usize;
        let mut max_note = 0usize;
        let mut min_frequency = 0.0f64;
        let mut max_frequency = 0.0f64;

        'octaves: for octave_index in 0..OCTAVE_COUNT {
            if max_frequency != 0.0 {
                break;
            }
            min_octave = octave_index;
            min_note = Note::B.index();
            for note_index in 0..NOTE_COUNT {
                let reference = self.table.get(octave_index, note_index);
                if frequency < reference {
                    max_frequency = reference;
                    max_note = note_index;
                    max_octave = octave_index + 1;
                    break 'octaves;
                }
                min_frequency = reference;
                min_note = note_index;
            }
        }

        let offset_to_min = frequency - min_frequency;
        let offset_to_max = max_frequency - frequency;
        let max_is_closer = offset_to_max < offset_to_min;
        let signed_offset = if max_is_closer {
            -offset_to_max
        } else {
            offset_to_min
        };
        let cent_value = (max_frequency - min_frequency) / 100.0;

        PartialInfo {
            partial,
            frequency: (frequency * 100.0).round() / 100.0,
            note: Note::from_index(if max_is_closer { max_note } else { min_note }),
            octave: if max_is_closer {
                max_octave
            } else {
                min_octave
            } as u32,
            cents: (signed_offset / cent_value) as i32,
            offset_hz: (signed_offset * 100.0).round() / 100.0,
        }
    }

    /// Pose ou retire le doigt d'une corde (`userClickedOnScene`, partie action).
    pub fn toggle_fret(&mut self, string: usize, fret: u8) -> FretAction {
        if string >= self.instrument.string_count() || fret > self.instrument.fret_count() {
            return FretAction::Nothing;
        }
        if self.frets[string] == fret {
            self.frets[string] = 0;
            self.recompute_string(string);
            FretAction::Hide { string }
        } else {
            self.frets[string] = fret;
            self.recompute_string(string);
            let position = self.finger_position(string, fret);
            FretAction::Show {
                string,
                x: position.x,
                y: position.y,
            }
        }
    }

    /// Traduit un clic dans le repère scène en `(corde, frette)`.
    ///
    /// Reproduit `ModelADHCore::userClickedOnScene` : la frette retenue est la
    /// première dont l'abscisse + 100 px dépasse le clic ; la corde est trouvée
    /// à ± la moitié de l'écart entre les deux premières cordes.
    pub fn hit_test(&self, x: f64, y: f64) -> Option<(usize, u8)> {
        let [begin_x, end_x] = self.instrument.span_x();
        if x < begin_x || x > end_x {
            return None;
        }

        let fret_count = self.instrument.fret_count() as usize;
        let mut fret = fret_count;
        for candidate in 0..fret_count {
            if x <= self.instrument.fret_x(candidate) + FRET_HIT_MARGIN {
                fret = candidate;
                break;
            }
        }

        let strings = self.instrument.strings();
        let second_string = strings.get(1).unwrap_or(&strings[0]);
        let half_gap = ((second_string[0] - strings[0][0]) / 2.0).abs().max(1.0);
        let fret_x = self.instrument.fret_x(fret);

        for (index, span) in strings.iter().enumerate() {
            let position = (fret_x / end_x) * (span[1] - span[0]) + span[0];
            if y >= position - half_gap && y <= position + half_gap {
                return Some((index, fret as u8));
            }
        }
        None
    }

    /// Position de l'empreinte de doigt (`GetFingerPosition`).
    pub fn finger_position(&self, string: usize, fret: u8) -> Point {
        let [begin_x, end_x] = self.instrument.span_x();
        let span = self.instrument.string_span_y(string);
        let x = self.instrument.fret_x(fret as usize);
        let y = (x / (end_x - begin_x)) * (span[1] - span[0]) + span[0] - THUMBPRINT_HEIGHT / 2.0;
        Point::new(x, y)
    }

    /// Pastilles d'une corde, dans l'ordre de calcul (partiels graves → aigus).
    pub fn harmonics(&self, string: usize) -> &[Harmonic] {
        &self.harmonics[string.min(self.harmonics.len() - 1)]
    }

    /// Toutes les pastilles de toutes les cordes.
    pub fn all_harmonics(&self) -> impl Iterator<Item = &Harmonic> {
        self.harmonics.iter().flatten()
    }

    /// Pastilles filtrées par les masques de notes et de partiels.
    ///
    /// Les masques sont copiés (12 et 11 booléens) pour éviter toute contrainte
    /// de durée de vie sur l'itérateur renvoyé.
    pub fn visible_harmonics(
        &self,
        notes: crate::masking::NoteMask,
        partials: crate::masking::PartialMask,
    ) -> impl Iterator<Item = &Harmonic> + '_ {
        self.all_harmonics()
            .filter(move |h| notes.shows(h.note) && partials.shows(h.partial))
    }

    /// Détail d'un partiel sur une corde (contenu de l'infobulle).
    pub fn partial_info(&self, string: usize, partial: Partial) -> PartialInfo {
        self.infos[string.min(self.infos.len() - 1)][partial.index()]
    }
}
