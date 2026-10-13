# Tables de référence

Ces dumps viennent du **moteur Qt historique** (`src/ModelHarmonicManager.cpp`,
compilé par l'ancien `tools/qt-dump.cpp` sous Qt 6) et servent de référence au
moteur Rust. Dernière vérification : commit `5547712` de `develop`, la veille de
la suppression de l'arborescence Qt (dernier tag de l'ère Qt : **`1.0`**).

| Fichier | Lignes (en-tête compris) | SHA-256 du CSV décompressé |
|---|---|---|
| `qt-corde-a-vide.csv.gz` | 20 611 | `012aa2707564a83cb3bee50a577727185673982e999b0347402c0a1db993a9e7` |
| `qt-doigt-case-12.csv.gz` | 41 221 | `47baf9bd44abcff6bbf7bcbf85a8f9823220c4ff977c64d6cdee3c01cb0f8e4e` |
| `guitar6-standard-corde0.csv` | 47 | échantillon lisible : corde 0 de la guitare 6, accord standard |

- Rejouer la comparaison complète : `./tools/parity.sh`.
- Retrouver le moteur Qt d'origine : `git checkout 1.0 -- src/`.
