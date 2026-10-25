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
| 1. Duplicate nodes | How many "distinct" nodes are the same name written differently? | Certain: identical after case, accents, punctuation and leading articles, and of the same type. Likely, for review: singular/plural forms and names one typo apart (edit distance), never pairing reordered words, different numbers, different types or different honorifics (`MR.` / `MRS.`). Partial names: a one-word name that is the first or last word of exactly one other same-type name (`CRUZ` → `TAYLOR CRUZ`), for review. |
| 2. Edge inflation | How many edges disappear once duplicate nodes merge? | Re-keys every edge on merged nodes; counts collapsed edges and new self-loops. |
| 3. Source support vs. chance | Is "N documents state X → Y" more than two frequent entities produce anyway? | Exact hypergeometric test conditioned on the edge's own first document; Benjamini–Hochberg across **all** edges, not only the multi-source ones. |
| 2b. Two-way edges | Is the same relationship stored as A → B and B → A? | Counted always; `--undirected` merges them (GraphRAG graphs are undirected). |
| 4. Structure | Is "no cycles" a finding or a property of how edges were built? | Exact count of nodes on directed cycles (strongly connected components); flags graphs where no node is ever both a source and a target. Degree concentration, dangling edges, unused nodes. |

## Real-world check: Microsoft GraphRAG's own example

GraphRAG ships an indexed example, *Operation Dulce* (an AI-generated novella), in its repository
(`docs/examples_notebooks/inputs/operation dulce`). Its graph has 39 entities and 107 relationships
extracted from 5 text units.

```powershell
python tools\graphrag_to_csv.py --example graphrag_example
graphaudit graphrag_example\relationships.csv --nodes graphrag_example\entities.csv --id title --label title --doc text_unit_ids --undirected
```

| Finding | Evidence |
|---|---|
| The same person is two nodes: `CRUZ` and `TAYLOR CRUZ` | The graph itself contains the edge *"Taylor Cruz is the full name of the person referred to as Cruz, both representing the same team leader"* |
| The same person is two nodes: `RIVERA` and `SAM RIVERA` | The descriptions on `RIVERA`'s edges read *"Sam Rivera is…"* |
| 5 relationships are stored in both directions | 107 rows hold 102 distinct undirected relationships |
| 2 relationships point to entities that do not exist | `ELEVATOR` and `CONCRETE HALLWAY` are missing from the entity table |
| "Mentioned in N text units" cannot be evidence here | With 5 text units, the main characters appear in nearly all of them |

So 6 PERSON nodes stand for 4 named characters. This is a small demo dataset, not a measure of GraphRAG at
scale, but these are the failure modes that grow with a graph: surnames used alone, two-way duplicates and
dangling references. graphaudit flags `DULCE` → `DULCE BASE` and `MISSION` → `MISSION TO DULCE BASE` too;
the first is a town and the base beneath it, which is why partial names are a review list, not a merge.

## Real-world check: *A Christmas Carol* (GraphRAG, 217 text units)

A GraphRAG index of Dickens's *A Christmas Carol* from Project Gutenberg: 277 entities, 342 relationships.

| Finding | Evidence |
|---|---|
| Each of the three ghosts is two nodes | `GHOST OF CHRISTMAS PAST` and `THE GHOST OF CHRISTMAS PAST`, and likewise for Present and Yet to Come; 15 such article duplicates in all (277 → 262) |
| Characters split by first name or surname | `BOB` / `BOB CRATCHIT`, `JACOB` / `JACOB MARLEY`, `MARTHA` / `MARTHA CRATCHIT`, `DICK` / `DICK WILKINS`, `JOE` / `OLD JOE` (12 partial names to review) |
| Generic nodes that mean different ghosts | `THE GHOST` (and `GHOST`) and `THE SPIRIT` attach to whichever ghost the chunk was about; the strongest "Scrooge — The Spirit" edge mixes three characters |
| The license is part of the story graph | 14 Project Gutenberg boilerplate entities and 21 relationships; 2 of the 4 edges that beat chance are license text (`PROJECT GUTENBERG — UNITED STATES`) |
| Unused entities | 23 entities have no relationship |

Read by hand, the Ghost of Christmas Yet to Come is about six nodes (`THE PHANTOM`, `GHOST OF THE FUTURE`,
`THE LAST OF THE SPIRITS`…). Those are same-meaning duplicates and need a human or embeddings; graphaudit
catches only the spelling-level ones. This book also showed why plurals are a review list and not a merge
(`THE BOYS` / `THE BOY` are different people), why `MR. FEZZIWIG` / `MRS. FEZZIWIG` must never pair, and why
`CAROLINE'S HUSBAND` is not a longer form of `CAROLINE`.

## Run

```powershell
cargo build --release
.\target\release\graphaudit.exe edges.csv
.\target\release\graphaudit.exe relationships.csv --nodes entities.csv --out audit1
```

Columns are detected by name; override with `--src`, `--dst`, `--rel`, `--doc`, `--id`, `--label`, `--type`.

| Role | Recognised column names |
|---|---|
| Edge source / target | source, subject, head, from, :START_ID / target, object, tail, to, :END_ID |
| Relation | relation, predicate, type, relationship, :TYPE |
| Provenance | doc, document, source_doc, provenance, text_unit_ids, chunk_id, sources, file, book |
| Node table | id, node_id, entity_id / label, name, title, entity / type, entity_type, category |

Comma, semicolon and tab separators are detected. Multi-valued provenance cells such as
`d1;d2`, `d1|d2` or `['t1', 't2']` (GraphRAG `text_unit_ids`) are split. Parquet output
(e.g. GraphRAG) converts with `tools/graphrag_to_csv.py` (needs `pip install pandas pyarrow`).

Output in `graphaudit_out/` (or `--out`): `report.md`, `duplicates.csv`, `likely_duplicates.csv`, `partial_names.csv`, `edge_support.csv`.
Options: `--undirected` (A → B and B → A are one edge), `--typos S` (edit budget for likely duplicates; 1 = default, 0 = off),
`--alpha A` (FDR, default 0.05), `--top N`. With fewer than 20 documents the source-support test is reported as not testable.

## Validation

`examples/` holds a synthetic graph with a known answer (`make_synthetic.py` regenerates it):
262 concepts written in 1–5 surface forms plus 15 typos, 400 documents with random edges
weighted by popularity, and 10 genuine relations planted in 12 documents each.

| Check | Result |
|---|---|
| Node count | 453 surface names → 332 certain groups → **262** with likely duplicates merged (true answer: 262) |
| Likely duplicates | 71 pairs flagged (55 singular/plural, 16 typo), all genuinely the same concept |
| Planted relations recovered | 10 of 10 |
| Random multi-source edges called significant | 0 (all 13 significant edges are planted relations, 3 of them repeated because a plural split the node) |
| Fully random graph (100k nodes, 500k edges) | 0 edges called significant; runs in ~23 s |
| February-style tension → resolution graph | 0 of 30 edges beat chance; flagged acyclic by construction |

## Limits

- Duplicates with the same meaning in different words ("car" / "automobile") need an
  embedding model; graphaudit only finds spelling-level duplicates.
- Source support assumes documents are exchangeable; near-duplicate documents (the same text
  ingested twice) inflate support and should be removed first.
- A significant edge means "stated together more often than chance", not "true".

Std-only Rust, no crates. `cargo test` runs the unit tests. Author: Ernesto Antonio Arias Díaz · MIT
