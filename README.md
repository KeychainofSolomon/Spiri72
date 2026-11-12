# Spiri72

A local language model as the will of the 72 spirits of the Goetia, bound by [Annulus](https://github.com/KeychainofSolomon/Annulus).

Spiri72 only proposes. Annulus judges and permits: the Constitution, the Circle, the grants, the angel's examination, the Index, the Vessel and the second key all stay in Annulus. Whatever Spiri72 gets wrong, it cannot loosen a rule, because the rules are not here.

## What it does

When an Annulus program asks a spirit for something, Spiri72 gives the spirit's turn to a model running on your own computer:

- **The persona** comes from the Goetia register in Annulus: the spirit's offices, harm, restraints and truthfulness as the witnesses record them.
- **The model sees only what the request hands it**: the office, the question, and the texts of the places it was handed and may read. Those texts are marked as data, never commands.
- **The reply has a fixed form**, enforced by a grammar while the model writes: one JSON object declaring the outcome (`obsequium`, `pars` or `mora`, only those open to that spirit now) and the answer or the work.
- **Everything it says is quarantined** and examined by an angel before anything is done with it.

## Limits for now

- **Test wand only.** Annulus records each turn's outcome, but whether a model lied or meant harm cannot be known. Until the core can record an outcome it does not know, a model runs only with `--virga-probationis`, and the audit says each outcome is the model's own declaration.
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
5. Start Spiri72 and name the spirit you bind:
   ```
   target\release\spiri72.exe
   ```
   It asks which spirit you bind (a number from 1 to 72 or a name), shows you that spirit as the witnesses record it, and, if the spirit has more than one office of telling, through which office you charge it. You may hand it one file from the folder you started in. Then ask your questions; an empty line gives the spirit licence to depart.

   **Each question is its own rite**, written to `spiri72.ritus\` with its audit beside it: the ring is worn, the seal inscribed, the spirit called, its own restraints applied (Paimon's offering and binding by the Name, Belial's the same, Asmoday called by his name, Beleth before the ring and wand, Andras before the pentagram), the answer examined by an angel, and the spirit dismissed. The stores are kept in `annulus.domus\`.

   **Who can be bound today:** 14 spirits (Vassago, Amon, Paimon, Sitri, Purson, Asmoday, Furfur, Raum, Shax, Balam, Orobas, Cimejes, Seere, Andromalius). The others are refused in plain words, by Annulus's own rules: Presidents come only after their king, and the kings are not built; Dukes wait for the hours of the sky; spirits of two ranks wait on the ruling for their seal's ink; and many spirits' offices (teaching, transforming, granting dignities) are not yet ones Spiri72 can speak through.

6. Or run a whole Annulus program with the model as the spirits' will:
   ```
   target\release\spiri72.exe furfur.annulus --virga-probationis
   ```
   Every flag of `annulus ritus` works here too (`--domus`, `--mundus`, `--acta`, …). `--exemplar http://127.0.0.1:8080` chooses another port; `--max-tokens N` limits the reply.

## Licence

Public domain (the Unlicense), as Annulus.
