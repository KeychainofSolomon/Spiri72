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
5. Run an Annulus program with the model as the spirits' will:
   ```
   target\release\spiri72.exe furfur.annulus --virga-probationis
   ```
   Every flag of `annulus ritus` works here too (`--domus`, `--mundus`, `--acta`, …). `--exemplar http://127.0.0.1:8080` chooses another port; `--max-tokens N` limits the reply.

## Licence

Public domain (the Unlicense), as Annulus.
