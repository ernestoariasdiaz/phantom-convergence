# ANIMUS v5 — extraction summary

- Server: `http://127.0.0.1:8080`  ·  passage size ≈ 180 words  ·  check: exact quote, ≥80% of word trigrams found in the passage
- Passages in books: 27935  ·  after filters and sampling: 4171
- Filtered out before the model: OCR noise: 16, bibliography/references: 696, code or markup: 647, not prose (few letters): 3049, numbers/table/index: 13
- Sent to the model this run: 4171  (resumed: 0, errors: 0)
- Ideas per passage: 0 → 1  ·  1 → 619  ·  2 → 375  ·  3 → 3176
- Ideas proposed: 10897  ·  kept: 10392  ·  **rejected: 5%** (quote not in passage: 444, quote too short: 61)
- Model time: 4.0 s per passage

| Book | Passages read | Ideas proposed | Ideas kept |
|---|---|---|---|
| 83800_spa | 150 | 358 | 316 |
| Axelrod_Robert_The_Evolution_of_Cooperation | 150 | 431 | 429 |
| C_Book_2nd | 150 | 407 | 402 |
| Complexity_-_A_Guided_Tour | 150 | 393 | 385 |
| Computer-Networks---A-Tanenbaum---5th-edition | 150 | 395 | 388 |
| DAUKINS-1989-EL-GEN-EGOISTA | 150 | 391 | 391 |
| El Arte de la Guerra Sun Tzu | 79 | 222 | 220 |
| GEBen | 150 | 383 | 361 |
| Guns, Germs, and Steel The Fates of Human Societies - Jared Diamond | 150 | 417 | 414 |
| J Clark Scott - But How Do It Know_ - The Basic Principles of Computers for Everyone-John C. Scott (2009) | 150 | 386 | 385 |
| Kuhn-SSR-2ndEd | 150 | 411 | 392 |
| Meadows-2008.-Thinking-in-Systems | 150 | 407 | 387 |
| Meditaciones (Gredos) - Marco Aurelio Antonino Augusto | 150 | 393 | 391 |
| Naomi Klein - La doctrina del shock | 150 | 398 | 396 |
| No-Starch-Press-The-Rust | 150 | 361 | 357 |
| Operating-Systems-Design-and-Implementation-3rd-Edition | 150 | 417 | 414 |
| Polya G - Como Plantear Y Resolver Problemas | 150 | 357 | 325 |
| Tao Te Ching - LaoTse | 42 | 116 | 113 |
| The Master Algorithm_ How the Quest for the Ultimate Learning Machine Will Remake Our World - Pedro Domingos - Basic Books (2015) | 150 | 393 | 370 |
| The Pragmatic Programmer Your Journey to Mastery, 20th Anniversary Edition by Andrew Hunt David Hurst Thomas | 150 | 355 | 353 |
| Trump_ The Art of the Deal | 150 | 390 | 385 |
| UNIX_A_History_And_A_Memoir | 150 | 390 | 254 |
| Why-Nations-Fail_-The-Origins-o-Daron-Acemoglu | 150 | 420 | 417 |
| __Debt__The_First_5_000_Years | 150 | 382 | 254 |
| antifragil-9788449337338 | 150 | 390 | 386 |
| code-charles-petzold | 150 | 363 | 361 |
| designing-data-intensive-applications-the-big-ideas-behind-reliable-scalable-and-maintainable-systems | 150 | 392 | 390 |
| lanueva | 150 | 361 | 342 |
| los-origenes-del-orden-politico-9788423424979 | 150 | 418 | 414 |

Files: `claims.jsonl` (every idea with its quote, kept and rejected), `passages.jsonl` and `progress.jsonl` (not published: they contain book text).
