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
| 1. Duplicate nodes | How many "distinct" nodes are the same name written differently? | Certain: identical after case, accents, punctuation and leading articles, and of the same type. Likely, for review: singular/plural forms and names one typo apart (edit distance), a leading title (`KING EDWARD IV` / `EDWARD IV`) or a legal form (`APPLE` / `APPLE INC.`), never pairing reordered words, different numbers or model markers (`HENRY VI` / `HENRY VII`, `IPHONE 4` / `IPHONE 4S`, `APPLE TV` / `APPLE TV+`, `GROUP A` / `GROUP B`), whole-word swaps (`SOUTH` / `NORTH`), opposites (`MOVABLE` / `IMMOVABLE`), different types or different honorifics (`MR.` / `MRS.`). An `UNKNOWN` type matches any type. Partial names: a capitalised one-word name that is the first or last word of exactly one other same-type name, or of several that all contain the shortest (`JOBS` → `STEVE JOBS`), for review. |
| 2. Edge inflation | How many edges disappear once duplicate nodes merge? | Re-keys every edge on merged nodes; counts collapsed edges and new self-loops. |
| 3. Source support vs. chance | Is "N documents state X → Y" more than two frequent entities produce anyway? | Exact hypergeometric test conditioned on the edge's own first document; Benjamini–Hochberg across **all** edges, not only the multi-source ones. |
| 2b. Two-way edges | Is the same relationship stored as A → B and B → A? | Counted always; `--undirected` merges them (GraphRAG graphs are undirected). |
| 4. Structure | Is "no cycles" a finding or a property of how edges were built? | Exact count of nodes on directed cycles (strongly connected components); flags graphs where no node is ever both a source and a target. Connected pieces (how much of the graph a traversal can reach), degree concentration, dangling edges, unused nodes. |

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

## Real-world check: a LightRAG graph (Wars of the Roses)

