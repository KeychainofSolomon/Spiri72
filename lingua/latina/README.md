# The classical Latin lexicon

Built by `tools/latina.py` from William Whitaker's WORDS (`DICTLINE.GEN`, `INFLECTS.LAT`, `UNIQUES.LAT`; https://github.com/mk270/whitakers-words). Whitaker gave "permission ... freely ... for any and all use of program and data" (`LICENCE.txt`).

- `verba.tsv`: the dictionary entries, with four stems, part of speech, declension or conjugation, the dictionary form and the meaning.
- `flexiones.tsv`: the endings, with the stem each attaches to.
- `singularia.tsv`: irregular forms read whole.

**Classical only.** An entry or ending is kept when WORDS dates it X (any age), A (archaic), B (early) or C (classical). Late, medieval, scholarly and modern Latin (D to H) is left out, so Church and grimoire words such as *angelus*, *exorcista* and *pentaculum* are not found.

**Spelling** is normalised as Spiri72 reads: lower case, *j* as *i*, *v* as *u*.

**One addition:** WORDS parses *sum, esse* in its program code rather than its data, so `tools/latina.py` adds that one entry itself.

To rebuild: `python3 tools/latina.py PATH-TO-WHITAKERS-WORDS`.
