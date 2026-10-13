//! Tests de la scène : positions des pastilles, hit-test d'un clic, empreinte
//! de doigt, masques de visibilité.

use adh_core::builtin::instruments;
use adh_core::harmonics::{FretAction, HarmonicEngine};
use adh_core::masking::{NoteMask, PartialMask};
use adh_core::note::Note;
use adh_core::partial::Partial;
use adh_core::Instrument;

fn guitar6() -> Instrument {
    instruments()
        .unwrap()
        .into_iter()
        .find(|instrument| instrument.id() == "guitar6")
        .expect("guitar6 absente")
}

fn high_e() -> HarmonicEngine {
    HarmonicEngine::new(guitar6())
}

#[test]
fn positions_des_pastilles_depuis_le_sillet() {
    let engine = high_e();
    let half = engine
        .harmonics(0)
        .iter()
        .find(|harmonic| harmonic.partial == Partial::Half)
        .unwrap();
    // 234 + (6432 - 234) / 2 = 3333
    assert!((half.x - 3333.0).abs() < 0.01);
    assert!((half.y - 576.62).abs() < 0.01);

    // À l'intérieur d'un partiel, les abscisses croissent strictement et
    // restent sur la corde.
    for partial in Partial::ALL {
        let harmonics = engine
            .harmonics(0)
            .iter()
            .filter(|harmonic| harmonic.partial == partial)
            .collect::<Vec<_>>();
        assert!(
            harmonics.windows(2).all(|pair| pair[1].x > pair[0].x),
            "{}",
            partial.ratio_label()
        );
        assert!(harmonics
            .iter()
            .all(|harmonic| harmonic.x > 234.0 && harmonic.x < 6432.0));
        // Corde 0 : y entre 540 et 616, décroissant vers le chevalet.
        assert!(harmonics
            .iter()
            .all(|harmonic| (539.0..617.0).contains(&harmonic.y)));
    }
}

#[test]
fn poser_un_doigt_double_la_frequence_de_l_octave() {
    let mut engine = high_e();
    let action = engine.toggle_fret(0, 12);
    assert!(matches!(action, FretAction::Show { string: 0, .. }));
    assert_eq!(engine.fret(0), 12);
    // 12 cases = une octave : la corde sonne une octave plus haut.
    assert!((engine.sounding_frequency(0) - 659.255).abs() < 1e-2);
    assert!((engine.open_frequency(0) - 329.6276).abs() < 1e-3);

    // Les harmoniques sont recalculées à partir de la frette posée.
    let half = engine
        .harmonics(0)
        .iter()
        .find(|harmonic| harmonic.partial == Partial::Half)
        .unwrap();
    assert!((engine.partial_info(0, Partial::Half).frequency - 1318.51).abs() < 0.01);
    assert!(half.x > 3333.0, "le doigt posé repousse les harmoniques");

    let action = engine.toggle_fret(0, 12);
    assert_eq!(action, FretAction::Hide { string: 0 });
    assert_eq!(engine.fret(0), 0);
    assert!((engine.open_frequency(0) - 329.6276).abs() < 1e-3);
}

#[test]
fn retirer_les_doigts_ne_supprime_pas_les_harmoniques() {
    let mut engine = high_e();
    engine.toggle_fret(0, 5);
    engine.toggle_fret(3, 7);
    engine.clear_frets();
    assert_eq!(engine.fret(0), 0);
    assert_eq!(engine.fret(3), 0);
    assert_eq!(engine.harmonics(0).len(), 45);
}

#[test]
fn clic_traduit_en_corde_et_frette() {
    let engine = high_e();
    // Clic sur la pastille 1/2 de la corde 0.
    let (string, fret) = engine.hit_test(3333.0, 576.62).expect("clic hors corde");
    assert_eq!(string, 0);
    // 3333 px tombe sur la 12e case de la table de référence (3254 + 100 ≥ 3333).
    assert_eq!(fret, 12);

    // Hors de la zone des cordes : rien.
    assert!(engine.hit_test(100.0, 600.0).is_none());
    assert!(engine.hit_test(7000.0, 600.0).is_none());

    // Sur la corde grave (corde 5).
    let (string, _) = engine.hit_test(3333.0, 1016.0).expect("clic hors corde");
    assert_eq!(string, 5);

    // Entre deux cordes : rien.
    assert!(engine.hit_test(3333.0, 800.0).is_none());
}

#[test]
fn un_clic_reposible_la_frette_precedente() {
    let mut engine = high_e();
    let (string, fret) = engine.hit_test(3333.0, 576.62).unwrap();
    let action = engine.toggle_fret(string, fret);
    assert!(matches!(action, FretAction::Show { .. }));
    assert_eq!(engine.fret(0), 12);
    let action = engine.toggle_fret(string, fret);
    assert_eq!(action, FretAction::Hide { string: 0 });
}

#[test]
fn empreinte_de_doigt_sur_la_frette() {
    let engine = high_e();
    let point = engine.finger_position(0, 1);
    assert!((point.x - 486.0).abs() < 1e-9);
    // (486 / (6432-234)) * (540-616) + 616 - 96/2 = 562.04…
    assert!((point.y - 562.04).abs() < 0.01);
}

#[test]
fn masques_de_notes_et_de_partiels() {
    let engine = high_e();
    let all = NoteMask::all_visible();
    let all_partials = PartialMask::all_visible();
    // 6 cordes × 45 pastilles.
    assert_eq!(engine.visible_harmonics(all, all_partials).count(), 270);

    let no_notes = NoteMask::none_visible();
    assert_eq!(no_notes.visible_count(), 0);
    assert_eq!(engine.visible_harmonics(no_notes, all_partials).count(), 0);

    let mut one_note = NoteMask::none_visible();
    one_note.show(Note::E);
    let visible = engine.visible_harmonics(one_note, all_partials).count();
    assert!(visible > 0 && visible < 270, "{visible}");

    let mut partials = PartialMask::all_visible();
    assert!(!partials.toggle(Partial::Half));
    // Le 1/2 n'apparaît qu'une fois par corde.
    assert_eq!(engine.visible_harmonics(all, partials).count(), 264);
    assert!(partials.toggle(Partial::Half));
    assert_eq!(engine.visible_harmonics(all, partials).count(), 270);

    // Masquer une note retire bien ses pastilles de la corde testée.
    let mut without_e = NoteMask::all_visible();
    without_e.hide(Note::E);
    let hidden = engine
        .harmonics(0)
        .iter()
        .filter(|harmonic| !without_e.shows(harmonic.note))
        .count();
    assert!(hidden > 0);
    assert_eq!(
        engine.harmonics(0).len() - hidden,
        engine
            .visible_harmonics(without_e, all_partials)
            .filter(|h| h.string == 0)
            .count()
    );
}
