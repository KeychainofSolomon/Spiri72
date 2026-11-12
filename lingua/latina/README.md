# The classical Latin lexicon

Built by `tools/latina.py` from William Whitaker's WORDS (`DICTLINE.GEN`, `INFLECTS.LAT`, `UNIQUES.LAT`; https://github.com/mk270/whitakers-words). Whitaker gave "permission ... freely ... for any and all use of program and data" (`LICENCE.txt`).

- `verba.tsv`: the dictionary entries, with four stems, part of speech, declension or conjugation, the dictionary form and the meaning.
- `flexiones.tsv`: the endings, with the stem each attaches to.
- `singularia.tsv`: irregular forms read whole.

**Classical first, later Latin marked.** WORDS dates every entry and ending. Entries dated X (any age), A (archaic), B (early) and C (classical) are read as classical. Entries dated D (late, c. 200–600), E (later, c. 600–1000), F (medieval, c. 1000–1500) and G (scholarly, Renaissance and after) are kept too, so Church and grimoire words such as *angelus*, *exorcista* and *marchio* are found, but every reading that rests on one of them is marked with its period and comes after the classical readings. H (modern coinages) is left out.

**Spelling** is normalised as Spiri72 reads: lower case, *j* as *i*, *v* as *u*.

**One addition:** WORDS parses *sum, esse* in its program code rather than its data, so `tools/latina.py` adds that one entry itself.

To rebuild: `python3 tools/latina.py PATH-TO-WHITAKERS-WORDS`.
