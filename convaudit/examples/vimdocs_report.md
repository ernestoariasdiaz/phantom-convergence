# convaudit report

- Corpus: `corpora/vimdocs` — 151 sources, 5340 windows of 300 tokens
- Vocabulary: 50 tension × 39 resolution words → 57 pairs observed at least once
- Null baseline: 50 random vocabularies, each word matched to a corpus word of similar source frequency

## 1. Does the vocabulary beat random words?

| Metric | Real vocabulary | Random vocabularies (mean) |
|---|---|---|
| Strongest pair: # sources 'confirming' | 20 | 15.3 (range 7–32) |
| Pairs 'confirmed' by ≥10 sources | 5 | 1.6 |
| Pairs significant after FDR (q<0.05) | 1 | 0.3 |

Example from a random vocabulary: **exists → replacement** is 'confirmed' by 7 sources.

## 2. Top pairs: observed vs. expected by chance

`expected` = sources that would 'confirm' the pair if both words were scattered at random inside each source (exact hypergeometric). `p` = exact Poisson-binomial tail; `q` = Benjamini–Hochberg.

| Pair | Sources confirming | Expected by chance | Window lift | p | q | Verdict |
|---|---|---|---|---|---|---|
| error → fixed | 20 | 16.3 | 1.27 | 0.0826 | 0.5700 | not significant |
| problems → fixed | 18 | 10.5 | 1.90 | 8.8e-5 | 0.0050 | association beyond chance |
| problem → fixed | 15 | 11.4 | 0.95 | 0.0325 | 0.4329 | not significant |
| error → solution | 13 | 11.6 | 1.02 | 0.3014 | 0.7788 | frequency artifact |
| problem → solution | 11 | 8.8 | 1.09 | 0.0793 | 0.5700 | not significant |
| errors → fixed | 9 | 9.2 | 1.43 | 0.6741 | 0.9388 | frequency artifact |
| failure → solution | 8 | 5.5 | 1.04 | 0.0225 | 0.4268 | not significant |
| errors → solution | 8 | 8.4 | 1.00 | 0.7528 | 0.9388 | frequency artifact |
| problems → solution | 8 | 8.3 | 1.00 | 0.7666 | 0.9388 | frequency artifact |
| error → algorithm | 7 | 5.2 | 1.43 | 0.2113 | 0.7788 | not significant |
| error → reduced | 6 | 4.5 | 1.14 | 0.2261 | 0.7788 | not significant |
| gap → solution | 5 | 3.6 | 1.27 | 0.1427 | 0.5809 | not significant |
| failure → fixed | 5 | 4.0 | 1.16 | 0.3302 | 0.7788 | not significant |
| corruption → solution | 4 | 3.2 | 1.20 | 0.3269 | 0.7788 | not significant |
| problem → framework | 4 | 4.2 | 0.95 | 0.8564 | 0.9388 | frequency artifact |

## 3. Pairs that survive the test

| Pair | Sources | Expected | Window lift | q |
|---|---|---|---|---|
| problems → fixed | 18 | 10.5 | 1.90 | 0.0050 |

## 4. Can this graph ever contain a feedback loop?

No. Every edge goes from a tension word to a resolution word, and the two lists share no word. A graph whose edges only run from set T to a disjoint set R is bipartite and directed one way, so it is acyclic **by construction** — whatever the sources say. Observing 'zero loops' is therefore not evidence about the sources.

<details><summary>Sources confirming the top pair (error → fixed)</summary>

if_pyth, mbyte, message, motion, options, os_vms, pi_getscript, pi_netrw, pi_vimball, pi_zip, popup, syntax, todo, usr_30, version5, version6, version7, version8, version9, vi_diff

</details>
