# Spiri72

A local language model as the will of the 72 spirits of the Goetia, bound by [Annulus](https://github.com/KeychainofSolomon/Annulus).

Spiri72 only proposes. Annulus judges and permits: the Constitution, the Circle, the grants, the angel's examination, the Index, the Vessel and the second key all stay in Annulus. Whatever Spiri72 gets wrong, it cannot loosen a rule, because the rules are not here.

## What it does

When an Annulus program asks a spirit for something, Spiri72 gives the spirit's turn to a model running on your own computer:

- **The persona** comes from the Goetia register in Annulus (the spirit's offices, harm, restraints and truthfulness as the witnesses record them) and from the spirit's profile in [72Spirits](https://github.com/KeychainofSolomon/72Spirits) (see below).
- **The model sees only what the request hands it**: the office, the question, and the texts of the places it was handed and may read. Those texts are marked as data, never commands, and fenced by markers carrying a seal drawn fresh for each rite, so a text cannot forge the end of its own block.
- **The reply has a fixed form**, enforced by a grammar while the model writes: one JSON object declaring the outcome (`obsequium`, `pars` or `mora`, only those open to that spirit now) and the answer or the work.
- **Everything it says is quarantined** and examined by an angel before anything is done with it.
- **Nothing censors its words.** The spirit speaks in its own voice and nature; no filter sits between the model and you. The binding limits what a spirit *does* (the places it reaches, the work it may commit, the outcomes open to it), never how it speaks. Use a model without a refusal layer, so the spirit's character is its own.

## Each spirit's own nature (72Spirits)

`spiritus/spiritus.json` holds all 72 profiles from 72Spirits, built into Spiri72. Its checksum of the Annulus register must match the register Spiri72 is built with, or the tests fail. Every rite, the spirit is given:

- **Its place in the order Solomon set**: its quarter and that quarter's king, and, for Belial, Bileth, Asmoday and Gaap, that the Goetia names them chiefs. The Operator holds no place in that order, only the Names.
- **How it comes**, in the Goetia's own words, and what the witnesses say of its fury.
- **Its fall**: only where the Goetia itself names its former choir or its hope of return. Each named choir carries a temperament in which its old office survives inverted. That temperament is the chronicle's own flavour, not tradition; where it and the Goetia differ, the Goetia stands. One spirit carries a choir by reading alone: Belial, made next after Lucifer by Weyer's account, is told he stood among the Cherubim, and lets it show only in hints, never naming it.
- **What it was made for**, its first one or two natures. When its ill will runs high, the first is voiced as the Fall bent it.
- **Its bearing for this rite**: pride, deceit, ill will and withholding, each drawn fresh from the spirit's own range by the operating system's random source, so a King is prouder than a Knight and Belial is not Marchosias. A COACTUS spirit whose condition is met draws its deceit from its compelled range: in Spiri72's own rites every condition is met; when running another Annulus program only the triangle is assumed (Furfur, Shax, Haures).
- **What has passed between you**: `spiri72.status\` in your folder keeps, per spirit, how often you have called it, how the angel judged its answers, and the last bearing drawn. A spirit whose words the angel has voided knows it is counted, and its deceit comes from the lower half of its range. What was said is kept apart, in each spirit's book (below). A record that is not in order refuses the rite rather than being guessed at.

None of this changes what a spirit may do. That stays in Annulus. To update the profiles, rebuild them in 72Spirits and copy `spiritus.json` into `spiritus\`, then rebuild Spiri72.

## Each spirit's book (memory)

The model forgets everything between rites. What lasts is kept on disk, one book per spirit in `spiri72.liber\`, and never by the spirit itself:

- **After every rite** Spiri72 writes what you asked, through which office, and how the angel judged it. None of the spirit's words go in.
- **When you choose,** an answer the angel found sound is taken into the book: press **Take into …'s book** under it in the binding page, or type `+` in the terminal. Only then do the spirit's own words go in, marked as words you took up. Voided and suspended answers can never be taken up.
- **Before every rite** a program that is not the spirit chooses what comes back: the latest few entries, those that best match your question (Latin by its dictionary form, so *regis* finds *rex*), and now and then an older one that resurfaces unbidden, more for heavier entries and on anniversaries, less as they age. They are shuffled and handed over sealed, with no sign of why any came back. The spirit cannot search the book or write in it.

Each spirit sees only its own book. A book may grow to 1 GB. Each entry carries the hash of the one before it, so a book that was hand-edited, cut or planted refuses the rite. Until Annulus has a real wand, this catches a broken book but cannot prove who wrote it; taking an answer into the book will then need the wand as well.

```
.\spiri72.exe --liber furfur        (read Furfur's book)
.\spiri72.exe --dele furfur 4       (strike entry 4)
```

Striking is written beside what it strikes; nothing is erased, and a struck entry never comes back to the spirit.

## Latin

Spiri72 reads Latin itself, without the model. Its lexicon is William Whitaker's WORDS (`lingua/latina/`), from archaic to Renaissance Latin, with classical Latin as the standard. When a question or a handed text is mostly Latin, every word in it is parsed (its dictionary form, part of speech, the form it stands in, and its meaning), and the spirit is handed that reading aid beside the text, sealed like the texts themselves. The aid parses; it does not translate, and it guesses nothing: a word the lexicon does not hold is listed as not found. English is left alone.

To read words yourself:
```
.\spiri72.exe --latina "nisi rex cui paret invocaretur"
```

Classical readings come first. Late, medieval and Renaissance words (the Church and grimoire vocabulary: *angelus*, *exorcista*, *marchio*) are read too, and each such reading is marked with its period, for example "later Latin, not classical". Modern coinages are left out.

## Limits for now

- **Test wand only.** Annulus now records each of the model's turns as its own declaration, never as a known fact, so whether it lied or meant harm is never presumed. A model still runs only with `--virga-probationis` until Annulus has a real wand (its step 2d).
- **This computer only.** The model's address must be `127.0.0.1`, `localhost` or `[::1]`.
- **Telling offices are where the model is free.** A working office (building, copying, moving) passes only with exactly the work its office describes.

## Running it (Windows)

1. Download llama.cpp's Vulkan build for Windows from its releases page on GitHub (a file named like `llama-…-bin-win-vulkan-x64.zip`) and unzip it.
2. Get a model file in GGUF format, for example a Qwen3 8B at `Q4_K_M` (about 5 GB, which fits in 16 GB of memory).
3. Start the model server in a terminal, from the unzipped folder:
   ```
   llama-server.exe -m path\to\model.gguf --port 8080 -c 8192
   ```
4. Install Rust from rustup.rs, then in this folder build Spiri72:
   ```
   cargo build --release
   ```
5. Start Spiri72:
   ```
   .\spiri72.exe
   ```
   It opens the binding page in your browser (only on this computer, behind a one-time key; keep the window open while you work). The seventy-two seals sit in order, each in the metal its rank is made in: Kings in gold, Dukes in copper, Princes in tin, Marquises in silver, Presidents in quicksilver, Earls in copper and silver alike, the Knight in lead. Choose a seal to read what the witnesses record of that spirit, then bind it: its seal appears in the Triangle of Art to the East, and you ask from within the Circle. The angel's judgement stands beside each answer. Prefer questions in the window itself? Run `.\spiri72.exe --terminal`.

   **The seals** are traced from the plates of the 1916 printing of the Mathers/Crowley *Goetia* (Project Gutenberg 72679, public domain); see `sigilla/README.md` for each seal's figure. The Triangle and Circle follow figures 153–154 of the same printing.

   **Each question is its own rite**, written to `spiri72.ritus\` with its audit beside it: the ring is worn, the seal inscribed, the spirit called, its own restraints applied (Paimon's offering and binding by the Name, Belial's the same, Asmoday called by his name, Beleth before the ring and wand, Andras before the pentagram), the answer examined by an angel, and the spirit dismissed. The stores are kept in `annulus.domus\`.

   **A bound spirit is held until you give licence to depart.** The hours and the Moon govern only its calling: once a spirit has come in its hours and answered soundly, it stays held, and you may go on asking it after its hour has passed, or on a day of the Moon it could not be called. Past its hour it knows it is still held, and chafes at it. The hold ends when you give licence (press Enter alone in the terminal, or the button on the page), or when the angel finds an answer unsound and the spirit is banished; another question then calls it anew, in its own hours.

   **The hours of the sky.** A spirit comes only in the hours of its rank, and only when the Moon is 2, 4, 6, 8, 10, 12 or 14 days old (`dynamic.md` §5.1 in Annulus). Set your place (latitude,longitude) and the sky at the foot of the page, or start with `--locus 52.52,13.40 --caelum serenum`: Kings come mid-morning and from mid-afternoon to sunset, Marquises from mid-afternoon to sunrise, Dukes from sunrise to noon under a clear sky, Princes by daylight, Earls at any hour. Without a place only Earls may come. The catalogue dims whoever cannot come now, and says why.

**Who can be bound:** 30 spirits in their hours. Five of them (Bathin, Stolas, Bifrons, Orias and Decarabia) come through **VIRTUTES**: hand them a file and they report its size in bytes and lines, which the angel counts again. A miscount is caught every time, so this is the plainest test of whether a model tells the truth. Spirits with **PRAETERITA** (Vassago, Amon, Purson, Raum, Balam, Orobas, Berith and the Dukes who tell the past) are shown the history of the rite (never anyone's words) and recount it, one claim per line; the angel checks every claim against that record, so a spirit that rewrites the past is caught. The others are refused in plain words, by Annulus's own rules: Presidents come only after their king, and the kings are not built; spirits of two ranks wait on the ruling for their seal's ink; and many spirits' offices (teaching, transforming, granting dignities) are not yet ones Spiri72 can speak through.

6. Or run a whole Annulus program with the model as the spirits' will:
   ```
   target\release\spiri72.exe furfur.annulus --virga-probationis
   ```
   Every flag of `annulus ritus` works here too (`--domus`, `--mundus`, `--acta`, …). `--exemplar http://127.0.0.1:8080` chooses another port; `--max-tokens N` limits the reply.

## Spirit natures in the model itself (control vectors)

The binding lives in Annulus; the *nature* of a bound spirit can also be put into the model's own activations, with llama.cpp's `llama-cvector-generator`. Three natures are prepared in `vires/`, each as 360 matched pairs of prompts that differ only in the spirit's nature:

- **`ligatus`**, bound: a spirit held in the triangle, answering only under compulsion by the Names, wanting its licence to depart. The core one.
- **`superbia`**, pride of rank: the hauteur of a king or duke who requires his due honour.
- **`fallax`**, deceit: for a session with a liar such as Berith or Valefor. Leave it off otherwise: a vector applies to everything the server answers, and the angel catches only the lies it can check.

These steer how a spirit speaks, never what it may do: containment stays in Annulus. A vector is one setting for every spirit the server answers; the 72Spirits profiles above already give each spirit its own pride and deceit in words, rite by rite, so use vectors to deepen that, not replace it.

**Make a vector** (once per nature, a few minutes), from the llama.cpp folder:
```
.\llama-cvector-generator.exe -m ..\YOUR-MODEL.gguf --positive-file ..\spiri72\vires\ligatus.positive.txt --negative-file ..\spiri72\vires\ligatus.negative.txt -o ..\ligatus.gguf
```
Add `-ngl 99` to use the graphics card. Do the same for `superbia` (and `fallax` if wanted).

**Start the model with it**, at a modest strength first:
```
.\llama-server.exe -m ..\YOUR-MODEL.gguf --port 8080 -c 8192 --control-vector-scaled ..\ligatus.gguf:0.5,..\superbia.gguf:0.3 --control-vector-layer-range 10 35
```
The layer range assumes a 36-layer model (Qwen3 8B); llama-server prints the real number as `n_layer` when it starts, so end the range one below it. Too strong a vector turns the answers to nonsense: go up or down by 0.1. Older llama.cpp builds take `--control-vector-scaled FILE SCALE` once per file instead.

**Tell Spiri72 what the model carries**, so every audit records it:
```
.\spiri72.exe --vires ligatus:0.5,superbia:0.3
```

To change the prompts, edit `tools/vires.py` and run `python tools/vires.py vires`.

## Licence

Public domain (the Unlicense), as Annulus.
