# ANIMUS v5 — cross-book links

- 10392 verified ideas from 29 books · 52067685 cross-book pairs compared

## 1. Can the embedder recognise meaning at all? (positive control)

Each idea must pick its own source quote out of 20 quotes (its own + 19 random). Chance = 5%.

| Book | Correct | Ideas |
|---|---|---|
| 83800_spa | 88% | 316 |
| Axelrod_Robert_The_Evolution | 98% | 429 |
| C_Book_2nd | 99% | 402 |
| Complexity_-_A_Guided_Tour | 98% | 385 |
| Computer-Networks---A-Tanenb | 99% | 388 |
| DAUKINS-1989-EL-GEN-EGOISTA | 98% | 391 |
| El Arte de la Guerra Sun Tzu | 100% | 220 |
| GEBen | 95% | 361 |
| Guns, Germs, and Steel The F | 98% | 414 |
| J Clark Scott - But How Do I | 97% | 385 |
| Kuhn-SSR-2ndEd | 97% | 392 |
| Meadows-2008.-Thinking-in-Sy | 98% | 387 |
| Meditaciones (Gredos) - Marc | 97% | 391 |
| Naomi Klein - La doctrina de | 96% | 396 |
| No-Starch-Press-The-Rust | 99% | 357 |
| Operating-Systems-Design-and | 99% | 414 |
| Polya G - Como Plantear Y Re | 96% | 325 |
| Tao Te Ching - LaoTse | 99% | 113 |
| The Master Algorithm_ How th | 97% | 370 |
| The Pragmatic Programmer You | 97% | 353 |
| Trump_ The Art of the Deal | 93% | 385 |
| UNIX_A_History_And_A_Memoir | 94% | 254 |
| Why-Nations-Fail_-The-Origin | 98% | 417 |
| __Debt__The_First_5_000_Year | 98% | 254 |
| antifragil-9788449337338 | 97% | 386 |
| code-charles-petzold | 97% | 361 |
| designing-data-intensive-app | 99% | 390 |
| lanueva | 99% | 342 |
| los-origenes-del-orden-polit | 99% | 414 |
| **All** | **97%** | 10392 |

Decoy closeness: an idea and its decoy have mean similarity **0.89**.

Decoy mode: **words** — every word shuffled.

## 2. Links found vs. links that word overlap alone produces

τ = 0.610 (the 99th percentile of all cross-book similarities; median is 0.437). A link = mutual nearest neighbours across two books with similarity ≥ τ.

| | Links |
|---|---|
| Real ideas | **18258** |
| Decoys (all words shuffled), mean of 3 runs | 20050.3 (runs: [20066, 19849, 20236]) |

**Decoys produce about as many links as real ideas: these links are explained by shared words and topic, not by shared meaning.**

## Negation check

- Ideas containing a negation (not, no, never, nobody…): **9%**
- If negation played no role, a link would have both sides negated about **1%** of the time
- All links with both sides negated: **5%** (891 of 18258)
- Top 100 links by lift with both sides negated: **42%** (42 of 100)

## 3. Links, best meaning lift first

`lift` = similarity minus the similarity of the same ideas with their words shuffled. Near 0 → the match is word overlap.

### 1. sim 0.734 · lift +0.212

- **GEBen** — An explanation at a certain level of detail does not exist for certain propositions.
- **Meadows-2008.-Thinking-in-Sy** — No one can define or measure various concepts.

### 2. sim 0.655 · lift +0.184

- **83800_spa** — Seeking defense from those who observe celestial bodies is futile.
- **Meadows-2008.-Thinking-in-Sy** — Seeing events as a series of events has little predictive or explanatory value.

### 3. sim 0.706 · lift +0.181

- **Guns, Germs, and Steel The F** — Physical and linguistic evidence has not solved the mysteries of prehistoric engulfings.
- **J Clark Scott - But How Do I** — Complete and exact structures and functions have not been uncovered.

### 4. sim 0.706 · lift +0.176

- **Meadows-2008.-Thinking-in-Sy** — Self-regulatory systems do not cause problems.
- **Meditaciones (Gredos) - Marc** — The cessation of energy at the right time does not cause harm.

### 5. sim 0.856 · lift +0.175

- **Complexity_-_A_Guided_Tour** — A gene can be defined as a substring of DNA that codes for a particular protein.
- **DAUKINS-1989-EL-GEN-EGOISTA** — A gene is defined as a sequence of nucleotides situated between start and stop symbols that code for a protein chain.

### 6. sim 0.773 · lift +0.175

- **No-Starch-Press-The-Rust** — A sequence of operations is performed to read a file's content.
- **Operating-Systems-Design-and** — A search is performed to find a process that can share text.

