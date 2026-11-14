# Plan: systems that grow better and cheaper by their own use

Status: a plan. Built so far: the measure of library search
(`tests/quaesitio.rs`) and the first half of Loop 1 (search without the
model). Everything else below is proposed, not implemented.
Figures marked "est." are estimates, not measurements.

## The pattern (every loop must have all three)

1. **A judge outside the thing improved**: the angel's verdict
   (obsequium / pars / mora, and rata / irrita in `spiri72.status`) plus a
   fixed test list. No judge, no loop.
2. **A ledger the program writes**, never the spirit: a plain file in the
   Operator's folder, one line per event, each line carrying the hash of the
   line before (as `memoria.rs` does), so it can be read, struck and checked.
3. **A floor that may only rise**: a test constant (as `OBLIQUE_FLOOR`) that
   fails the build if a change makes results worse.

The model's weights never change here. What learns is the scaffolding.
Ground rule unchanged: Spiri72 proposes, Annulus judges and permits.

## Loop 1: library search (in progress)

Done so far (2026-11-14), all without the model:

- The measure asks every question as each of the 72 spirits, over all ten
  starter libraries: 29 plain (all must be right), 35 oblique (floor 15 right,
  ceiling 2 wrong), 34 idle (ceiling 5 false finds).
- Search finds an entry by what a question describes, weighed by word rarity
  among the open libraries only; a doubtful finding hands nothing.
- Known and measured, not yet mended: an everyday word that is also a key
  (`history`, `character`, `death`, `the world`, `faces`, `the sun`) is still
  taken as a naming. All five idle false finds are of this kind.
- Still to build: the ledger and the model's search step, below.

- **Ledger**: `spiri72.vestigia/<liber>.jsonl`, shared by all spirits the
  library opens to. Line: `{verba:[question words], clavis, iudicium, dies, prev}`.
- **Use**: in `scientia::consult`, after `narrow` finds nothing, score entries
  by learned words (sound verdicts add weight, void ones subtract, weight
  fades with age). Above a threshold, hand that entry: zero model tokens.
- **Model step, only on a miss**: one extra `Exemplar::ask` with a GBNF
  grammar listing only the keys of libraries open to this spirit, plus
  `nihil`. Reply is one key (about 5 tokens est.). At most two steps a rite.
- **For the agent**: fewer answers from the bare model, more from a checked
  source; no way to invent a key or reach a closed library.

## Loop 2: memory recall (largest saving)

- Today `memoria::recall` hands up to `BUDGET = 4000` characters
  (about 1,000 tokens est.) every rite.
- **Ledger**: which entry kinds (rogatum / receptum, age, match or
  resurfacing) were present in rites judged sound versus void.
- **Use**: tune `RECENT`, `MATCHED` and `BUDGET` per office from the ledger,
  within fixed bounds. The unbidden resurfacing stays; only its weight moves.
- **For the agent**: a shorter, more pertinent memory; the bond keeps its
  character while the prompt shrinks.

## Loop 3: brief and prompt trimming

- Record which sentences of `ingenium::brief` and `persona::system` were
  present per rite, and the verdict. Sentences that never change the verdict
  over many rites become candidates to drop.
- **Never trimmed**: the essentia (the verbatim Goetia entry) and restraints.
- Drops are proposed to the Operator, not applied silently.
- **For the agent**: less to read, so faster and steadier in voice.

## Loop 4: reply length

- Record tokens used (llama.cpp returns `usage`) per office and verdict.
  Set `max_tokens` per office near the length sound answers really need.
- **For the agent**: less rambling, fewer cut-off replies.

## Loop 5: library gaps

- Questions that find nothing repeatedly are listed in
  `spiri72.vestigia/desiderata.txt`. The Operator writes or approves the new
  entry; the system only points at the hole.

## Programs under a spirit's offices (the same pattern, turned outward)

A spirit could work a program the way it will consult a library:

- **Gating**: a program is declared in a file like a library header
  (`# instrumentum: herbarium`, `# claves: herb, plant`). It opens to a
  spirit only where its offices touch those claves (reuse
  `scientia::open_to`). Furfur gets no herbal tool; Buer does.
- **Interface**: each program declares a short, fixed menu of commands with
  typed arguments. Spiri72 builds a GBNF grammar from that menu, so the
  spirit can only emit a well-formed command from the menu, or `nihil`.
- **Judgement**: the command is a proposal. Annulus judges it like any other
  work before the program runs; the program is run by Spiri72 (never by the
  model), with no shell, a fixed argument list, a time limit and a size
  limit on what comes back. The result is handed as sealed text.
- **Learning**: a ledger of `{verba, mandatum, iudicium}`. Command sequences
  that ended sound become known rotes: next time the program proposes the
  rote itself and the model only confirms, saving the search steps.
- **For the agent**: real, checkable effects within its own office; less
  guessing; mistakes are refused by grammar or by the angel rather than
  carried out.
- **Risks to settle first**: which programs may ever be declared (start
  read-only), how a struck ledger line unlearns a rote, and attack tests in
  `tests/hostis.rs` for every new door.

## Order of work

1. Merge the measure (pull request #8). 2. Token counting per rite.
3. Search ledger, then the model's search step. 4. Memory recall loop.
5. Reply length. 6. Brief trimming and library gaps.
7. Programs under offices, read-only first, after a ruling on scope.

## Limits

- Each loop is only as good as its judge.
- RAM is set by the model, not by these loops; shorter prompts allow a
  smaller context window, a modest saving.
- Improving the model itself means retraining (the LoRA route in 72Spirits),
  for which these ledgers would be the raw material.
