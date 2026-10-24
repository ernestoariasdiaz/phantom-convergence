# convaudit report

- Corpus: `corpora/vimdocs` — 151 sources, 5340 windows of 300 tokens
- Vocabulary: 4 tension × 4 resolution words → 15 pairs observed at least once
- Null baseline: 50 random vocabularies, each word matched to a corpus word of similar source frequency

## 1. Does the vocabulary beat random words?

| Metric | Real vocabulary | Random vocabularies (mean) |
|---|---|---|
| Strongest pair: # sources 'confirming' | 11 | 8.9 (range 2–19) |
| Pairs 'confirmed' by ≥10 sources | 2 | 0.6 |
| Pairs significant after FDR (q<0.05) | 6 | 0.0 |

Example from a random vocabulary: **complicated → unloaded** is 'confirmed' by 2 sources.

## 2. Top pairs: observed vs. expected by chance

`expected` = sources that would 'confirm' the pair if both words were scattered at random inside each source (exact hypergeometric). `p` = exact Poisson-binomial tail; `q` = Benjamini–Hochberg.

| Pair | Sources confirming | Expected by chance | Window lift | p | q | Verdict |
|---|---|---|---|---|---|---|
| netbeans → highlighting | 11 | 7.1 | 0.81 | 0.0039 | 0.0198 | not significant |
| perl → highlighting | 10 | 12.4 | 0.82 | 0.9617 | 0.9617 | frequency artifact |
| ruby → highlighting | 9 | 7.5 | 1.09 | 0.2134 | 0.3535 | not significant |
| python → highlighting | 9 | 10.3 | 0.86 | 0.8692 | 0.9313 | frequency artifact |
| python → dll | 8 | 5.1 | 1.76 | 0.0059 | 0.0198 | association beyond chance |
| perl → dll | 8 | 4.7 | 1.80 | 0.0066 | 0.0198 | association beyond chance |
| netbeans → socket | 6 | 2.4 | 2.74 | 0.0027 | 0.0198 | association beyond chance |
| ruby → dll | 6 | 3.9 | 1.69 | 0.0393 | 0.0842 | not significant |
| python → interpreter | 5 | 2.0 | 2.43 | 0.0052 | 0.0198 | association beyond chance |
| perl → interpreter | 4 | 1.1 | 3.52 | 0.0133 | 0.0334 | association beyond chance |
| python → socket | 4 | 2.5 | 2.07 | 0.1976 | 0.3535 | not significant |
| netbeans → dll | 4 | 2.9 | 1.47 | 0.2558 | 0.3535 | not significant |
| ruby → socket | 2 | 1.0 | 2.64 | 0.2592 | 0.3535 | not significant |
| ruby → interpreter | 1 | 0.5 | 2.07 | 0.3970 | 0.4963 | not significant |
| perl → socket | 1 | 1.1 | 0.81 | 0.7236 | 0.8350 | frequency artifact |

## 3. Pairs that survive the test

| Pair | Sources | Expected | Window lift | q |
|---|---|---|---|---|
| netbeans → socket | 6 | 2.4 | 2.74 | 0.0198 |
| python → interpreter | 5 | 2.0 | 2.43 | 0.0198 |
| python → dll | 8 | 5.1 | 1.76 | 0.0198 |
| perl → dll | 8 | 4.7 | 1.80 | 0.0198 |
| perl → interpreter | 4 | 1.1 | 3.52 | 0.0334 |

## 4. Can this graph ever contain a feedback loop?

No. Every edge goes from a tension word to a resolution word, and the two lists share no word. A graph whose edges only run from set T to a disjoint set R is bipartite and directed one way, so it is acyclic **by construction** — whatever the sources say. Observing 'zero loops' is therefore not evidence about the sources.

<details><summary>Sources confirming the top pair (netbeans → highlighting)</summary>

gui_x11, help, index, netbeans, options, sign, various, version6, version7, version8, version9

</details>
