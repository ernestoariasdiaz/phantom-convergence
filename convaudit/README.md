# convaudit

**Audit "N independent sources confirm X → Y" claims against chance.**

Autonomous knowledge-graph builders often report convergence: *"failure → algorithm is
confirmed by 32 independent sources."* convaudit asks the question that number skips:
**how many sources would "confirm" that pair if the two words were scattered at random
inside each source?** If the answer is close to 32, the convergence is a word-frequency
artifact, not a finding.

It is the second audit in a line of work that started with
[Phantom Nodes](https://doi.org/10.5281/zenodo.20674981) (52% of self-reported nodes in an
autonomously built graph were undetected duplicates). Phantom Nodes audits *how many things
a system says it knows*; convaudit audits *how strongly it says its sources agree*.

## What it measures

For every tension → resolution word pair:

| Column | Meaning |
|---|---|
| Sources confirming | Sources where both words share at least one window (the usual "N sources confirm" metric) |
| Expected by chance | Same count if one word's windows were placed at random within each source — exact hypergeometric, summed over sources |
| Window lift | Observed shared windows ÷ expected shared windows |
| p | Exact Poisson-binomial upper tail of the source count |
| q | Benjamini–Hochberg FDR across all pairs |

It also builds **random vocabularies** in which every word is replaced by a corpus word of
similar source frequency, and reports how many "convergences" those meaningless vocabularies
produce. And it checks whether the graph *could* contain a feedback loop at all: if the
tension and resolution lists are disjoint, every edge runs one way between two disjoint sets,
so the graph is acyclic by construction.

## Example: Vim's documentation (151 files)

Using the reconstructed ANIMUS (Feb 2026) tension/resolution vocabulary on the Vim manual:

| Metric | Real vocabulary | 50 random vocabularies |
|---|---|---|
| Strongest pair, sources "confirming" | 20 (error → fixed) | 15.3 on average, up to **32** |
| Pairs "confirmed" by ≥10 sources | 5 | 1.6 |
| Pairs that survive FDR | 1 | 0.3 |

`error → fixed` is "confirmed" by 20 Vim help files — chance alone predicts 16.3 (p = 0.08,
not significant). The one pair that survives, `problems → fixed` (lift 1.9), is real: Vim's
version notes are written as *Problem: … Solution: …*. A random vocabulary reached 32 sources
in a text editor manual.

**Positive control** (`examples/positive_control_vocab.txt`): the tool does detect genuine
association — `python → interpreter` (lift 2.4), `perl → interpreter` (3.5),
`netbeans → socket` (2.7) all survive FDR, while `python → highlighting`, which co-occurs in
9 sources, is flagged as a frequency artifact (10.3 expected).

Full reports: `examples/vimdocs_report.md`, `examples/positive_control_report.md`.

## Build and run

Std-only Rust, no crates.

```bash
cargo build --release
cargo test
mkdir -p corpora/vimdocs && cp /usr/share/vim/vim9*/doc/*.txt corpora/vimdocs/   # any Vim install
./target/release/convaudit corpora/vimdocs                       # default ANIMUS vocabulary
./target/release/convaudit my_txts --vocab my_vocab.txt --out report.md
```

Vocabulary file format:

```
tension: failure, crisis, fracaso, colapso
resolution: algorithm, regulation, algoritmo, regulación
```

Options: `--window N` (tokens per window, default 300), `--nulls K` (random vocabularies,
default 50), `--alpha A` (FDR level, default 0.05), `--top N`, `--seed S`, `--out FILE`.

PDFs → text: `python tools/pdf2txt.py pdfs/ txts/` (needs `pypdf`; scanned PDFs need OCR first).

## Limitations

- Windows are fixed-size token chunks, not sentences or pages.
- Matching is exact word forms (no stemming or lemmatisation); list variants explicitly.
- "Expected by chance" holds each word's frequency per source fixed, which is the right null
  for "do these two words attract each other", not for "is this pair true".
- A significant association is co-occurrence, not causation and not direction.

## Author

Ernesto Arias Díaz — License: MIT
