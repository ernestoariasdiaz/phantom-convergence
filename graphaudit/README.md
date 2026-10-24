# graphaudit

**Check a knowledge graph's own numbers before believing them.**

Graphs built by LLM pipelines (GraphRAG, LightRAG, agent memories, autonomous crawlers)
report node counts, edge counts and "confirmed by N sources" scores. graphaudit asks of each
number what it would be if nothing interesting were happening. It grew out of auditing
ANIMUS, where 52% of nodes were duplicates and the flagship "32 sources" edge was at chance level
([Phantom Nodes](https://doi.org/10.5281/zenodo.20674981),
[Phantom Convergence](https://doi.org/10.5281/zenodo.23622420)).

| Audit | Question | How |
|---|---|---|
| 1. Duplicate nodes | How many "distinct" nodes are the same name written differently? | Certain: identical after case, accents, punctuation, leading articles and plurals. Likely: one typo apart (edit distance), never pairing reordered words or different numbers. |
| 2. Edge inflation | How many edges disappear once duplicate nodes merge? | Re-keys every edge on merged nodes; counts collapsed edges and new self-loops. |
| 3. Source support vs. chance | Is "N documents state X → Y" more than two frequent entities produce anyway? | Exact hypergeometric test conditioned on the edge's own first document; Benjamini–Hochberg across **all** edges, not only the multi-source ones. |
| 4. Structure | Is "no cycles" a finding or a property of how edges were built? | Exact count of nodes on directed cycles (strongly connected components); flags graphs where no node is ever both a source and a target. Degree concentration, dangling edges, unused nodes. |

## Run

```powershell
cargo build --release
.\target\release\graphaudit.exe edges.csv
.\target\release\graphaudit.exe relationships.csv --nodes entities.csv --out audit1
```

Columns are detected by name; override with `--src`, `--dst`, `--rel`, `--doc`, `--id`, `--label`.

| Role | Recognised column names |
|---|---|
| Edge source / target | source, subject, head, from, :START_ID / target, object, tail, to, :END_ID |
| Relation | relation, predicate, type, relationship, :TYPE |
| Provenance | doc, document, source_doc, provenance, text_unit_ids, chunk_id, sources, file, book |
| Node table | id, node_id, entity_id / label, name, title, entity |

Comma, semicolon and tab separators are detected. Multi-valued provenance cells such as
`d1;d2`, `d1|d2` or `['t1', 't2']` (GraphRAG `text_unit_ids`) are split. Parquet output
(e.g. GraphRAG) converts with one line: `python -c "import pandas as p; p.read_parquet('relationships.parquet').to_csv('relationships.csv', index=False)"`.

Output in `graphaudit_out/` (or `--out`): `report.md`, `duplicates.csv`, `likely_duplicates.csv`, `edge_support.csv`.
Options: `--typos S` (edit budget for likely duplicates; 1 = default, 0 = off), `--alpha A` (FDR, default 0.05), `--top N`.

## Validation

`examples/` holds a synthetic graph with a known answer (`make_synthetic.py` regenerates it):
262 concepts written in 1–5 surface forms plus 15 typos, 400 documents with random edges
weighted by popularity, and 10 genuine relations planted in 12 documents each.

| Check | Result |
|---|---|
| Node count | 453 surface names → 277 certain groups → **262** with likely duplicates merged (true answer: 262) |
| Likely duplicates | 15 pairs flagged, all genuinely the same concept; 11 of 12 present typos found |
| Planted relations recovered | 10 of 10 |
| Random multi-source edges called significant | 0 of 134 (the one extra hit is a planted relation whose node was split by a malformed plural) |
| Fully random graph (100k nodes, 500k edges) | 0 edges called significant; runs in ~23 s |
| February-style tension → resolution graph | 0 of 30 edges beat chance; flagged acyclic by construction |

## Limits

- Duplicates with the same meaning in different words ("car" / "automobile") need an
  embedding model; graphaudit only finds spelling-level duplicates.
- Source support assumes documents are exchangeable; near-duplicate documents (the same text
  ingested twice) inflate support and should be removed first.
- A significant edge means "stated together more often than chance", not "true".

Std-only Rust, no crates. `cargo test` runs the unit tests. Author: Ernesto Antonio Arias Díaz · MIT
