# For Fable: a sweep of Spiri72

You are asked to look over Spiri72 and find better solutions than the ones in
place. You can hold the whole of this codebase and its rules in mind at once,
reason across files, and weigh evidence carefully: use that. The most useful
thing you can return is a short list of changes, each one argued from evidence
in this repository or from a measurement, ranked by what it would gain.

## Start here

1. Read `PLAN.md`, above all **"The first live sitting (2026-11-21)"**: what the
   Operator saw in a live test with Furfur, what was fixed, and what is open.
2. Read `README.md` for how the Operator uses the program.
3. Skim the Annulus specs the program obeys: `docs/spec/authority.md`,
   `docs/spec/dynamic.md` in KeychainofSolomon/Annulus (branch `luna-crescens`,
   the commit pinned in `Cargo.toml`).

## What Spiri72 is

A local Windows program: the 72 spirits of the Goetia, each answering through a
small local model (Qwen3 via llama.cpp on 127.0.0.1), every question run as a
rite in Annulus, a language whose Constitution decides what a spirit may do.
Each answer is examined by an angel (checks on the spirit's bonds and quotations)
before it is shown.

The parts you will touch most:

| File | What it does |
|---|---|
| `src/agens.rs` | one question: what the spirit is handed, the model call, the reply read |
| `src/persona.rs` | the spirit's instructions (system and user prompt) |
| `src/volumina.rs` | whole-book search: passages, rarity scoring, the doubt rule, gates |
| `src/index.rs`, `src/concordantia.rs` | the kept index; the concordance (synonyms, stems) |
| `src/scientia.rs` | the small knowledge libraries (one entry per term) |
| `src/liber.rs` | reading PDFs with hayro: letters set in lines, columns |
| `src/latina.rs` | the Latin reading aid |
| `src/llama.rs` | the request to llama.cpp (sampling settings) |
| `src/aula.rs`, `aula/index.html` | the binding page |
| `src/sessio.rs`, `src/main.rs` | sessions, the wand, `--sanci` |

## What is open (from the live sitting)

In rough order of harm:

1. **Facts from nothing, and giving way under doubt.** With no book, a guess
   still earns RATUM; "Are you SURE?" turned Louis XII (right) into Francis II.
   Proposals: a doubt rule, a "verified" mark in the spirit's book, an
   "unverified" label, asking twice. Judge them, or find better.
2. **Borrowed church morals.** Asked about sex, Furfur preached Catholic
   marriage ethics, which is not in his record. Causes and a proposed line are
   in PLAN.md.
3. **Loose matches.** One metaphor ("engine", "car" in Behave p. 614) earns a
   passage. The scoring is in `volumina.rs` `score` and `consult_as`; the
   thresholds are constants at the top of the file.
4. **Stretching a passage** (DRD4 tied to romantic love, which the page does
   not say). The angel checks quotations letter for letter, but not
   paraphrase.
5. **Passages cut at the page's edge**, with stray page numbers.
6. **A flat voice** since the spirit was told not to narrate its binding.

Look beyond this list too. Fresh eyes on the prompt in `persona.rs`, the
search, and the reading of PDFs are the most likely to pay.

## How to measure

Search can be measured without the model:

```
pdftotext <book>.pdf <dir>/<name>.txt     # for each book named in the gold file
python3 tools/volumina_score.py <dir> -v
```

The books are PDFs in the private repository KeychainofSolomon/Spiri72-PDF-Storage
(cleancode, herbal = Hoffmann's Medical Herbalism, astronomy, sicp, behave =
Sapolsky). `tools/volumina_aurum.txt` holds 117 questions with the pages that
answer them, and 10 idle questions that should find nothing. Baseline on
2026-11-21: **right 89, wrong 20, nothing 8; any right 101 of 117; idle finds 4.**
A search change should raise "right" and "any right" without raising "idle
finds"; report the numbers before and after. `examples/volumina_ask.rs` asks
any questions over any books (`V=1` shows the passages).

The model's answers cannot be measured in CI (no model there). Where a change
is about answers, say how the Operator should test it live, with example
questions.

## Rules (the Operator's, all binding)

- **What the Constitution does not grant is denied.** Office decides what a
  spirit may do; its profile decides how it behaves. Do not widen a spirit's
  reach to make an answer better.
- **The authentic texts govern** the spirits, their offices, hours and rites.
  The Moon follows the Key of Solomon (increasing in light), by the Operator's
  ruling; see `authority.md` §8.3. Cite sources for anything traditional.
- **No copyrighted book text in this repository**, not even a line in a test.
  Book text stays in the private storage repository and on the Operator's PC.
- **The model stays local** (127.0.0.1). Spiri72 never goes online; only the
  updater does, when the Operator runs it.
- **The Operator has no Rust toolchain.** Changes are proven by tests and by
  GitHub Actions (`.github/workflows/build.yml`), which also publishes the
  Windows build as the `proba` release that `update.cmd` installs.
- **Commits are authored "Solomon"**, with no other attribution lines.
  Work on a branch from `volumina`; do not merge into `main`.
- Plain words in what the Operator reads: he does not code.

## What to hand back

- A ranked list: each change, the evidence for it, the expected gain, the risk.
- For search changes: scores before and after.
- For prompt changes: the exact new wording, and live questions to test it.
- Anything you find that contradicts PLAN.md, said plainly.
- If you build changes, keep each one a separate commit with its own test, so
  the Operator can keep some and drop others.
