# Migration Qt → Rust/egui

Réécriture de **L'Atlas des harmoniques** en Rust avec `egui`/`eframe`, sans
dépendance Qt, sous licence permissive, pour Linux, macOS et Windows.

## Pourquoi quitter Qt

| Problème Qt | Conséquence | Solution Rust |
|---|---|---|
| Qt6 sous LGPL v3 / GPL v2 | obligations de licence, notices, possibilité de relier | `egui`/`eframe` en `MIT OR Apache-2.0` |
| Plugins d'instruments en `.so`/`.dylib`/`.dll` chargés par `QLibrary` | ABI fragile, 4 `CMakeLists.txt` dédiés, déploiement par plateforme | un instrument = un fichier TOML |
| `signals`/`slots` + 23 `Q_PROPERTY` de masquage | 23 signaux, 23 setters, 23 getters | deux tableaux de booléens relus à chaque image |
| QML (`Flickable`, `PinchArea`, `Image`, `Repeater`) | dépend de `qtdeclarative`, de `rcc` et d'un moteur de rendu | scène dessinée par `egui::Painter` |
| `debian/copyright` = template `dh-make` © 1998-2011 Craig Small | licence du dépôt incorrecte | `LICENSE-MIT`, `LICENSE-APACHE`, `LICENSE-assets.md`, `REUSE.toml` |

## État d'avancement

| Phase | Contenu | Statut |
|---|---|---|
| 0 | Licences, arborescence, workspace Cargo | **fait** |
| 1 | `adh-core` : notes, partiels, fréquences, accordages, géométrie, masques, 7 instruments en TOML | **fait** |
| 2 | Parité Qt ↔ Rust | **fait** (41 221 lignes CSV identiques) |
| 3 | `adh-app` : fenêtre `eframe`, menus, scène, zoom/pan | **fait** |
| 4 | Pastilles, infobulles, empreinte de doigt, médiators | **fait** |
| 5 | Masques notes/partiels, barres d'outils, accordages personnalisés | **fait** |
| 6 | i18n FR/EN, paramètres persistants, raccourcis clavier | **fait** |
| 7 | Suppression de `src/`, `image/`, `CMakeLists.txt`, `ADH.pro`, `debian/` | **fait** — les tables du moteur Qt sont gelées dans `crates/adh-core/tests/golden/` |
| 8 | `cargo-dist` : paquets Linux, `.dmg` macOS, `.msi` Windows | **fait** pour Linux (archives, paquet Arch, rpm) — macOS et Windows demandent une CI sur ces plateformes |

## Interface

`crates/adh-app` (binaire `adh-app`) rend tout avec `egui` en mode immédiat :

- menus **Instruments / Accordages / Affichage / Aide**, identiques à
  `MenusForm.ui.qml`, y compris les six liens du menu Aide ;
- barres « Afficher les partiels » (11 boutons `1/2 … 1/12`) et « Afficher les
  notes » (12 boutons + Toutes / Aucune), comme `ToolBarHidePartiel` et
  `ToolBarHideNote` ;
- scène : photo, empreintes, pastilles, médiators, infobulle ;
- barre d'état : instrument, accordage, nombre de cordes, doigts posés, zoom ;
- raccourcis : `+` / `-` zoom, `F` ajuster, `C` retirer les doigts, `Échap`
  fermer la boîte « À propos » ;
- réglages persistés en TOML (`~/.config/adh/settings.toml`, ou l'équivalent
  macOS/Windows) : instrument, accordage, langue, masques, zoom, décalage et
  accordages personnalisés ;
