//! `adh-dump` — exporte les harmoniques calculées par le moteur Rust.
//!
//! Outil de parité : il produit exactement les mêmes tables que le moteur Qt
//! historique ; les dumps de référence sont gelés dans
//! `crates/adh-core/tests/golden/` et comparés par `tools/parity.sh`.
//!
//! ```text
//! adh-dump                       résumé instrument par instrument
//! adh-dump --csv                 toutes les harmoniques, corde à vide
//! adh-dump --csv --frets 0,12    idem, mais aussi doigt posé en case 12
//! adh-dump --checksum            empreinte FNV-1a de la table complète
//! ```

use std::process::ExitCode;

use adh_core::builtin::instruments;
use adh_core::harmonics::HarmonicEngine;

const USAGE: &str = "usage: adh-dump [--csv [--frets 0,12]] [--checksum]";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut csv = false;
    let mut checksum = false;
    let mut frets: Vec<u8> = vec![0];

    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--csv" => csv = true,
            "--checksum" => checksum = true,
            "--frets" => {
                index += 1;
                let Some(value) = args.get(index) else {
                    eprintln!("--frets attend une liste séparée par des virgules\n{USAGE}");
                    return ExitCode::from(2);
                };
                frets = match parse_frets(value) {
                    Some(frets) => frets,
                    None => {
                        eprintln!("--frets : valeur invalide {value:?}\n{USAGE}");
                        return ExitCode::from(2);
                    }
                };
            }
            "-h" | "--help" => {
                println!("{USAGE}");
                return ExitCode::SUCCESS;
            }
            other => {
                eprintln!("argument inconnu : {other:?}\n{USAGE}");
                return ExitCode::from(2);
            }
        }
        index += 1;
    }

    let instruments = match instruments() {
        Ok(instruments) => instruments,
        Err(error) => {
            eprintln!("instruments embarqués invalides : {error}");
            return ExitCode::from(1);
        }
    };

    if checksum {
        println!(
            "fnv1a64,{}",
            adh_core::parity::checksum(&instruments, &frets)
        );
        return ExitCode::SUCCESS;
    }

    if csv {
        dump_csv(&instruments, &frets);
        return ExitCode::SUCCESS;
    }

    println!("instrument,cordes,accordages,frettes,harmoniques_par_corde");
    for instrument in &instruments {
        let engine = HarmonicEngine::new(instrument.clone());
        println!(
            "{},{},{},{},{}",
            instrument.id(),
            instrument.string_count(),
            instrument.tunings().len(),
            instrument.fret_count(),
            engine.harmonics(0).len()
        );
    }
    ExitCode::SUCCESS
}

fn parse_frets(value: &str) -> Option<Vec<u8>> {
    let frets: Option<Vec<u8>> = value
        .split(',')
        .map(|part| part.trim().parse().ok())
        .collect();
    frets.filter(|frets| !frets.is_empty())
}

fn dump_csv(instruments: &[adh_core::Instrument], frets: &[u8]) {
    println!("{}", adh_core::parity::CSV_HEADER);
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
                        println!(
                            "{}",
                            adh_core::parity::csv_row(
                                instrument.id(),
                                tuning_index,
                                string,
                                fret,
                                harmonic,
                                &info
                            )
                        );
                    }
                    if fret > 0 {
                        engine.toggle_fret(string, fret);
                    }
                }
            }
        }
    }
}
