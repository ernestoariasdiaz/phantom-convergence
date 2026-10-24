# ANIMUS v5

**A knowledge graph that has to prove each connection is not chance.**

ANIMUS v1–v4 read books and reported "convergences" such as *failure → algorithm,
confirmed by 32 sources*. Two audits showed what those numbers were:

- **Phantom Nodes**: 52% of self-reported nodes were undetected duplicates
  ([preprint](https://doi.org/10.5281/zenodo.20674981)).
- **convaudit**: on the same 29 books, *failure → algorithm* co-occurs in 2 books
  where chance alone predicts 2.6. Of 639 "convergences", 4–5 survive a
  frequency-matched null, and they are idioms (*problem → solution*). The
  strongest real pair is matched by a random vocabulary.

v5 is rebuilt around those lessons:

1. **No fixed vocabulary.** The model reads each passage and states the ideas it
   asserts. Nothing decides the answer before the books are read.
2. **Every idea is checked against its own passage**, with a second, separate question.
   Rejected ideas are kept on disk and counted: the rejection rate is a result.
3. **Every idea keeps its evidence**: book, passage number, word offset, full text,
   and the raw model reply.
4. *(step 2)* Equivalent ideas from different books merge into one concept, so one
   idea is never counted as many: Phantom Nodes, applied before the fact.
5. *(step 3)* A cross-book connection enters the graph only if it beats a
   frequency-matched null, as in convaudit.

This release contains **step 1: extraction with self-verification.**

## Run

Needs a running `llama-server` (llama.cpp) with any instruction model, e.g. Gemma.

```powershell
cargo build --release
.\target\release\animus5.exe ping                    # is the server answering?
.\target\release\animus5.exe extract txts --books axelrod,kleppmann,meditaciones --per-book 30
```

Output in `animus5_out/`:

| File | Content |
|---|---|
| `extract_summary.md` | counts, rejection rate, per-book table |
| `claims.jsonl` | every idea, with `verified: true/false` |
| `passages.jsonl` | the passage text behind each idea |
| `progress.jsonl` | raw model replies, for audit; also makes runs resumable |

Runs resume: run the same command again and finished passages are skipped.
Options: `--per-book N` (0 = whole book), `--words N` (passage size, default 180),
`--server URL` (default `http://127.0.0.1:8080`), `--no-verify`, `--timeout S`.

Passages that are code, tables, indexes or OCR noise are filtered before reaching the
model, and the summary says how many and why.

Std-only Rust (own HTTP and JSON), no crates. `cargo test` runs the unit tests.

## Known limits

- A small model checking itself is not independent verification. The check catches
  additions and loose paraphrases. It cannot catch shared misreadings.
- Ideas are written in English even from Spanish passages (model translation). That is
  what makes cross-language merging possible, and it is also a source of drift.
- Sampling (`--per-book`) is for testing; any claim about a whole book needs `--per-book 0`.

Author: Ernesto Arias Díaz · MIT