A public repository ([GraphRAGssko/LightRAG](https://github.com/GraphRAGssko/LightRAG), `rag_storage/`) commits
the graph LightRAG built from three texts: the Wars of the Roses, property law and diabetes therapies.
202 entities, 135 relationships, 16 chunks.

```powershell
python tools\lightrag_to_csv.py path\to\rag_storage lightrag_csv
graphaudit lightrag_csv\relationships.csv --nodes lightrag_csv\entities.csv --id title --label title --type type --doc source_id --undirected
```

| Finding | Evidence |
|---|---|
| Edward IV is three nodes | `EDWARD IV`, `KING EDWARD IV` and `EDWARD`; the graph's own edge says *"Edward was proclaimed King Edward IV"* |
| Henry VII is two nodes | `HENRY TUDOR`'s description reads *"Henry Tudor, later known as Henry VII"*, next to a separate `HENRY VII` node |
| The same story is told twice, unconnected | Full names (`HENRY VI`, `MARGARET OF ANJOU`, `RICHARD, DUKE OF YORK`) form one piece; short names (`HENRY`, `MARGARET`, `YORK`, `DUKE OF YORK`) another, with no edge between them |
| One node merges father and son | `DUKE OF YORK` is described as York's lineage *and* as the man "proclaimed King Edward IV" |
| The graph is 47 pieces | The largest holds 28 of 177 linked nodes; a traversal from any node reaches 16% of the graph at most. 25 entities have no edge at all |
| Source counts carry no evidence | 16 chunks; only 5 edges come from more than one |

Three texts on unrelated subjects explain three pieces, not 47. graphaudit finds the spelling-level duplicates
(`KING EDWARD IV`, `MARGARET`, `WARWICK`); `HENRY TUDOR` / `HENRY VII` and `EDWARD` / `EDWARD IV` need a human or embeddings.
This graph also showed why opposites (`MOVABLE PROPERTY` / `IMMOVABLE PROPERTY`, one edit apart) and Roman numerals
must never pair, and why `UNKNOWN` must not block a match with a typed node.

## Real-world check: kg-gen (Stanford), Apple Inc. article

[kg-gen](https://github.com/stair-lab/kg-gen) ships example graphs it extracted from Wikipedia articles
(`app/examples/*.json`). The largest, *Apple Inc.*, has 1,188 entities and 1,386 relationships from 50 chunks.

| Finding | Evidence |
|---|---|
| The company is split | `Apple` holds 456 relationship endpoints, `Apple Inc.` 47, plus `Apple Computer, Inc.`, `Apple Computer Company` and `Apple Computer` |
| Most of what the graph knows about Jobs is on his surname | `Jobs` 46 endpoints, `Steve Jobs` 19, `Apple co-founder Steve Jobs` 2; likewise `Wozniak` 16 / `Steve Wozniak` 4 |
| Surnames used alone | `Amelio`, `Gassée`, `Ive`, `Raskin`, `Sculley`, `Spindler`, `Elphicke` each sit apart from the full name |
| Relationships from a node to itself | 7, such as *iTunes — is — iTunes* and *Apple II — has — Apple II* |
| 65 disconnected pieces | The largest holds 83% of linked nodes; 66 entities have no relationship |

graphaudit lists 7 certain duplicates (case only), 31 likely pairs (23 singular/plural, 8 legal forms) and
41 partial names. The partial-name list is noisier in a graph without entity types: by hand, about half are the
same thing (`JOBS` → `STEVE JOBS`) and half are not (`AMAZON` → `AMAZON ECHO`). kg-gen records one chunk per
relationship, so the source-support test does not apply.

This graph exposed four more tool errors, now fixed: `APPLE TV` / `APPLE TV+` and `ICLOUD` / `ICLOUD+` were merged
as certain duplicates (punctuation was stripped), `IPHONE 4` / `IPHONE 4S` and `GROUP A` / `GROUP B` were proposed
as typos, `SOUTH AMERICAN` / `NORTH AMERICAN` too, and bare years and common nouns (`2020`, `ads`) were proposed as
partial names.

## Control: a hand-built Neo4j graph

Neo4j's *Movies* example ([neo4j-graph-examples/movies](https://github.com/neo4j-graph-examples/movies),
`scripts/movies.cypher`) is curated by hand: 171 nodes, 253 relationships. graphaudit reports 0 duplicates,
0 likely pairs, 0 partial names, 0 unused or dangling nodes, and one connected piece. The findings above come
from LLM extraction, not from the checks themselves.

## Same meaning in different words (experimental, `--embed`)

Spelling-level checks cannot see that `HENRY TUDOR` is `HENRY VII`. With an embedding server running,
graphaudit embeds every distinct name and lists pairs above a cosine threshold that no other check already
proposed. It works with any OpenAI-compatible `/v1/embeddings` endpoint, for example llama.cpp with bge-m3:

```powershell
llama-server -m bge-m3-Q8_0.gguf --embeddings --port 8081
graphaudit relationships.csv --nodes entities.csv --embed http://127.0.0.1:8081 --out audit_embed
graphaudit relationships.csv --nodes entities.csv --embed http://127.0.0.1:8081 --desc description --out audit_embed_desc
```

`--desc COL` embeds each name with its description (the converters export one for GraphRAG and LightRAG).
The same guards apply: `HENRY VI` / `HENRY VII`, `MR.` / `MRS.` and opposites never pair, and typed nodes pair
only with the same or an `UNKNOWN` type.

**Result on the Wars of the Roses graph with bge-m3 (Q8_0): embeddings do not separate "the same entity" from
"a related entity".**

| Input | Pairs at cosine ≥ 0.80 | Genuine duplicates among them | Known duplicates missed |
|---|---|---|---|
| Names only | 25 | 0 (`BURGLARY` / `ROBBERY`, `BEEF` / `PORK INSULIN`, `BATTLE OF ST. ALBANS` / `SECOND BATTLE…`) | `EDWARD` / `EDWARD IV`, `HENRY TUDOR` / `HENRY VII` |
| Names + descriptions | 37 | `EDWARD` / `EDWARD IV` (0.823), and arguably `CIVIL LAW` / `CIVIL-LAW SYSTEM` and `ANGLO-AMERICAN LAW` / `…SYSTEM` | `HENRY TUDOR` / `HENRY VII`, although its description says *"later known as Henry VII"* |

The one clear duplicate ranks below `GOLD` / `SILVER` (0.886), `HOUSE OF LANCASTER` / `HOUSE OF YORK` and
`DEBTOR` / `CREDITOR`, so no threshold keeps it and drops those. At the default 0.90 both runs return nothing.
`semantic_duplicates.csv` is a list for a human to skim, never a merge; finding same-meaning duplicates reliably
needs more than cosine similarity (for example, reading the descriptions with an LLM and asking).

## Same entity, judged by a language model (experimental, `--judge`)

Embeddings measure topic, not identity. `--judge` instead asks a local instruction model, for each candidate
pair, whether the two nodes are one and the same entity, showing it both names, types and descriptions.
Each pair is asked twice, A–B and B–A, and counts only if both answers are SAME; related entities (spouses,
predecessor and successor, a part and its whole, two battles in a series) are defined as DIFFERENT in the prompt.

Candidates are every pair the other checks propose, plus names of compatible type that share a rare word
(`HENRY TUDOR` / `HENRY VII`), plus each name's three nearest embedding neighbours when `--embed` is also given.
On the Wars of the Roses graph that is 110 pairs, and they include every duplicate found by hand.

```powershell
llama-server -m gemma-4-E2B-it-UD-Q4_K_XL.gguf -ngl 99 -c 4096 --port 8080 --reasoning-budget 0
graphaudit relationships.csv --nodes entities.csv --desc description --judge http://127.0.0.1:8080 --out audit_judge
```

**Result on the Wars of the Roses graph with Gemma 4 E2B (Q4, 2B parameters):**

| | Pairs | Verdicts |
|---|---|---|
| Known person duplicates | 7 | 3 same (`EDWARD IV` / `KING EDWARD IV`, `EDWARD` / `KING EDWARD IV`, `WARWICK` / `RICHARD NEVILLE, EARL OF WARWICK`), 3 uncertain (`EDWARD` / `EDWARD IV`, `MARGARET` / `MARGARET OF ANJOU`, `HENRY TUDOR` / `HENRY VII`), 1 different (`HENRY` / `HENRY VI`: "a generic name") |
| All candidates | 110 | 11 same: 9 right (the 3 above, 4 singular/plural pairs, `DIABETES` / `DIABETES MELLITUS`, `RICHARD, DUKE OF YORK` / `DUKE OF YORK`), 1 arguable (`ANGLO-AMERICAN LAW` / `ANGLO-AMERICAN SYSTEMS`), 1 wrong (`HOUSE OF LANCASTER` / `HOUSE OF YORK`: "rival factions"); 10 uncertain; 89 different, including `HENRY VI` / `HENRY TUDOR`, every pair of battles, insulin types and tenancies |

Compared with embeddings on the same graph (1 duplicate found, ranked below gold / silver), a small model reading
descriptions separates "same" from "related" far better, but misses half the duplicates. `HENRY TUDOR` / `HENRY VII`
failed because the graph's own description of `HENRY VII` is a sentence about someone marrying him; the model read
it as a different person. A larger model may do better; that is untested.

Output: `judged_pairs.csv` with every candidate, the verdict (same / different / uncertain when the two orders
disagree) and the model's reason. A model can be wrong; the list is for review before merging.

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
(e.g. GraphRAG) converts with `tools/graphrag_to_csv.py` (needs `pip install pandas pyarrow`); LightRAG's
`graph_chunk_entity_relation.graphml` with `tools/lightrag_to_csv.py`, kg-gen JSON with `tools/kggen_to_csv.py` (both standard library only).

Output in `graphaudit_out/` (or `--out`): `report.md`, `duplicates.csv`, `likely_duplicates.csv`, `partial_names.csv`, `edge_support.csv`, and `semantic_duplicates.csv` with `--embed`.
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

- Duplicates with the same meaning in different words ("car" / "automobile") are not found reliably: `--embed`
  ranks related things (gold / silver) above true duplicates on a real graph.
- Source support assumes documents are exchangeable; near-duplicate documents (the same text
  ingested twice) inflate support and should be removed first.
- A significant edge means "stated together more often than chance", not "true".

Std-only Rust, no crates (the embedding client is plain HTTP to a local server). `cargo test` runs the unit tests. Author: Ernesto Antonio Arias Díaz · MIT
