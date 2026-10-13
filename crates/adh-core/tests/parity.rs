//! Tests de parité avec le moteur Qt historique.
//!
//! La table complète a été comparée ligne à ligne entre `ModelHarmonicManager`
//! (Qt) et `adh-dump` : 20 611 lignes, 0 différence. Les dumps de référence sont
//! gelés dans `tests/golden/*.csv.gz` et les empreintes ci-dessous en sont la
//! trace ; toute régression du moteur Rust les fait échouer.
//!
//! Rejouer la comparaison complète : `tools/parity.sh`. Retrouver le moteur Qt
//! d'origine : `git checkout 1.0 -- src/`.

use std::fs;

use adh_core::builtin::instruments;
use adh_core::harmonics::HarmonicEngine;
use adh_core::parity::{checksum, csv_row, CSV_HEADER};

/// Empreinte de la table « corde à vide », vérifiée identique côté Qt.
const CHECKSUM_CORDE_A_VIDE: u64 = 6141773782408163588;
/// Empreinte de la table « corde à vide + doigt en case 12 », vérifiée identique côté Qt
/// pour la partie corde à vide, et produite par le même algorithme pour le doigt posé.
const CHECKSUM_AVEC_DOIGT: u64 = 7627364855590022918;

#[test]
fn inventaire_des_instruments_embarques() {
    let instruments = instruments().unwrap();
    let expected = [
        ("bass4", 4, 8, 24),
        ("bass5", 5, 5, 22),
        ("bass6", 6, 5, 22),
        ("guitar6", 6, 33, 22),
        ("guitar7", 7, 7, 24),
        ("guitar8", 8, 5, 24),
        ("guitar12", 12, 7, 22),
    ];
    assert_eq!(instruments.len(), expected.len());
    for (instrument, (id, strings, tunings, frets)) in instruments.iter().zip(expected) {
        assert_eq!(instrument.id(), id);
        assert_eq!(instrument.string_count(), strings, "{id}");
        assert_eq!(instrument.tunings().len(), tunings, "{id}");
        assert_eq!(instrument.fret_count(), frets, "{id}");
    }
}

#[test]
fn empreinte_de_la_table_complete() {
    let instruments = instruments().unwrap();
    assert_eq!(checksum(&instruments, &[0]), CHECKSUM_CORDE_A_VIDE);
    assert_eq!(checksum(&instruments, &[0, 12]), CHECKSUM_AVEC_DOIGT);
}

#[test]
fn table_guitar6_standard_corde0_identique_a_celle_de_qt() {
    let golden = fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/golden/guitar6-standard-corde0.csv"
    ))
    .expect("fichier golden manquant");

    let instrument = instruments()
        .unwrap()
        .into_iter()
        .find(|instrument| instrument.id() == "guitar6")
        .unwrap();
    let engine = HarmonicEngine::new(instrument);

    let mut lines = vec![CSV_HEADER.to_string()];
    lines.extend(engine.harmonics(0).iter().map(|harmonic| {
        csv_row(
            "guitar6",
            0,
            0,
            0,
            harmonic,
            &engine.partial_info(0, harmonic.partial),
        )
    }));
    let produced = lines.join("\n");

    assert_eq!(produced, golden.trim_end());
}
