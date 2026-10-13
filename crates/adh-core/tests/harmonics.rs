//! Tests de calcul du moteur d'harmoniques.
//!
//! Les valeurs attendues sont celles du moteur Qt historique, vérifiées par
//! `tools/parity.sh` (20 610 lignes CSV identiques). Les invariants musicaux
//! (octaves exactes, écarts des justes intonations) sont testés en plus.

use adh_core::builtin::instruments;
use adh_core::harmonics::HarmonicEngine;
use adh_core::note::Note;
use adh_core::partial::{Partial, PARTIAL_COUNT};
use adh_core::Instrument;

fn guitar6() -> Instrument {
    instruments()
        .unwrap()
        .into_iter()
        .find(|instrument| instrument.id() == "guitar6")
        .expect("guitar6 absente")
}

/// Corde de Mi aigu (corde 0), accordage Standard, corde à vide.
fn high_e() -> HarmonicEngine {
    HarmonicEngine::new(guitar6())
}

#[test]
fn chaque_corde_produit_45_pastilles() {
    for instrument in instruments().unwrap() {
        let engine = HarmonicEngine::new(instrument.clone());
        for string in 0..instrument.string_count() {
            assert_eq!(
                engine.harmonics(string).len(),
                45,
                "{} corde {}",
                instrument.id(),
                string
            );
        }
    }
}

#[test]
fn nombre_de_pastilles_par_partiel() {
    let expected = [1, 2, 2, 4, 2, 6, 4, 6, 4, 10, 4];
    let engine = high_e();
    for partial in Partial::ALL {
        let count = engine
            .harmonics(0)
            .iter()
            .filter(|harmonic| harmonic.partial == partial)
            .count();
        assert_eq!(
            count,
            expected[partial.index()],
            "{}",
            partial.ratio_label()
        );
    }
    assert_eq!(expected.iter().sum::<usize>(), 45);
    assert_eq!(PARTIAL_COUNT, 11);
}

#[test]
fn corde_a_vide_mi_aigu() {
    let engine = high_e();
    assert_eq!(engine.note(0), Note::E);
    assert_eq!(engine.octave(0), 3);
    assert!((engine.open_frequency(0) - 329.6276).abs() < 1e-3);
}

#[test]
fn harmoniques_d_octave_sont_exactes() {
    let engine = high_e();
    let open = engine.open_frequency(0);
    for (partial, octaves_up) in [
        (Partial::Half, 1),
        (Partial::Quarter, 2),
        (Partial::Eighth, 3),
    ] {
        let info = engine.partial_info(0, partial);
        let expected = open * 2f64.powi(octaves_up);
        assert!(
            (info.frequency - expected).abs() < 0.01,
            "{} : attendu {:.2}, obtenu {:.2}",
            partial.ratio_label(),
            expected,
            info.frequency
        );
        assert_eq!(info.note, Note::E);
        assert!(info.cents.abs() <= 1, "{}", partial.ratio_label());
    }
}

#[test]
fn quinte_et_tierce_sont_legerement_fausses_comme_attendu() {
    let engine = high_e();

    // 1/3 = octave + quinte juste (3/2) : +2 cents par rapport au tempérament.
    let fifth = engine.partial_info(0, Partial::Third);
    assert_eq!(fifth.note, Note::B);
    assert_eq!(fifth.cents, 1);
    assert!((fifth.frequency - 988.88).abs() < 0.01);

    // 1/5 = 2 octaves + tierce juste (5/4) : −14 cents.
    let third = engine.partial_info(0, Partial::Fifth);
    assert_eq!(third.note, Note::Gs);
    assert_eq!(third.cents, -14);
    assert!((third.frequency - 1648.14).abs() < 0.01);

    // 1/7 : la septième harmonique est très en dessous du tempérament.
    let seventh = engine.partial_info(0, Partial::Seventh);
    assert_eq!(seventh.note, Note::D);
    assert_eq!(seventh.cents, -31);

    // 1/11 : la plus fausse de toutes.
    let eleventh = engine.partial_info(0, Partial::Eleventh);
    assert_eq!(eleventh.note, Note::As);
    assert_eq!(eleventh.cents, -49);
}

#[test]
fn etiquettes_de_infobulle() {
    let engine = high_e();
    let info = engine.partial_info(0, Partial::Third);
    assert_eq!(info.frequency_label(), "988.88Hz");
    assert_eq!(info.partial_label(), "Partiel 1/3, 2e quinte (P5)");
    assert_eq!(info.offset_label(), "1 cents de B5 (1.12Hz)");
}

#[test]
fn changer_accordage_change_les_frequences() {
    let mut engine = high_e();
    assert_eq!(engine.tuning_index(), 0);
    assert_eq!(engine.tuning().name, "Standard");

    // Accordage « -1 ton » (index 2) : la corde 0 passe de Mi à Ré.
    assert!(engine.set_tuning(2));
    assert_eq!(engine.note(0), Note::D);
    assert_eq!(engine.octave(0), 3);
    assert!((engine.open_frequency(0) - 293.6648).abs() < 1e-3);

    // Un index déjà sélectionné ou hors plage ne change rien.
    assert!(!engine.set_tuning(2));
    assert!(!engine.set_tuning(999));

    // Changer d'instrument réinitialise accordage et doigts.
    let bass4 = instruments()
        .unwrap()
        .into_iter()
        .find(|instrument| instrument.id() == "bass4")
        .unwrap();
    engine.set_instrument(bass4);
    assert_eq!(engine.instrument().string_count(), 4);
    assert_eq!(engine.tuning_index(), 0);
    assert_eq!(engine.fret(0), 0);
}
