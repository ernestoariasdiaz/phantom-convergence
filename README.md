# Phantom Convergence

**Tools for auditing the numbers that autonomously built knowledge graphs report about themselves.**

> **Have a graph of your own?** Export its edges to CSV and run [`graphaudit`](graphaudit): it reports how many nodes are duplicates, how many edges collapse, and which "confirmed by N sources" edges beat chance.

These are the tools behind the preprint [*Phantom Convergence: Auditing Cross-Domain Claims in an
Autonomously Built Knowledge Graph*](https://doi.org/10.5281/zenodo.23622420) (Arias Díaz, 2026,
doi:10.5281/zenodo.23622420). The code is archived on Zenodo:

| Version | What it adds | DOI |
|---|---|---|
| 1.0.0 | The code exactly as used in the paper | [10.5281/zenodo.23622368](https://doi.org/10.5281/zenodo.23622368) |
| 1.1.0 | `graphaudit` | [10.5281/zenodo.23623444](https://doi.org/10.5281/zenodo.23623444) |
| 1.3.0 | Duplicate-detection fixes found on *A Christmas Carol* | [10.5281/zenodo.23652614](https://doi.org/10.5281/zenodo.23652614) |
| 1.4.0 | LightRAG support | [10.5281/zenodo.23652737](https://doi.org/10.5281/zenodo.23652737) |
| 1.5.0 | kg-gen support, experimental embedding check | [10.5281/zenodo.23652957](https://doi.org/10.5281/zenodo.23652957) |
| 1.6.0 | Experimental language-model judge for same-entity pairs | [10.5281/zenodo.23653176](https://doi.org/10.5281/zenodo.23653176) |
| 1.7.0 | Judge limited to named entities | [10.5281/zenodo.23653413](https://doi.org/10.5281/zenodo.23653413) |
| 1.8.0 | Self-contradicting generic nodes flagged | [10.5281/zenodo.23653711](https://doi.org/10.5281/zenodo.23653711) |
| 1.9.0 | Fixes from a blind test on a 7,945-entity graph | [10.5281/zenodo.23654666](https://doi.org/10.5281/zenodo.23654666) |
| 1.10.0 | Edges with no provenance, ai-knowledge-graph support | [10.5281/zenodo.23654796](https://doi.org/10.5281/zenodo.23654796) |
| 1.11.0 | Self-loops as merge evidence, itext2kg support | [10.5281/zenodo.23654876](https://doi.org/10.5281/zenodo.23654876) |
| 1.12.0 | Short-word and title guards, entity comparison between graphs | [10.5281/zenodo.23654967](https://doi.org/10.5281/zenodo.23654967) |

They audit ANIMUS, an autonomous
knowledge-graph system. Each audit removed the effect the previous version had reported:

| Claim | What the audit found |
|---|---|
| ~1,892 nodes in the graph | 52% were undetected duplicates: 911 unique ([Phantom Nodes](https://doi.org/10.5281/zenodo.20674981)) |
| *failure → algorithm* "confirmed by 32 sources" | Co-occurs in 2 of 29 books; chance alone predicts 2.6 (`convaudit`) |
| Convergence of ideas across 29 books | 10,392 quote-verified ideas link **less** often than decoys: 18,258 vs. 20,050 (`animus5`) |
| A "limits of knowledge" theme across Kuhn, Meadows, Isaiah, GEB… | Mostly negation syntax: 42% of the top 100 links were both-negated, 11% once decoys keep the grammar |

## What's here

| Folder | Tool | What it does |
|---|---|---|
| [`convaudit/`](convaudit) | `convaudit` | Tests co-occurrence convergence counts against chance: exact hypergeometric expectation per source, Poisson-binomial test, Benjamini–Hochberg correction, frequency-matched random vocabularies. |
| [`animus5/`](animus5) | `animus5 extract` | Idea extraction with a local LLM (llama.cpp); an idea is kept only if its verbatim quote is found in the passage. |
| | `animus5 link` | Multilingual embeddings (bge-m3), a positive control, cross-book links, and the same procedure run on word decoys and form-preserving decoys. |
| [`graphaudit/`](graphaudit) | `graphaudit` | **Audit any knowledge graph from a CSV export**: duplicate nodes, edge inflation, "N sources confirm X → Y" against chance, and cycles that are impossible by construction. Reads GraphRAG, Neo4j and plain triple exports. |

All three are standard-library-only Rust (no crates). `graphaudit` needs no GPU; `animus5` runs on a laptop GPU (the paper's results were produced on an RTX 3050, 4 GB).

```powershell
cd convaudit; cargo test; cargo build --release
cd ..\animus5; cargo test; cargo build --release
cd ..\graphaudit; cargo test; cargo build --release
.\target\release\graphaudit.exe examples\synthetic_edges.csv
```

Usage and options: see each folder's README.

## Reusable recipe

1. **Ground extraction mechanically.** A small model asked to verify its own output approved 267 of 269 ideas; a verbatim-quote check cannot be talked into a yes.
2. **Run a positive control before reading results.** Here, each idea picks its own source quote out of 20 (97%; chance 5%).
3. **Run the whole procedure on decoys that keep the suspected confound.** Word decoys exposed vocabulary; only form decoys exposed negation.

## Data

Book texts are not included (copyright). `convaudit/tools/seleccionar_libros.ps1` and `pdf2txt.py` rebuild the corpus from your own copies.

## Cite

Arias Díaz, E. A. (2026). *Phantom Convergence: Auditing Cross-Domain Claims in an Autonomously Built Knowledge Graph* (Version 1.0). Zenodo. https://doi.org/10.5281/zenodo.23622420

## Author

Ernesto Antonio Arias Díaz · MIT License
