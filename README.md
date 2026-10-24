# Phantom Convergence

Tools for auditing convergence claims in autonomously built knowledge graphs.

- convaudit/ - tests "N sources confirm X -> Y" counts against chance (hypergeometric + Poisson-binomial, BH correction, random vocabularies)
- animus5/ - ANIMUS v5: quote-grounded idea extraction with a local LLM, embedding positive control, cross-book links vs. word and form decoys

Std-only Rust, no crates. Book texts are not included (copyright); see convaudit/tools to rebuild the corpus from your own copies.

Related preprint: Phantom Nodes, https://doi.org/10.5281/zenodo.20674981

Author: Ernesto Arias Diaz - MIT License
