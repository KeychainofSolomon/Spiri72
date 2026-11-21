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

## Volumina: whole books a spirit may read from (ruled 2026-11-14; built on branch `volumina`)

The Operator's ruling: all reading material should be functional, handing a
spirit the book's own passages, not a hundred-word summary.

- **Where**: books (PDF or text) lie in a folder in the Operator's folder,
  beside `spiri72.scientia`. They never enter this repository.
- **Reading**: each book is read once with the PDF reader already here
  (`liber.rs`), split into passages of a few paragraphs with their page
  numbers, and the split is kept on disk so it is not redone at every start.
  Text PDFs first; a page that is only a picture is reported as unread.
- **Finding**: the same arithmetic as `scientia` (word rarity among the open
  books, least weight, fair part of the question, clear of the next). A
  doubtful match hands nothing.
- **Handing**: up to three passages, about 3,000 characters in all (a setting),
  each marked with book and page, sealed and fenced as untrusted handed text,
  and shown verbatim to the Operator.
- **Gating**: each book has a short header of claves, as a library has; it
  opens only to spirits whose offices hold them. Agreed gates for new matter:
  programming on `mechanical arts, mechanics, all arts`; human behaviour on
  `thoughts, love, discord, wrath`. Modern books are flagged as modern.
- **Proof before merge**: a measure like `tests/quaesitio.rs` for passages,
  and attack tests in `tests/hostis.rs` (a book can hold any text).
- The hand-written libraries stay, for short curated knowledge.

