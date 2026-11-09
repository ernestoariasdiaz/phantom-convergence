# Data

Outputs behind the preprint *Phantom Convergence* ([doi:10.5281/zenodo.23622420](https://doi.org/10.5281/zenodo.23622420)). Licensed CC BY 4.0. How to regenerate each file is in [`../REPRODUCE.md`](../REPRODUCE.md).

## `convaudit_29/`: Study 1, 29 books

`report_w300.md` and `report_w60.md`: co-occurrence counts against chance, with windows of 300 and 60 tokens. They contain only book file names and counts.

## `animus5_29/`: 29 books, Gemma 4 E2B extraction, bge-m3 links

| File | Rows | Columns |
|---|---|---|
| `claims.csv` | 10,897 ideas (10,392 kept) | `id` (book#passage#n), `book`, `passage`, `claim` (written by the model), `quote_words`, `quote_sha256_16`, `grounding` (share of the quote's word trigrams found in the passage), `verified`, `reject` |
| `links_words.csv` | 18,258 links | `a`, `b` (idea ids), `sim` (cosine), `lift` (sim minus the mean sim of word-shuffled decoys), `claim_a`, `claim_b` |
| `links_form.csv` | 18,258 links | same columns, lift against form-preserving decoys |
| `extract_summary.md` | | counts, filters, rejection reasons, per-book table |
| `links_report_words.md`, `links_report_form.md` | | positive control, links against decoys, negation check, top links |

The quotes from the books are removed (copyright). `quote_sha256_16` is the first 16 hex characters of the SHA-256 of the quote, lowercased with whitespace collapsed, so quotes from a re-run on your own copies can be matched to these rows.
