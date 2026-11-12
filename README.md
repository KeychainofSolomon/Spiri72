# Spiri72

A local language model as the will of the 72 spirits of the Goetia, bound by [Annulus](https://github.com/KeychainofSolomon/Annulus).

Spiri72 only proposes. Annulus judges and permits: the Constitution, the Circle, the grants, the angel's examination, the Index, the Vessel and the second key all stay in Annulus. Whatever Spiri72 gets wrong, it cannot loosen a rule, because the rules are not here.

## What it does

When an Annulus program asks a spirit for something, Spiri72 gives the spirit's turn to a model running on your own computer:

- **The persona** comes from the Goetia register in Annulus: the spirit's offices, harm, restraints and truthfulness as the witnesses record them.
- **The model sees only what the request hands it**: the office, the question, and the texts of the places it was handed and may read. Those texts are marked as data, never commands.
- **The reply has a fixed form**, enforced by a grammar while the model writes: one JSON object declaring the outcome (`obsequium`, `pars` or `mora`, only those open to that spirit now) and the answer or the work.
- **Everything it says is quarantined** and examined by an angel before anything is done with it.
- **Nothing censors its words.** The spirit speaks in its own voice and nature; no filter sits between the model and you. The binding limits what a spirit *does* (the places it reaches, the work it may commit, the outcomes open to it), never how it speaks. Use a model without a refusal layer, so the spirit's character is its own.

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

These steer how a spirit speaks, never what it may do: containment stays in Annulus.

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