- accordages personnalisés : le menu **Accordages → Accordage personnalisé…**
  ouvre une fenêtre (nom + cordes, du grave vers l'aigu), validée par
  `Instrument::add_tuning` — même parseur et même nombre de cordes que les
  accordages du TOML — puis enregistrée dans `settings.toml` sous
  `[[custom_tunings]]` et supprimable depuis la même fenêtre ;
- langue FR/EN basculable depuis le menu Aide. Les noms d'intervalle des
  partiels restent en français : c'est le vocabulaire de l'Atlas.

Le repère écran est détaillé dans `docs/SPEC.md`.

## Commandes

```sh
cargo build                 # adh-core + adh-assets + adh-app + adh-dump
cargo test                  # 29 tests
cargo run -p adh-app        # l'application
cargo run --bin adh-dump    # résumé instrument par instrument
cargo run --bin adh-dump -- --csv > table.csv
cargo run --bin adh-dump -- --checksum
./tools/parity.sh           # diff complet contre les tables du moteur Qt, gelées
```

Dans VS Code, **F5** lance l'application : `launch.json` construit
`target/debug/adh-app` (tâche par défaut `cargo build -p adh-app`) puis l'exécute
sous CodeLLDB. Sans débogueur, la tâche `cargo: run adh-app` fait la même chose.

Pour vérifier le rendu sans ouvrir la fenêtre :

```sh
ADH_SCREENSHOT=/tmp/adh.png cargo run -p adh-app
```

`eframe` a bien son propre drapeau (`EFRAME_SCREENSHOT_TO` + feature
`__screenshot`), mais son backend glow lit le tampon après son échange et rend une
image noire ; `Atlas::capture_screenshot` passe par la commande de viewport
d'`egui`, qui capture juste après le dessin.

`ADH_DEBUG=1` affiche sur la sortie d'erreur le point scène et le résultat du
hit-test de chaque clic.

`tools/parity.sh` n'a aucun prérequis hors Rust : il compare le dump du moteur aux
tables du moteur Qt gelées dans `crates/adh-core/tests/golden/`. Pour rejouer la
comparaison d'origine en C++ (Qt 6 + `moc` + `g++`), il faut d'abord sortir le
code de l'histoire : `git checkout 1.0 -- src/`.

## Emballage

`dist-workspace.toml` configure `cargo-dist` (archives) et `tools/release.sh`
assemble une release complète :

```sh
cargo install cargo-dist
./tools/release.sh              # tests + clippy + parité, puis dist/
./tools/release.sh --quick      # uniquement la construction
./tools/release.sh --publish    # publication Forgejo (FORGEJO_URL/TOKEN/REPO)
```

`dist/` reçoit les deux archives `cargo-dist` (`adh-app` et `adh-dump`), une
archive portable `usr/` (binaire, `.desktop`, AppStream, icônes hicolor,
licences), le paquet Arch (`packaging/PKGBUILD`) et le paquet rpm
(`packaging/adh.spec`), plus les empreintes SHA-256. L'identifiant
d'application est `com.lecompositeur.adh`.

## Décisions de conception

1. **Un instrument = un TOML** (`crates/adh-core/assets/instruments/`).
   Le TOML décrit : `id`, `name`, `image`, `fret_count`, la géométrie des cordes,
   éventuellement une table de positions de frettes propre à l'instrument, et la
   liste des accordages.
2. **Les visuels sont embarqués** par `adh-assets` (`include_bytes!`) :
   l'exécutable reste autonome, ce qui remplace le `.qrc` et `_GetResourcesPath()`.
3. **Géométrie en `f64`** pour reproduire bit à bit les `qreal` du moteur C++
   (une version `f32` donnait des écarts de 0,01 px sur les abscisses).
4. **Un seul modèle, relu à chaque image** : `HarmonicEngine` détient l'instrument,
   l'accordage, les frettes posées, les pastilles et les détails de partiels.
   L'UI ne fait que le lire.
5. **Parité d'abord, corrections ensuite** : les quirks de l'original sont
   reproduits et documentés (voir plus bas) pour que la comparaison soit possible ;
   chaque correction devient ensuite un choix explicite.

## Bugs de l'original trouvés pendant la migration

Les chemins `src/…` cités ci-dessous désignent l'arborescence Qt, retirée du dépôt
mais consultable au tag `1.0`.

1. **`guitar8` : accordages incomplets.** `src/instruments/guitar8/guitar8.cpp`
   déclarait « -1/2 ton » et « -2 tons » avec 7 jetons pour une 8 cordes
   (copier-coller depuis la 7 cordes : `F1` et `D1` manquants). La validation du
   TOML la refuse ; les deux accordages ont été complétés.
2. **`guitar12` : 12 cordes traitées comme 6.** Les libellés étaient écrits
   `E2E3 A2A3 …` ; `QString::split(' ')` ne produisait que 6 jetons. Les paires
   sont séparées dans le TOML.
3. **Parseur d'accordage fragile.** `_GetTuningFromString` cherchait les noms de
   notes avec `contains()` et lisait l'octave à un index dépendant de la longueur
   du nom : `E10` était lu `E1`. Le parseur Rust est strict
   (`lettre + altération + chiffres`) et refuse ce qui est ambigu.
4. **Table de frettes globale.** `harmonics::frettesPositionInX` est une table
   absolue unique (25 entrées) partagée par tous les instruments, quelle que soit
   la largeur de leur photo. Le TOML permet désormais une table par instrument
   (`fret_positions_x`).
5. **Accordages legacy incohérents.** Les `update()` de `b5.cpp`, `b6.cpp`,
   `g6.cpp` et `g12.cpp` (code mort : la classe de base `Ainstru` n'existe plus)
   contenaient des octaves recopiées depuis la guitare. Les libellés de `QAction`,
   eux, sont cohérents : ce sont eux qui ont été repris.
6. **Le modèle Qt duplique les harmoniques** à chaque clic (`SetFrette` appelle
   `ComputeAllHarmonics` sans vider la corde) ; seule la vue effaçait les
   pastilles. Le moteur Rust recalcule la corde à chaque changement.

## Quirks volontairement conservés

- **Octave affichée une octave trop haut** quand l'harmonique tombe pile sur une
  note tempérée : l'original rend `maxOctave = octave + 1`. Le 1/2 du Mi aigu est
  libellé `E5` alors que sa fréquence (659,26 Hz) est exacte. Documenté dans
  `harmonics.rs`.
- **Cents tronqués** vers zéro (`static_cast<int>`), et non arrondis.
- **Numérotation des octaves** propre à l'Atlas : l'octave 0 commence 45 demi-tons
  sous le La 440, donc octave *n* de l'Atlas = octave *n+1* scientifique.
- **`-0` affiché** pour un écart de 0 Hz négatif.
- **Zones mortes du hit-test** : la tolérance verticale d'un clic est ± la moitié
  de l'écart entre les deux premières cordes, calculée une fois pour toutes. Sur
  les instruments dont les cordes s'écartent vers le chevalet, un clic entre deux
  cordes ne pose donc aucun doigt. Reproduit à l'identique dans l'interface Rust.

## Choix tranchés depuis

- **Licence des visuels** : les photos, pastilles, médiators, empreinte, logo et
  icônes ont été réalisés par les auteurs de l'Atlas. Ils sont `MIT OR
  Apache-2.0`, comme le code (voir `LICENSE-assets.md`). La mention CC BY-NC-ND
  de `debian/control` et le `debian/copyright` hérité de `dh-make` sont abandonnés.
- **Police** : `egui` n'a pas de glyphe `♯`/`♭`. Un sous-ensemble de
  Noto Sans Symbols (OFL-1.1, sans Reserved Font Name), renommé « ADH Symbols » et
  réduit à `♪ ♭ ♮ ♯` (6 Ko), est embarqué par `adh-assets` et déclaré en repli de
  police. Le fichier est régénéré par `tools/subset_font.py`, sa licence est dans
  `LICENSE-OFL`.
- **Version d'`egui`** : `eframe`/`egui` 0.36 exigent rustc ≥ 1.95. La machine
  est en 1.93.1, donc `eframe`/`egui` sont épinglés à **0.35** (MSRV 1.92) et le
  workspace passe en `resolver = "3"` avec `rust-version = "1.92"`. Passer à 0.36
  demandera seulement `rustup update stable` et le retrait du `=`.
- **Rendu** : backend `glow` (pas `wgpu`) pour limiter la taille de compilation ;
  `x11` et `wayland` tous les deux activés.
- **Accordages personnalisés** : l'app Qt les gardait dans `settings.ini`
  (`tuningsPersoCustoms`). Ils sont désormais saisis dans l'interface, validés par
  le moteur (`Instrument::add_tuning`, même parseur que les accordages du TOML) et
  enregistrés en TOML sous `[[custom_tunings]]` — même fichier que le reste des
  réglages, donc pas de second format à maintenir.
- **Arborescence Qt** : supprimée (`src/`, `image/`, `debian/`, `deployment/`,
  `CMakeLists.txt`, `ADH.pro`, `adh_fr_FR.ts`, `install_lib_linux.sh` — 177 fichiers,
  24 Mo). La parité n'est plus assurée en re-compilant le moteur C++ mais en
  comparant aux tables gelées de `crates/adh-core/tests/golden/` ; le code d'origine
  reste accessible au tag `1.0`.
- **Emballage** : `cargo-dist` configuré dans `dist-workspace.toml` pour les
  archives, `tools/release.sh` pour la release complète — archive portable `usr/`,
  paquet Arch (`packaging/PKGBUILD`) et paquet rpm (`packaging/adh.spec`).
  L'identifiant d'application est `com.lecompositeur.adh` ; l'archive source est
  désactivée, le dépôt contient 300 Mo de ressources graphiques.

## Ce qui reste à décider

- **Paquets macOS et Windows** : `.dmg` et `.msi` demandent une CI sur ces
  plateformes, et `cargo-dist` 0.32 ne sait publier que sur GitHub Releases alors
  que le dépôt est sur Forgejo.
- **Paquet `.deb`** : la machine de développement est une Arch (`makepkg`,
  `rpmbuild`, pas `dpkg`). Le paquet Debian reste à brancher sur une machine
  `dpkg` avec le même arbre d'installation décrit par `packaging/`.
- **Boîte « À propos de Qt »** : sans objet, remplacée par « À propos » avec
  l'image `about.png`.