### 7. sim 0.649 · lift +0.174

- **designing-data-intensive-app** — A program does not care about the communication method.
- **lanueva** — Few real meanings of stories would materialize by simply continuing the execution of the algorithm details.

### 8. sim 0.697 · lift +0.173

- **GEBen** — An explanation at a certain level of detail does not exist for certain propositions.
- **Kuhn-SSR-2ndEd** — There is no theory-independent way to reconstruct phrases like “really there.

### 9. sim 0.754 · lift +0.172

- **El Arte de la Guerra Sun Tzu** — Not fighting keeps one's strength full.
- **Meditaciones (Gredos) - Marc** — The cessation of activity or rest is not a bad thing.

### 10. sim 0.740 · lift +0.171

- **C_Book_2nd** — The handling of certain computations is not universally defined by the language.
- **Complexity_-_A_Guided_Tour** — There are limits to what can be computed.

### 11. sim 0.709 · lift +0.170

- **DAUKINS-1989-EL-GEN-EGOISTA** — There is no objective way to judge the best strategy in a general and less arbitrary sense.
- **Meadows-2008.-Thinking-in-Sy** — There is no single, legitimate boundary to draw around a system.

### 12. sim 0.762 · lift +0.169

- **DAUKINS-1989-EL-GEN-EGOISTA** — There is no objective way to judge the best strategy in a general and less arbitrary sense.
- **The Master Algorithm_ How th** — There is no basis to choose one generalization over another.

### 13. sim 0.687 · lift +0.169

- **C_Book_2nd** — The handling of certain computations is not universally defined by the language.
- **Operating-Systems-Design-and** — Definitions are separated based on where they are used.

### 14. sim 0.723 · lift +0.168

- **J Clark Scott - But How Do I** — The device does not need to know to fulfill its function.
- **designing-data-intensive-app** — A program does not care about the communication method.

### 15. sim 0.621 · lift +0.168

- **83800_spa** — Seeking defense from those who observe celestial bodies is futile.
- **Polya G - Como Plantear Y Re** — It is absurd to take a hypothesis as true simply because it has occurred to one.

### 16. sim 0.628 · lift +0.167

- **Why-Nations-Fail_-The-Origin** — Cultural differences did not cause diverging economic fortunes between two regions.
- **__Debt__The_First_5_000_Year** — The new global economy did not cause a collapse of living standards in Europe by making precious metals available.

### 17. sim 0.697 · lift +0.167

- **El Arte de la Guerra Sun Tzu** — The advice in the text remains relevant despite the passage of time.
- **Meditaciones (Gredos) - Marc** — Truth, justice, and prudence do not extinguish.

### 18. sim 0.614 · lift +0.166

- **83800_spa** — Seeking defense from those who observe celestial bodies is futile.
- **The Master Algorithm_ How th** — There is no basis to choose one generalization over another.

### 19. sim 0.774 · lift +0.166

- **Polya G - Como Plantear Y Re** — One should consider the possibility without confusing it with certainty.
- **The Master Algorithm_ How th** — The aim is to touch each part without jumping to conclusions.

### 20. sim 0.677 · lift +0.165

- **Kuhn-SSR-2ndEd** — Textbooks truncate the scientist's sense of their discipline's history.
- **__Debt__The_First_5_000_Year** — Translation of ancient scripts significantly pushed back knowledge of written history.

### 21. sim 0.777 · lift +0.165

- **J Clark Scott - But How Do I** — The device does not need to know to fulfill its function.
- **Operating-Systems-Design-and** — Some devices do not use interrupts for their operation.

### 22. sim 0.613 · lift +0.165

- **Meadows-2008.-Thinking-in-Sy** — Chemical emissions decreased significantly despite no legal actions.
- **Trump_ The Art of the Deal** — Concerns about density were considered unwarranted due to low population decline in an area.

### 23. sim 0.665 · lift +0.164

- **C_Book_2nd** — Examples are not as concise and elegant as they might be.
- **lanueva** — The current theory is insufficient for a complete picture of the world.

### 24. sim 0.633 · lift +0.164

- **J Clark Scott - But How Do I** — Complete and exact structures and functions have not been uncovered.
- **antifragil-9788449337338** — The direction of information technologies has not been determined by academic science.

### 25. sim 0.685 · lift +0.163

- **J Clark Scott - But How Do I** — The device does not need to know to fulfill its function.
- **Meadows-2008.-Thinking-in-Sy** — Self-regulatory systems do not cause problems.


_Quotes from the books are removed from this published copy (copyright). Each idea's quote can be recovered by re-running `animus5 extract` on the same books and matched by `quote_sha256_16` in `claims.csv`._

