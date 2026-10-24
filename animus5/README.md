# ANIMUS v5

**A knowledge graph that has to prove each connection is not chance.**

ANIMUS v1–v4 read books and reported "convergences" such as *failure → algorithm,
confirmed by 32 sources*. Two audits showed what those numbers were:

- **Phantom Nodes**: 52% of self-reported nodes were undetected duplicates
  ([preprint](https://doi.org/10.5281/zenodo.20674981)).
- **convaudit**: on the same 29 books, *failure → algorithm* co-occurs in 2 books
  where chance alone predicts 2.6. Of 639 co-occurring pairs, 4–5 survive a chance
  model with FDR control: fixed expressions (*problem → solution*) and one topical
  pair. The strongest real pair is matched by a random vocabulary.

v5 is rebuilt so that every number it reports has a null computed by the same code:

1. **No fixed vocabulary.** The model reads each passage and states the general ideas
   it asserts. Nothing decides the answer before the books are read.
2. **Every idea is proven by an exact quote.** The model must copy the sentence that
   states each idea; an idea is kept only if ≥ 80% of the quote's word trigrams are in
   the passage. (An earlier version asked the model "does the passage say this?"; it
   approved 267 of 269 ideas.) Rejected ideas are kept on disk and counted.
3. **Every idea keeps its evidence**: book, passage number, word offset, full text,
   quote, and the raw model reply.
4. **A positive control runs before any link is read**: each idea must pick its own
   source quote out of 20 (chance 5%), Spanish quotes against English ideas included.
5. **A cross-book link counts only against decoys** run through the identical procedure:
   word decoys (all words shuffled) and form decoys (content words shuffled, function
   words such as *not / no / there / is* kept in place).

## Result on 29 books

10,392 quote-verified ideas, 52 million cross-book pairs, positive control 97%.
Real ideas produce **fewer** links than decoys (18,258 vs. 20,050 word decoys,
19,709 form decoys). The most striking theme, statements about the limits of knowledge,
was mostly negation syntax: both-negated pairs were 42% of the top 100 links under word
decoys and 11% under form decoys. Full write-up: [*Phantom Convergence*](https://doi.org/10.5281/zenodo.23622420) (preprint, Zenodo).

## Run

Needs `llama-server` (llama.cpp): an instruction model for step 1, an embedding model for step 2.

```powershell
cargo build --release

# step 1 — extract (e.g. Gemma 4 E2B on port 8080, thinking off)
llama-server -m gemma-4-E2B-it-UD-Q4_K_XL.gguf -ngl 99 -c 4096 --port 8080 --reasoning-budget 0
.\target\release\animus5.exe ping
.\target\release\animus5.exe extract txts --per-book 150 --out run1

# step 2 — link (bge-m3 on port 8081)
llama-server -m bge-m3-Q8_0.gguf --embeddings --port 8081 -ngl 99
.\target\release\animus5.exe link --out run1
.\target\release\animus5.exe link --out run1 --decoy-mode content
```

`txts` = one `.txt` per book (see `../convaudit/tools` to build it from PDFs).

| File | Content |
|---|---|
| `extract_summary.md` | counts, ideas per passage, rejection reasons, per-book table |
| `claims.jsonl` | every idea with its quote, grounding score, kept or rejected |
| `passages.jsonl` | the passage text behind each idea |
| `progress.jsonl` | raw model replies, for audit; makes runs resumable |
| `links_report.md`, `links_report_content.md` | positive control, links vs. decoys, negation check, top links with quotes |
| `embeddings.bin` | binary embedding cache; reruns load in seconds |

Extract options: `--per-book N` (0 = whole book), `--books a,b` (filter by file name),
`--words N` (passage size, default 180), `--server URL`, `--timeout S`.
Link options: `--decoy-mode words|content`, `--decoys K`, `--percentile P`, `--top N`, `--server URL`.

Passages that are code, tables, indexes, bibliographies or OCR noise are filtered before
reaching the model, and the summary says how many and why.

Std-only Rust (own HTTP and JSON), no crates. `cargo test` runs the unit tests.

## Known limits

- Quote grounding proves an idea is anchored to a real sentence, not that it follows from it.
- A 2B model fills the three-idea maximum for most passages.
- Ideas are written in English even from Spanish passages (model translation).
- bge-m3 is only moderately sensitive to word order (idea vs. its decoy: 0.89–0.91), so the
  decoy test rules out a large excess of cross-book links, not a small one.
- Sampling (`--per-book`) covers part of each book; claims about whole books need `--per-book 0`.

Author: Ernesto Arias Díaz · MIT
