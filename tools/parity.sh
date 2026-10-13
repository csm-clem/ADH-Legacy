#!/usr/bin/env bash
# Comparaison ligne à ligne entre le moteur Rust (crates/adh-core) et les tables
# de référence produites par le moteur Qt historique (ModelHarmonicManager).
#
# Les dumps de référence sont gelés dans crates/adh-core/tests/golden/*.csv.gz :
# ils ont été produits par tools/qt-dump.cpp compilé contre
# src/ModelHarmonicManager.cpp avant la suppression de l'arborescence Qt.
# Pour retrouver le moteur d'origine : git checkout 1.0 -- src/
#
# Prérequis : Rust uniquement.
#   tools/parity.sh
set -euo pipefail
cd "$(dirname "$0")/.."

GOLDEN="crates/adh-core/tests/golden"
mkdir -p target

echo '[1/2] construction du moteur Rust'
cargo build -q

echo '[2/2] comparaison avec les dumps de référence'
./target/debug/adh-dump --csv > target/rust.csv
gzip -dc "$GOLDEN/qt-corde-a-vide.csv.gz" | diff -u - target/rust.csv
echo "parité OK (corde à vide) : $(wc -l < target/rust.csv) lignes identiques"

./target/debug/adh-dump --csv --frets 0,12 > target/rust-fretted.csv
gzip -dc "$GOLDEN/qt-doigt-case-12.csv.gz" | diff -u - target/rust-fretted.csv
echo "parité OK (doigt posé case 12) : $(wc -l < target/rust-fretted.csv) lignes identiques"

echo "empreinte FNV-1a : $(./target/debug/adh-dump --checksum)"
