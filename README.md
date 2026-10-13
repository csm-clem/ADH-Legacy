# L'Atlas des harmoniques

Un instrument est pincé à un point précis de la corde : quelle note sort, à
quelle fréquence, et à combien de cents de la note tempérée ? L'Atlas répond ça
visuellement, sur une photo de l'instrument, pour les onze premiers partiels.

Application née en C++/Qt en 2011, réécrite en Rust avec `egui`/`eframe` : la
version 2.0 ne contient plus que du Rust.

## État du projet

Le moteur de calcul est **réécrit, testé et validé contre l'application Qt** :
les tables produites par `ModelHarmonicManager` sont gelées dans
`crates/adh-core/tests/golden/` et `tools/parity.sh` les compare ligne à ligne au
moteur Rust — **41 221 lignes identiques** (corde à vide et doigt posé,
7 instruments, 65 accordages).

L'interface `egui`/`eframe` est **en place** : instruments, accordages (y compris
personnalisés et persistés), zoom, glisser, pastilles, infobulles, médiators
cliquables, masques de notes et de partiels, empreinte de doigt, FR/EN, réglages
persistés.

```sh
cargo run -p adh-app     # ou F5 dans VS Code
```

Voir `docs/MIGRATION.md` (plan, décisions, bugs corrigés) et `docs/SPEC.md`
(spécification du moteur et du repère écran).

## Arborescence

```
Cargo.toml                     workspace
LICENSE-MIT / LICENSE-APACHE   licence du code et des visuels : MIT OR Apache-2.0
LICENSE-assets.md              idem, avec l'historique corrigé
REUSE.toml                     licence par chemin

crates/adh-core/               moteur, aucune dépendance GUI
  src/note.rs                  12 demi-tons, noms FR/EN, sprites de pastilles
  src/partial.rs               11 partiels, règles de masquage des doublons
  src/frequency.rs             table 9 octaves × 12 notes autour du La 440
  src/tuning.rs                parseur d'accordages (« E2 A2 D3 … », « F♯3 »)
  src/instrument.rs            modèle d'instrument + validation TOML
  src/geometry.rs              repère scène, table de frettes, Point
  src/harmonics.rs             HarmonicEngine : fréquences, cents, positions, hit-test
  src/masking.rs               masques de notes et de partiels
  src/parity.rs                empreinte et format CSV de comparaison
  src/builtin.rs               instruments embarqués
  src/bin/adh_dump.rs          export CSV / empreinte
  assets/instruments/*.toml    basse 4/5/6, guitare 6/7/8/12
  tests/                       25 tests d'intégration + tables de référence du moteur Qt

crates/adh-assets/             visuels embarqués (photos, pastilles, médiators,
                               empreinte, logo, icônes, police de symboles ♯♭)

crates/adh-app/                interface egui/eframe (binaire `adh-app`)
  src/main.rs                  fenêtre, icône, backend glow
  src/app.rs                   menus, barres, scène, infobulle, à propos, accordages personnalisés
  src/scene.rs                 zoom, glisser, repère scène ↔ écran, médiators
  src/textures.rs              décodage PNG/JPEG et cache de textures
  src/settings.rs              réglages persistés en TOML
  src/i18n.rs                  libellés FR et EN
  src/links.rs                 ouverture des liens du menu Aide

.vscode/                       launch.json (F5) et tasks.json

dist-workspace.toml            configuration de cargo-dist (archives de release)
packaging/                     .desktop, AppStream, PKGBUILD Arch, spec rpm
tools/parity.sh                dump Rust + diff contre les tables de référence
tools/release.sh               vérifications + archives + paquet Arch + paquet rpm
tools/subset_font.py           régénère la police de symboles ♪ ♭ ♮ ♯ (OFL-1.1)

crates/adh-core/tests/golden/  tables complètes du moteur Qt, gelées : 20 611 et
                               41 221 lignes (107 Ko et 259 Ko compressés)
```

L'arborescence Qt (`src/`, `image/`, `debian/`, `deployment/`, `CMakeLists.txt`,
`ADH.pro`) a été supprimée. Elle reste dans l'histoire du dépôt, au dernier tag
de l'ère Qt : `git checkout 1.0 -- src/`.

## Construire et tester

```sh
cargo build
cargo test
cargo run -p adh-app        # l'application
cargo run --bin adh-dump
```

