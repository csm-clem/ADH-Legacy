# Licence des visuels (assets)

Les visuels embarqués dans `crates/adh-assets/assets/` (photos d'instruments,
pastilles d'harmoniques, médiators, empreinte de doigt, logo, icônes) ont été
réalisés par les auteurs de l'Atlas des harmoniques. Ils sont placés sous la
**même licence que le code** :

> MIT OR Apache-2.0
> Copyright (c) 2011-2026 Tristan Morvan

Les textes complets sont dans `LICENSE-MIT` et `LICENSE-APACHE` à la racine du
dépôt. `REUSE.toml` déclare cette licence sur `crates/adh-assets/assets/**`.

## Exception : la police de symboles

`crates/adh-assets/assets/fonts/adh-symbols.ttf` n'est pas une œuvre des auteurs
de l'Atlas : c'est un sous-ensemble (glyphes `♪ ♭ ♮ ♯`) de **Noto Sans Symbols**,
sous **SIL Open Font License 1.1** — texte dans `LICENSE-OFL` — renommé
« ADH Symbols » comme l'OFL l'exige pour une version modifiée. La police originale
n'a aucun Reserved Font Name, mais le nom d'origine n'est conservé nulle part
ailleurs. Le fichier est régénéré par `tools/subset_font.py` ; la mention de
copyright et la référence à la licence sont recopiées dans ses métadonnées.

## Historique corrigé

L'application Qt historique déclarait dans `debian/control` une licence
**Creative Commons BY-NC-ND** (pas d'usage commercial, pas d'œuvre dérivée), et
`debian/copyright` contenait un template `dh-make` © 1998-2011 Craig Small
(GPL-3.0+) : une erreur de packaging, sans rapport avec les ayants droit réels.

Ces deux déclarations sont abandonnées. L'arborescence Qt (`src/`, `image/`,
`debian/`) relevait de la même licence, sauf `debian/copyright`, faux, supprimé
avec elle. Elle reste consultable dans l'histoire du dépôt, au tag `1.0`.
