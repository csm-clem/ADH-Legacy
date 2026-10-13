//! Tests du parseur d'accordages.
//!
//! Ils vérifient deux choses :
//! 1. le parseur accepte ce que l'utilisateur écrit réellement (`F♯3`, `Eb2`, …) ;
//! 2. il produit les mêmes notes/octaves que `_GetTuningFromString` +
//!    `GetTuningFromIndex` du moteur Qt (octave décrémentée, liste inversée).

use adh_core::note::Note;
use adh_core::tuning::{parse_token, parse_tuning, to_string_settings, StringSetting, TuningError};

#[test]
fn parse_token_accepte_les_notations_reelles() {
    assert_eq!(parse_token("C4").unwrap(), (Note::C, 4));
    assert_eq!(parse_token("c4").unwrap(), (Note::C, 4));
    assert_eq!(parse_token("F♯3").unwrap(), (Note::Fs, 3));
    assert_eq!(parse_token("F#3").unwrap(), (Note::Fs, 3));
    assert_eq!(parse_token("E♭2").unwrap(), (Note::Ds, 2));
    assert_eq!(parse_token("Eb2").unwrap(), (Note::Ds, 2));
    assert_eq!(parse_token("B♭1").unwrap(), (Note::As, 1));
    assert_eq!(parse_token("A♯2").unwrap(), (Note::As, 2));
    assert_eq!(parse_token("F♯0").unwrap(), (Note::Fs, 0));
    assert_eq!(parse_token("B0").unwrap(), (Note::B, 0));
}

#[test]
fn parse_token_refuse_les_jetons_invalides() {
    assert!(parse_token("").is_err());
    assert!(parse_token("H4").is_err());
    assert!(parse_token("C").is_err());
    assert!(parse_token("C4x").is_err());
    assert!(parse_token("4").is_err());
    assert!(parse_token("C-1").is_err());
    assert_eq!(
        parse_token("C10"),
        Err(TuningError::OctaveOutOfRange {
            token: "C10".into()
        })
    );
}

#[test]
fn octave_a_deux_chiffres_nest_pas_tronquee() {
    // L'original lisait l'octave à un index fixe : « E10 » était lu « E1 ».
    // Ici l'octave est lue en entier puis refusée proprement (la table de
    // fréquences de l'Atlas couvre les octaves 0..=8).
    assert_eq!(
        parse_token("E10"),
        Err(TuningError::OctaveOutOfRange {
            token: "E10".into()
        })
    );
    assert_eq!(parse_token("E9").unwrap(), (Note::E, 9));
}

#[test]
fn convertion_en_cordes_inverse_et_decremente() {
    // "E2 A2 D3 G3 B3 E4" (grave → aigu) devient corde 0 = Mi aigu.
    let settings = to_string_settings("E2 A2 D3 G3 B3 E4").unwrap();
    assert_eq!(
        settings,
        vec![
            StringSetting {
                note: Note::E,
                octave: 3
            },
            StringSetting {
                note: Note::B,
                octave: 2
            },
            StringSetting {
                note: Note::G,
                octave: 2
            },
            StringSetting {
                note: Note::D,
                octave: 2
            },
            StringSetting {
                note: Note::A,
                octave: 1
            },
            StringSetting {
                note: Note::E,
                octave: 1
            },
        ]
    );
}

#[test]
fn octave_zero_nest_pas_decremente_en_negatif() {
    // Comme `if (octave > 0) octave--;` dans le C++ : B0 reste à l'octave 0.
    let settings = to_string_settings("B0 E1 A1 D2 G2").unwrap();
    assert_eq!(settings.last().unwrap().octave, 0);
    assert_eq!(settings.last().unwrap().note, Note::B);
}

#[test]
fn accordage_vide_rejete() {
    assert_eq!(parse_tuning("   "), Err(TuningError::Empty));
}

#[test]
fn tous_les_accordages_embarques_sont_parsables() {
    let instruments = adh_core::builtin::instruments().expect("instruments embarqués invalides");
    assert_eq!(instruments.len(), 7);
    for instrument in &instruments {
        assert!(
            !instrument.tunings().is_empty(),
            "{} sans accordage",
            instrument.id()
        );
        for tuning in instrument.tunings() {
            assert_eq!(
                tuning.settings.len(),
                instrument.string_count(),
                "{} / {}",
                instrument.id(),
                tuning.name
            );
        }
    }
}

#[test]
fn accordage_personnalise_ajoute_et_valide() {
    let guitar6 = adh_core::builtin::instruments()
        .expect("instruments embarqués invalides")
        .into_iter()
        .find(|instrument| instrument.id() == "guitar6")
        .expect("guitare 6 cordes absente");
    let declared = guitar6.tunings().len();

    // Un accordage du bon nombre de cordes est accepté, et inversé comme les
    // accordages du TOML (corde 0 = la plus aiguë).
    let mut custom = guitar6.clone();
    custom
        .add_tuning("DADGAD", "D2 A2 D3 G3 A3 D4")
        .expect("accordage personnalisé refusé");
    assert_eq!(custom.tunings().len(), declared + 1);
    let added = custom.tunings().last().expect("accordage ajouté absent");
    assert_eq!(added.name, "DADGAD");
    assert_eq!(added.raw, "D2 A2 D3 G3 A3 D4");
    assert_eq!(added.settings.first().expect("corde 0").note, Note::D);

    // Le mauvais nombre de cordes est refusé, sans rien ajouter.
    let mut wrong = guitar6.clone();
    assert!(wrong.add_tuning("quatre cordes", "D2 A2 D3 G3").is_err());
    assert_eq!(wrong.tunings().len(), declared);

    // Un jeton invalide est refusé aussi.
    let mut bad = guitar6.clone();
    assert!(bad.add_tuning("faux", "D2 A2 D3 G3 B3 Z4").is_err());
    assert_eq!(bad.tunings().len(), declared);

    // Nom vide : le libellé reprend la chaîne, qui est décapée.
    let mut unnamed = guitar6.clone();
    unnamed
        .add_tuning("  ", " D2 A2 D3 G3 B3 D4 ")
        .expect("accordage personnalisé refusé");
    let added = unnamed.tunings().last().expect("accordage ajouté absent");
    assert_eq!(added.name, "D2 A2 D3 G3 B3 D4");
    assert_eq!(added.raw, "D2 A2 D3 G3 B3 D4");
}