Dans VS Code, **F5** construit et lance `target/debug/adh-app` (CodeLLDB) ;
les tâches `cargo: test`, `cargo: clippy` et `parité Qt ↔ Rust` sont proposées
par `Ctrl+Maj+P` → *Tasks: Run Task*.

Deux variables d'environnement aident au diagnostic :

```sh
ADH_SCREENSHOT=/tmp/adh.png cargo run -p adh-app   # capture la fenêtre puis quitte
ADH_DEBUG=1 cargo run -p adh-app                   # hit-test et actions sur stderr
```

Rejouer la comparaison de parité (Rust uniquement, aucun prérequis Qt) :

```sh
./tools/parity.sh
```

## Emballage

```sh
cargo install cargo-dist      # une seule fois
./tools/release.sh            # vérifications, puis tout dans dist/
./tools/release.sh --quick    # sans tests, clippy ni parité
```

| Fichier produit dans `dist/` | Contenu |
|---|---|
| `adh-app-<cible>.tar.xz` | binaire `adh-app`, licences et README (cargo-dist) |
| `adh-core-<cible>.tar.xz` | binaire `adh-dump` (cargo-dist) |
| `adh-<version>-<cible>.tar.gz` | archive portable : `usr/bin`, `.desktop`, AppStream, icônes hicolor, licences |
| `adh-<version>-1-x86_64.pkg.tar.zst` | paquet Arch (`makepkg`) |
| `adh-<version>-1.x86_64.rpm` | paquet rpm (`rpmbuild`) |
| `adh-<version>.sha256` | empreintes SHA-256 de tout ce qui précède |

L'identifiant d'application est `com.lecompositeur.adh` (voir `packaging/`) :
`adh-app` s'installe dans `/usr/bin`, avec son entrée `.desktop`, son fichier
AppStream et son icône hicolor 256 px + SVG.

`./tools/release.sh --publish` publie le tout sur Forgejo quand `FORGEJO_URL`,
`FORGEJO_TOKEN` et `FORGEJO_REPO` sont posés.

macOS (`.dmg`) et Windows (`.msi`) demandent une CI sur ces plateformes :
`cargo-dist` 0.32 ne sait publier que sur GitHub Releases alors que le dépôt est
sur Forgejo. Les cibles correspondantes sont listées en commentaire dans
`dist-workspace.toml`.

## Ajouter un instrument

Ajouter un fichier dans `crates/adh-core/assets/instruments/`, puis une ligne
dans `src/builtin.rs`. Pas de recompilation de plugin, pas de `CMakeLists.txt`,
pas de bibliothèque partagée :

```toml
id = "guitar10"
name = "Guitare 10 cordes"
image = "g10.jpg"
fret_count = 24
strings = [[616, 540], [690, 644], [762, 744], [836, 846], [912, 946],
           [984, 1048], [616, 540], [690, 644], [762, 744], [836, 846]]
tunings = [
  { name = "Standard", tuning = "E2 A2 D3 G3 B3 E4 E2 A2 D3 G3" },
]
```

## Licence

- **Code** : `MIT OR Apache-2.0` (`LICENSE-MIT`, `LICENSE-APACHE`).
- **Visuels** de `crates/adh-assets/assets/` : **`MIT OR Apache-2.0`** eux aussi,
  puisqu'ils ont été réalisés par les auteurs de l'Atlas. Voir
  `LICENSE-assets.md` pour l'historique (la mention CC BY-NC-ND de l'ancien
  `debian/control` et le `debian/copyright` hérité de `dh-make` sont abandonnés
  avec l'arborescence Qt).
- **Police** `crates/adh-assets/assets/fonts/adh-symbols.ttf` : sous-ensemble de
  Noto Sans Symbols, **OFL-1.1** (`LICENSE-OFL`), renommé « ADH Symbols » (les
  glyphes `♪ ♭ ♮ ♯` qui manquent aux polices `egui`). Régénéré par
  `tools/subset_font.py`.
- L'arborescence Qt supprimée (`src/`, `image/`, `debian/`, …) relevait de la même
  licence MIT OR Apache-2.0, sauf `debian/copyright`, faux, disparu avec elle. Elle
  reste consultable au tag `1.0`.

Le choix de la licence est une décision du détenteur des droits de l'œuvre
originale.

