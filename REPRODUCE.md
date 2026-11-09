# Reproducing *Phantom Convergence*

This file maps each result in the preprint ([doi:10.5281/zenodo.23622420](https://doi.org/10.5281/zenodo.23622420)) to the command that produces it and to the published output in `data/`.

## What is and is not included

| Included | Not included, and why |
|---|---|
| All code (`convaudit/`, `animus5/`, `graphaudit/`) | The 29 books: they are copyrighted |
| The model-written ideas, scores and links (`data/animus5_29/`) | `passages.jsonl`, `progress.jsonl` and the quotes: they contain book text |
| The reports behind every table, with the quotes removed | The model files: download them from the sources below |
| Scripts to rebuild the corpus from your own copies (`convaudit/tools/`) | |

Each idea in `data/animus5_29/claims.csv` carries `quote_sha256_16`: the first 16 hex characters of the SHA-256 of its quote, lowercased with whitespace collapsed. If you re-run the extraction on your own copies of the books, you can match your quotes to the published ideas with that hash.

## Environment used

| Component | Version |
|---|---|
| OS | Windows, x86-64 |
| GPU | NVIDIA RTX 3050, 4 GB VRAM |
| Rust | rustc 1.97.0 (2d8144b78 2026-07-07), cargo 1.97.0 |
| llama.cpp | build b9603 (ba1df050f), `llama-b9603-bin-win-cuda-13.3-x64` |
| Extraction model | Gemma 4 E2B instruct, GGUF `UD-Q4_K_XL`, temperature 0, thinking off |
| Embedding model | bge-m3, GGUF `Q8_0` |

The Rust code uses only the standard library (no crates), so `cargo build` needs no network access. Any recent stable Rust should work.

## 1. Build the corpus

The preprint uses 29 books in English and Spanish (the list is in `data/animus5_29/extract_summary.md`). From your own PDF copies:

```powershell
cd convaudit
python tools/pdf2txt.py pdfs/ txts/     # needs pypdf; scanned PDFs need OCR first
```

One `.txt` per book. File names become book identifiers, so rename them to match the published ones (for example `Kuhn-SSR-2ndEd.txt`) if you want to compare row by row.

## 2. Study 1: convaudit (co-occurrence against chance)

```powershell
cd convaudit
cargo test; cargo build --release
.\target\release\convaudit.exe ..\txts --top 700 --out report_w300.md
.\target\release\convaudit.exe ..\txts --window 60 --top 700 --out report_w60.md
```

Defaults: built-in reconstructed ANIMUS vocabulary (50 tension × 39 resolution words), window 300 tokens, 50 random vocabularies, FDR α = 0.05. `--top 700` lists every observed pair instead of the top 15.

| Result in the preprint | Published file |
|---|---|
| Study 1 table, w = 300: strongest pair 21 sources (random 14.1, max 21), 12 pairs with ≥ 10 sources (7.3), 4 survive FDR (0.3) | `data/convaudit_29/report_w300.md`, section 1 |
| Study 1 table, w = 60: 18 (9.1, max 18), 5 (1.1), 5 (0.3) | `data/convaudit_29/report_w60.md`, section 1 |
| 639 pairs observed at w = 300 | `report_w300.md`, header |
| Surviving pairs: problem(s) → solution(s), problema → solución, problems → algorithms, poverty → education | section 3 of each report |
| The tension → resolution graph is acyclic by construction | section 4 of each report |
| *failure → algorithm* in 2 books, 2.6 expected, p = 0.86 | `report_w60.md`, its row in section 2. This figure is for w = 60; at w = 300 the same pair is 3 books against 4.7 expected (p = 0.98), also at or below chance |

## 3. Study 2, step 1: animus5 extract

```powershell
llama-server -m gemma-4-E2B-it-UD-Q4_K_XL.gguf -ngl 99 -c 4096 --port 8080 --reasoning-budget 0
cd animus5
cargo test; cargo build --release
.\target\release\animus5.exe ping
.\target\release\animus5.exe extract ..\txts --per-book 150 --out animus5_29
```

Passage sampling is deterministic: 150 evenly spaced passages per book after filters. Generation runs at temperature 0, but GPU kernels can still change an occasional token, so expect small differences in individual ideas.

| Result in the preprint | Published file |
|---|---|
| 10,897 ideas proposed, 10,392 kept (5% rejected) | `data/animus5_29/extract_summary.md`, `claims.csv` (`verified`) |
| Rejection reasons: quote not in passage 444, too short 61 | `extract_summary.md`, `claims.csv` (`reject`) |

## 4. Study 2, step 2: animus5 link

```powershell
llama-server -m bge-m3-Q8_0.gguf --embeddings --port 8081 -ngl 99
.\target\release\animus5.exe link --out animus5_29                       # word decoys
.\target\release\animus5.exe link --out animus5_29 --decoy-mode content  # form decoys
```

Defaults: τ at the 99th percentile, 3 decoy runs, seed 20261023.

| Result in the preprint | Published file |
|---|---|
| Positive control: 97% own-quote retrieval (chance 5%) | `links_report_words.md`, section 1 |
| 52,067,685 cross-book pairs, τ = 0.610 | `links_report_words.md`, section 2 |
| 18,258 real links against 20,050 word decoys (19,849–20,236) | `links_report_words.md`, `links_words.csv` |
| 18,258 real links against 19,709 form decoys (19,674–19,770) | `links_report_form.md`, `links_form.csv` |
| Both sides negated in the top 100: 42% (word decoys), 11% (form decoys) | the "Negation check" section of each report |

## 5. graphaudit

`graphaudit` came after the preprint and is not needed for its results. Its tests and examples run with:

```powershell
cd graphaudit
cargo test; cargo build --release
.\target\release\graphaudit.exe examples\synthetic_edges.csv
```

## Licences

- Code: MIT (`LICENSE`).
- Data in `data/` and documentation: CC BY 4.0.
- Book texts are not distributed. Their copyright belongs to their authors and publishers.
