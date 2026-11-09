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

Decoy closeness: an idea and its decoy have mean similarity **0.91** — very high: this embedder barely notices word order, so the decoy test is weak.

Decoy mode: **content** — function words (not, no, never, there, is, can…) stay in place; only content words are shuffled. A decoy keeps the sentence's grammatical form but not its subject.

## 2. Links found vs. links that word overlap alone produces

τ = 0.610 (the 99th percentile of all cross-book similarities; median is 0.437). A link = mutual nearest neighbours across two books with similarity ≥ τ.

| | Links |
|---|---|
| Real ideas | **18258** |
| Decoys (content words shuffled, form kept), mean of 3 runs | 19709.0 (runs: [19683, 19770, 19674]) |

**Decoys produce about as many links as real ideas: these links are explained by shared words and topic, not by shared meaning.**

## Negation check

- Ideas containing a negation (not, no, never, nobody…): **9%**
- If negation played no role, a link would have both sides negated about **1%** of the time
- All links with both sides negated: **5%** (891 of 18258)
- Top 100 links by lift with both sides negated: **11%** (11 of 100)

## 3. Links, best meaning lift first

`lift` = similarity minus the similarity of the same ideas with their content words shuffled (grammar kept). Near 0 → the match is sentence form ("there is no X"), not subject.

### 1. sim 0.655 · lift +0.171

- **83800_spa** — Seeking defense from those who observe celestial bodies is futile.
- **Meadows-2008.-Thinking-in-Sy** — Seeing events as a series of events has little predictive or explanatory value.

### 2. sim 0.663 · lift +0.165

- **DAUKINS-1989-EL-GEN-EGOISTA** — Mutual accusations cease as cooperation becomes the main focus.
- **Trump_ The Art of the Deal** — Persistence led to the opposition receding.

### 3. sim 0.675 · lift +0.159

- **J Clark Scott - But How Do I** — The order of selecting storage locations does not affect speed.
- **Operating-Systems-Design-and** — Mounted file systems eliminate the need to specify file locations.

### 4. sim 0.726 · lift +0.156

- **Naomi Klein - La doctrina de** — A crisis contributed to a collapse of faith in private enterprise's ability to provide essential services.
- **los-origenes-del-orden-polit** — The inability to find adequate funds led to declarations of bankruptcy.

### 5. sim 0.697 · lift +0.155

- **El Arte de la Guerra Sun Tzu** — The advice in the text remains relevant despite the passage of time.
- **Meditaciones (Gredos) - Marc** — Truth, justice, and prudence do not extinguish.

### 6. sim 0.699 · lift +0.153

- **Meditaciones (Gredos) - Marc** — Serenity and firmness are key in deeply examined decisions.
- **The Pragmatic Programmer You** — Doubts can become solid when given time to address.

### 7. sim 0.637 · lift +0.152

- **Complexity_-_A_Guided_Tour** — Top scoring strategies share the characteristic of never being the first to defect.
- **El Arte de la Guerra Sun Tzu** — Good warriors secure positions where they cannot lose.

### 8. sim 0.680 · lift +0.148

- **Axelrod_Robert_The_Evolution** — Deviating from expected roles leads to negative consequences.
- **Guns, Germs, and Steel The F** — Devices often find uses other than their original intended purpose.

### 9. sim 0.678 · lift +0.147

- **Meadows-2008.-Thinking-in-Sy** — Seeing events as a series of events has little predictive or explanatory value.
- **lanueva** — Few real meanings of stories would materialize by simply continuing the execution of the algorithm details.

### 10. sim 0.660 · lift +0.143

- **GEBen** — People no longer feel awe even from sophisticated computer operations.
- **antifragil-9788449337338** — The absence of paperwork makes bureaucracy more acceptable than in times of paper files.

### 11. sim 0.709 · lift +0.142

- **The Master Algorithm_ How th** — Combining all the pieces into one solution is not obvious.
- **__Debt__The_First_5_000_Year** — A relationship between one person and everyone else is difficult to conceive.

### 12. sim 0.661 · lift +0.142

- **C_Book_2nd** — External data declarations without any specifiers or qualifiers are forbidden.
- **Kuhn-SSR-2ndEd** — Appeals to governing bodies or the general public in scientific matters are prohibited.

### 13. sim 0.711 · lift +0.141

- **Operating-Systems-Design-and** — Information hiding is essentially absent in this approach.
- **Trump_ The Art of the Deal** — The key to this approach was a lack of emotional attachment to people or products.

### 14. sim 0.723 · lift +0.141

- **Complexity_-_A_Guided_Tour** — Understanding how natural systems compute remains largely unanswered.
- **J Clark Scott - But How Do I** — Complete and exact structures and functions have not been uncovered.

### 15. sim 0.758 · lift +0.141

- **Complexity_-_A_Guided_Tour** — A procedure for solving a specific problem cannot exist.
- **GEBen** — An explanation at a certain level of detail does not exist for certain propositions.

### 16. sim 0.699 · lift +0.140

- **Meadows-2008.-Thinking-in-Sy** — Systems tend to produce exactly what they are asked to produce.
- **code-charles-petzold** — The speed of operations makes substantial completion possible.

### 17. sim 0.674 · lift +0.140

- **Naomi Klein - La doctrina de** — The essential element of surprise no longer exists in economic responses.
- **The Master Algorithm_ How th** — The need to earn a living will become a distant memory.

### 18. sim 0.647 · lift +0.139

- **Guns, Germs, and Steel The F** — The extinction eliminated candidates for domestication.
- **The Master Algorithm_ How th** — The need to earn a living will become a distant memory.

### 19. sim 0.664 · lift +0.138

- **Kuhn-SSR-2ndEd** — Adapting the work for terrestrial problems was not clear.
- **The Master Algorithm_ How th** — Combining all the pieces into one solution is not obvious.

### 20. sim 0.737 · lift +0.136

- **GEBen** — A situation is resolved despite initial disappointment.
- **antifragil-9788449337338** — Exploitation leads to a kind of happy ending for the subject of the story.

### 21. sim 0.628 · lift +0.135

- **Complexity_-_A_Guided_Tour** — The definition of information ignores the meaning of messages and considers only transmission frequency.
- **lanueva** — Few real meanings of stories would materialize by simply continuing the execution of the algorithm details.

### 22. sim 0.657 · lift +0.135

- **J Clark Scott - But How Do I** — Complete and exact structures and functions have not been uncovered.
- **Meadows-2008.-Thinking-in-Sy** — Problems rooted in the internal structure of complex systems have resisted resolution.

### 23. sim 0.810 · lift +0.134

- **Meditaciones (Gredos) - Marc** — The subject avoided certain factions and occupations.
- **Trump_ The Art of the Deal** — A person avoided projecting a certain image regarding their sexual orientation.

### 24. sim 0.732 · lift +0.134

- **Complexity_-_A_Guided_Tour** — There are limits to what can be computed.
- **GEBen** — A system cannot fully escape its constraints.

### 25. sim 0.757 · lift +0.133

- **Complexity_-_A_Guided_Tour** — Persistence in seeking contact led to a breakthrough.
- **Trump_ The Art of the Deal** — The achievement was recognized by officials.


_Quotes from the books are removed from this published copy (copyright). Each idea's quote can be recovered by re-running `animus5 extract` on the same books and matched by `quote_sha256_16` in `claims.csv`._

