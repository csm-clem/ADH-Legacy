#!/usr/bin/env python3
"""Régénération de `crates/adh-assets/assets/fonts/adh-symbols.ttf`.

`egui` n'a pas de glyphe `♯` `♭` `♮` `♪` dans ses polices embarquées. On embarque
donc un sous-ensemble de Noto Sans Symbols (OFL-1.1, sans Reserved Font Name),
**renommé « ADH Symbols »** : l'OFL interdit à une version modifiée de garder le
nom d'origine.

Nécessite `fonttools` :

    python3 tools/subset_font.py
"""

import pathlib

from fontTools import subset
from fontTools.ttLib import TTFont

SRC = pathlib.Path("/usr/share/fonts/noto/NotoSansSymbols-Regular.ttf")
OUT = pathlib.Path(__file__).resolve().parent.parent / (
    "crates/adh-assets/assets/fonts/adh-symbols.ttf"
)

# ♪ ♭ ♮ ♯
CODEPOINTS = [0x266A, 0x266D, 0x266E, 0x266F]

# Déclaration de licence recopiée dans la police modifiée (obligation OFL).
COPYRIGHT = (
    "Copyright 2022 The Noto Project Authors "
    "(https://github.com/notofonts/symbols), modified as ADH Symbols."
)
LICENSE = (
    "This Font Software is licensed under the SIL Open Font License, Version 1.1. "
    "This license is available with a FAQ at: https://scripts.sil.org/OFL"
)


def main() -> None:
    if not SRC.exists():
        raise SystemExit(f"police source absente : {SRC}")

    font = TTFont(SRC)
    cmap = font.getBestCmap()
    missing = [hex(code) for code in CODEPOINTS if code not in cmap]
    if missing:
        raise SystemExit(f"glyphe(s) manquant(s) dans {SRC.name} : {missing}")

    options = subset.Options()
    options.glyph_names = False
    options.notdef_outline = True
    options.recommended_glyphs = False
    options.name_IDs = ["*"]
    options.name_legacy = True
    options.name_languages = ["*"]

    subsetter = subset.Subsetter(options)
    subsetter.populate(text="".join(chr(code) for code in CODEPOINTS))
    subsetter.subset(font)

    # Renommage : une sous-partie d'une police ne doit pas garder son nom d'origine.
    renamed = {1, 2, 3, 4, 6, 16, 17}
    kept = renamed | {0, 11, 13, 14}
    for record in font["name"].names:
        if record.nameID in renamed:
            record.string = "ADH Symbols"
        elif record.nameID == 0:
            record.string = COPYRIGHT
        elif record.nameID == 13:
            record.string = LICENSE
    font["name"].names = [r for r in font["name"].names if r.nameID in kept]

    OUT.parent.mkdir(parents=True, exist_ok=True)
    font.save(OUT)

    check = TTFont(OUT)
    print(f"écrit {OUT} ({OUT.stat().st_size} octets)")
    print("glyphes :", sorted(hex(code) for code in check.getBestCmap()))
    for record in check["name"].names:
        if record.nameID in (1, 0, 13):
            print(f"nameID {record.nameID} : {record.toUnicode()}")


if __name__ == "__main__":
    main()