**Parts of a book with gates of their own** (built 2026-11-21). A book holds
matter for many offices (Pliny's Book II has stars, storms, earthquakes and
the sea), so a book-wide gate opened too much or too little. Done as others
have done it: libraries catalogue a chapter apart from its book (analytical
entries); Renaissance readers filed passages under heads (Erasmus, *De
Copia*); Pliny's own Book I lists what each book holds so parts may be read
alone; search engines filter by permission before searching, never after;
and medical indexing lets a machine propose subject terms that a person
approves. So: parts follow the book's own chapters, by PDF pages, in the
`.claves` file (`# pages 47-102 (Book II, the heavens): stars`); a passage
opens by its book's gate or its part's; closed passages are never candidates
(rarity is still counted over the whole book, which is its language and
reveals nothing); Claude drafts the parts and the Operator approves them.
Drafts for Pliny vol. I, Bowditch, The Prince and Behave are in the
Operator's private storage (`claves/`). Tested in `tests/volumina.rs` and
`tests/hostis.rs` (a closed part's secret never reaches the spirit).

**The index** (built 2026-11-21, `src/index.rs`). Every book was held whole
in memory, each word a string three times over: 23 books (20 MB of text)
took 222 MB and 3.9 s to prepare at every start. Now, as search engines do
it: each book's words are kept once and named by number, each passage is a
run of numbers, and for each word the index lists the passages that hold it;
the passages' text stays on disk (`.lecta/<book>.passus`) and only the few
handed are read, made plain again as they are read; the whole prepared book
is kept (`.lecta/<book>.index`), made anew when the book file, the passage
length, the Operator's concordance or the way of preparing (`VERSIO`)
changes, and anything amiss in it (cut short, a number out of range, a text
file that no longer fits) makes it anew. Measured on 23 books: 36 MB, a start
of 0.09 s, a search of 1.5 ms (was 97 ms, for only passages holding the
question's words are weighed). The search answers exactly as before: all 167
questions of the measure gave the same passages, word for word. Also: the
long ſ is read as s, and `concordantia.txt` is no longer taken for a book.

Later (ruled 2026-11-21, not built): **books of pictures.** A book whose pages
are only pictures and hold no readable script cannot be searched; the eyes
could describe each page once (kept like the text of other books, read again
only when the book changes) so the descriptions are searched instead. Each
description is handed marked as seen by sight, not the book's own words, and
any naming of what is drawn (a plant, a star) as the eyes' guess. First book:
the Voynich Manuscript (in the Operator's storage), whose script is unread;
it stays without a `.claves` file, opening to no spirit, until this is built,
then to be gated on herbs and the stars (its herbal, astrological and
pharmaceutical sections).

Built (branch `volumina`, 2026-11-20), as `src/volumina.rs`:

- A book opens only with a `.claves` file beside it (`nomen`, `claves`,
  `aetas`); a book without one is not read. What a PDF holds is kept in
  `spiri72.volumina/.lecta` and read again only when the book's file changes.
- Long books need their own arithmetic, for rarity among a thousand passages
  never reaches the libraries' fixed floor. A passage is weighed by the
  rarity of the question's words it holds, more where a word recurs, where
  two of them stand side by side as asked, and where a heading line of the
  passage holds them. It must hold two of the words and half the question.
  Passages near each other in a book are one place; more than four places
  answering nearly as well is doubt, and nothing is handed.
- Contents and index passages (dot leaders, lines ending in page numbers)
  are set aside; running heads (`The Law of Demeter 97`) are no headings.
- A passage opens at a heading. A short heading naming the question's rarest
  word, where that word recurs in the passage, finds it alone (`Tannins`,
  `Psoriasis`); a question of one everyday word still finds nothing. Near
  passages within four pages of each other are one place.
- Measure: `tests/volumina.rs` on a book written for the test (14 of 14
  plain, oblique floor 6 and ceiling 1 wrong, idle ceiling 1); attacks in
  `tests/hostis.rs` (forged passage ends, commands, hidden and control
  characters, closed and ungated books, floods).
- Trial on the Operator's own Clean Code (not in the repository): 1,142
  passages from 460 pages, 167 tables set aside, 3 pages pictures only.
  16 of 17 questions on its matter found the right section; 1 of 16 idle
  questions found a passage (`build a house`, the foreword's architect).
  Known weak: `keep my classes small` ranks a concurrency heading first.
- The concordance (`src/concordantia.rs`, 2026-11-20): word families cut to
  one root (`digestion`, `digestive`); what each book sets out itself
  (`Single Responsibility Principle (SRP)`, `Hypericum perforatum (St.
  John's wort)`, kept where used twice); and the Operator's own groups in
  `spiri72.volumina/concordantia.txt` (`liver = hepatic`). Passages and
  questions are both read through it. Joined letters (`ﬁ`, `ﬀ`) are spelled
  out; orders to the spirit (`write`, `teach`) and the frame of a comparison
  (`the difference between`) are words of asking.
- Measured with a fixed list (`tools/volumina_aurum.txt`: 117 questions with
  the pages that answer them, 40 of them written before any change and never
  tuned on, and 10 idle questions for each book). Scores: right page first /
  wrong page first / nothing; right page anywhere among those handed; idle
  finds of 50. Before the concordance: 82 / 18 / 17; 92; 4. After: 89 / 20 /
  8; 101; 4. Each part was switched off in turn to see what it adds: word
  families +4 found, headings only at a passage's top +3, the book's own
  names about nothing overall (it is St. John's wort), and handing in the
  books' order puts a right page first more often than handing the best
  first. A doubtful match is now more than seven places answering alike.
- Earlier trial on all five of the Operator's books (2026-11-20; each with 15 or 17
  questions on its matter and 10 or 16 idle ones; right / wrong / nothing,
  then idle finds): Clean Code 16/0/1, idle 1 of 16; Medical Herbalism
  12/2/1, idle 2 of 10; Astronomy for Mere Mortals 13/1/1, idle 0 of 10;
  SICP 11/0/4, idle 0 of 10; Behave 10/3/2, idle 1 of 10. Text read with
  pdftotext there; the program reads with hayro, which may differ (SICP's
  fonts drop the letters of some ligatures: `e` for `The`, `paern`).

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
