# Spécification du moteur

Référence de ce que fait `adh-core`, pour écrire l'UI sans relire le C++.

## Repères

- **Diapason** : La = 440 Hz.
- **Table de fréquences** : 9 octaves × 12 demi-tons, `440 × 2^(n/12)` pour
  `n = -45 … 62`. L'octave 0 de l'Atlas commence donc à 32,703 Hz (le Do grave)
  et l'octave *n* de l'Atlas correspond à l'octave *n+1* scientifique.
- **Partiels** : 1/2, 1/3, 1/4, 1/5, 1/6, 1/7, 1/8, 1/9, 1/10, 1/11, 1/12.
  L'index 0 = 1/2 ; la valeur du partiel est `index + 2`.
- **Corde 0** = la plus aiguë, celle du **haut** de la photo.
- **Frette 0** = corde à vide (sentinelle). Les frettes réelles vont de 1 à
  `fret_count` (22 ou 24 selon l'instrument).

## Accordages

Un accordage est une chaîne de jetons écrits **du grave vers l'aigu** :

```
E2 A2 D3 G3 B3 E4
```

Règles de conversion (`tuning::to_string_settings`) :

1. chaque jeton est analysé strictement : `A–G` (majuscule ou minuscule),
   altération optionnelle `♯ # ♭ b`, puis une octave de 0 à 9 ;
2. l'octave du libellé est décrémentée (`B0` reste à 0) ;
3. la liste est **inversée** : la corde 0 est la plus aiguë.

Un jeton mal formé ou une octave > 9 est une erreur explicite, jamais un
silence.

Les accordages ajoutés par l'interface obéissent à la même règle :
`Instrument::add_tuning(nom, cordes)` applique `to_string_settings` et refuse un
accordage qui ne décrit pas autant de cordes que l'instrument. Ils sont
enregistrés dans `settings.toml`, rattachés à l'instrument par son `id` :

```toml
[[custom_tunings]]
instrument = "guitar6"
name = "DADGAD"
raw = "D2 A2 D3 G3 A3 D4"
```

## Calcul d'un partiel

Pour la corde `s`, le partiel `p` (valeur `N = p + 2`) et la frette `f` :

1. note de base = note de la corde décalée de `f` demi-tons ;
2. `fréquence = table[octave][note] × N` ;
3. recherche dans la table de l'entrée immédiatement en dessous (`min`) et de
   celle immédiatement au-dessus (`max`) ;
4. la note retenue est la plus proche ; `cents = (fréquence − note) / ((max − min)/100)`,
   tronqué vers zéro ;
5. la fréquence est arrondie au centième de Hz, l'écart aussi.

Une harmonique 1/N se joue `N-1` fois sur la corde. Les occurrences réductibles
à un partiel déjà dessiné sont masquées (`partial::is_masked`) : le 1/4 ne
redessine pas le 1/2, le 1/6 ni le 1/3 ni le 1/2, etc. Résultat :
**45 pastilles par corde**, réparties 1, 2, 2, 4, 2, 6, 4, 6, 4, 10, 4.

## Positions

Repère de référence (celui des photos d'origine) : les cordes vont de
`x = 234` à `x = 6432`.

- **Corde à vide** : la première pastille est à `234 + 6198/N`, puis chaque
  occurrence est espacée de `6198/N`.
- **Doigt posé en case `f`** : la première pastille est à
  `frette[f] + 70 − 10`, et l'espacement est `(6432 − (frette[f] + 60)) / N`.
- **Ordonnée** : interpolation linéaire entre le début et la fin de la corde,
  `y = (x / 6432) × (fin − début) + début`.
- **Empreinte de doigt** : `x = frette[f]`,
  `y = (frette[f] / (6432 − 234)) × (fin − début) + début − 48`.

## Hit-test d'un clic

`HarmonicEngine::hit_test(x, y)` :

1. hors de `[234, 6432]` → rien ;
2. la frette retenue est la première dont `frette[f] + 100 ≥ x` (sinon
   `fret_count`) ;
3. la corde est trouvée si `y` est dans un intervalle de ± la moitié de l'écart
   entre les deux premières cordes, autour de la position de la corde à la
   frette retenue.

Le résultat est `(corde, frette)` ; l'UI appelle ensuite `toggle_fret`.

## Masques

- `NoteMask` : 12 booléens, un par note. Masquer une note masque **toutes** les
  pastilles de cette note, sur toutes les cordes.
- `PartialMask` : 11 booléens, un par partiel.
- Les deux sont `Copy` : l'UI les modifie et relit `visible_harmonics`.

## Format TOML d'un instrument

```toml
id = "guitar6"
name = "Guitare 6 cordes"
image = "g6.jpg"
fret_count = 22
strings = [[616, 540], [690, 644], [762, 744], [836, 846], [912, 946], [984, 1048]]
# facultatif : positions de frettes propres à l'instrument
# fret_positions_x = [0, 486, 814, ...]
# facultatif : emprise horizontale des cordes si la photo n'est pas la référence
# string_span_x = [234, 6432]
# facultatif : canevas scène, en pixels (défaut 6666 × 1598, la photo de référence)
# scene_size = [6666, 1598]

tunings = [
  { name = "Standard", tuning = "E2 A2 D3 G3 B3 E4" },
]
```

Validation : au moins une corde (12 maximum), `fret_count` entre 1 et 24, au
moins un accordage, et **autant de jetons que de cordes** dans chaque accordage.

## Outils de parité

- `adh-dump --csv [--frets 0,12]` : table complète du moteur Rust.
- `adh-dump --checksum` : empreinte FNV-1a de la table (les constantes testées
  dans `tests/parity.rs` en proviennent).
- `crates/adh-core/tests/golden/*.csv.gz` : tables complètes produites par le
  moteur Qt historique, gelées avant la suppression de `src/`.
- `tools/parity.sh` : construit le dump Rust et le compare ligne à ligne aux
  tables gelées.

## Repère écran de l'interface

Le moteur ne connaît que le repère scène. `crates/adh-app/src/scene.rs` le
transpose à l'écran, comme le faisait le `Flickable` du QML.

- **Canevas scène** : `Instrument::scene_size()`, par défaut 6666 × 1598 px — la
  taille des photos d'origine. La photo est étirée sur ce canevas, ce qui remet
  les harmoniques de la 12 cordes (photo fournie en 3333 × 799) au bon endroit :
  dans l'app Qt, la photo n'était pas mise à l'échelle et les pastilles
  débordaient de l'image.
- **Zoom** : `zoom = pixels écran / pixels scène`, ajusté à la fenêtre au
  démarrage et à chaque redimensionnement tant que l'utilisateur n'a pas zoomé.
  Un cran vaut 10 % (`zoomStep` du QML), minimum = l'ajustement, maximum = 4 ×
  l'ajustement. La molette zoome autour du curseur ; `returnToBounds` empêche
  toute marge vide quand la scène est plus grande que la fenêtre.
- **Pastilles** : taille écran constante de 16 × 26 px, quelle que soit
  l'échelle — l'équivalent de `sticker.scale = 1 / image.scale` dans `Sticker.qml`.
- **Empreinte de doigt** : 70 × 96 *unités scène*, donc elle grandit avec le zoom
  (`Finger.qml` était un enfant de l'image).
- **Médiators** : position et taille natives de la photo, en unités scène,
  dessinés dans l'ordre des `z` de `Plectrums.qml`. Un clic n'est accepté que si
  le pixel touché a un canal alpha > 0 (`imageData.data[3] > 0` du `Canvas` QML) ;
  un médiator dont la note est masquée est dessiné à 35 % d'opacité et reste
  cliquable.
- **Zone morte entre deux cordes** : conservée. Le hit-test utilise ± la moitié
  de l'écart entre les *deux premières* cordes, ce qui laisse des bandes où un
  clic ne pose aucun doigt — comportement exact de `userClickedOnScene`.
