# convaudit report

- Corpus: `txts` — 29 sources, 93344 windows of 60 tokens
- Vocabulary: 50 tension × 39 resolution words → 336 pairs observed at least once
- Null baseline: 50 random vocabularies, each word matched to a corpus word of similar source frequency

## 1. Does the vocabulary beat random words?

| Metric | Real vocabulary | Random vocabularies (mean) |
|---|---|---|
| Strongest pair: # sources 'confirming' | 18 | 9.1 (range 3–18) |
| Pairs 'confirmed' by ≥10 sources | 5 | 1.1 |
| Pairs significant after FDR (q<0.05) | 5 | 0.3 |

Example from a random vocabulary: **small → needs** is 'confirmed' by 9 sources.

## 2. Top pairs: observed vs. expected by chance

`expected` = sources that would 'confirm' the pair if both words were scattered at random inside each source (exact hypergeometric). `p` = exact Poisson-binomial tail; `q` = Benjamini–Hochberg.

| Pair | Sources confirming | Expected by chance | Window lift | p | q | Verdict |
|---|---|---|---|---|---|---|
| problem → solution | 18 | 8.5 | 6.64 | 2.3e-9 | 7.8e-7 | association beyond chance |
| problem → solutions | 12 | 4.9 | 5.13 | 1.2e-5 | 0.0010 | association beyond chance |
| problems → solutions | 11 | 3.2 | 6.00 | 4.3e-7 | 7.2e-5 | association beyond chance |
| problems → solution | 10 | 5.8 | 3.38 | 0.0100 | 0.3364 | not significant |
| problem → developed | 10 | 7.8 | 0.97 | 0.1683 | 0.6319 | not significant |
| problems → algorithms | 9 | 4.8 | 2.25 | 0.0002 | 0.0102 | association beyond chance |
| problem → algorithms | 8 | 6.0 | 1.14 | 0.0598 | 0.6105 | not significant |
| problema → solución | 7 | 3.0 | 1.79 | 8.1e-6 | 0.0009 | association beyond chance |
| problem → discovered | 7 | 5.6 | 1.70 | 0.2833 | 0.7290 | not significant |
| problem → fixed | 7 | 6.0 | 1.19 | 0.3439 | 0.7527 | not significant |
| problems → architecture | 6 | 3.3 | 1.21 | 0.0454 | 0.5095 | not significant |
| error → fixed | 6 | 4.3 | 0.98 | 0.1426 | 0.6229 | not significant |
| error → solution | 6 | 5.0 | 1.06 | 0.3427 | 0.7527 | not significant |
| problems → algorithm | 6 | 5.4 | 1.43 | 0.4471 | 0.7817 | frequency artifact |
| problems → developed | 6 | 5.4 | 1.33 | 0.4560 | 0.7817 | frequency artifact |
| problem → algorithm | 6 | 6.6 | 1.08 | 0.8702 | 0.9689 | frequency artifact |
| problem → regulation | 5 | 2.2 | 2.19 | 0.0335 | 0.4473 | not significant |
| failure → solution | 5 | 2.1 | 3.11 | 0.0346 | 0.4473 | not significant |
| problem → agreement | 5 | 2.9 | 1.65 | 0.1144 | 0.6229 | not significant |
| problems → policy | 5 | 2.9 | 1.65 | 0.1267 | 0.6229 | not significant |
| problems → discovered | 5 | 3.6 | 0.87 | 0.2541 | 0.7097 | not significant |
| problema → desarrollo | 5 | 4.0 | 1.57 | 0.3282 | 0.7504 | not significant |
| error → algorithm | 5 | 5.6 | 0.88 | 0.8690 | 0.9689 | frequency artifact |
| problems → education | 4 | 2.6 | 1.03 | 0.2085 | 0.6824 | not significant |
| errors → algorithms | 4 | 3.1 | 1.25 | 0.3651 | 0.7549 | not significant |
| error → developed | 4 | 3.1 | 1.20 | 0.3717 | 0.7549 | not significant |
| problem → framework | 4 | 3.3 | 1.13 | 0.4037 | 0.7619 | not significant |
| problems → fixed | 4 | 4.4 | 0.72 | 0.7587 | 0.9251 | frequency artifact |
| problem → policy | 4 | 4.9 | 1.40 | 0.8237 | 0.9593 | frequency artifact |
| waste → solution | 3 | 0.5 | 5.36 | 0.0127 | 0.3542 | not significant |
| failures → architecture | 3 | 1.0 | 2.02 | 0.0188 | 0.4473 | not significant |
| limitations → developed | 3 | 0.7 | 3.66 | 0.0239 | 0.4473 | not significant |
| debt → investment | 3 | 1.3 | 1.78 | 0.0299 | 0.4473 | not significant |
| poverty → policy | 3 | 1.1 | 4.47 | 0.0316 | 0.4473 | not significant |
| crisis → inversión | 3 | 1.3 | 3.21 | 0.0440 | 0.5095 | not significant |
| crisis → reform | 3 | 1.4 | 3.01 | 0.0848 | 0.6209 | not significant |
| riesgo → solución | 3 | 1.6 | 1.11 | 0.1015 | 0.6209 | not significant |
| loss → developed | 3 | 1.3 | 1.98 | 0.1168 | 0.6229 | not significant |
| crisis → policy | 3 | 1.6 | 2.17 | 0.1492 | 0.6229 | not significant |
| error → solución | 3 | 1.8 | 1.26 | 0.1634 | 0.6319 | not significant |
| riesgo → política | 3 | 2.0 | 1.32 | 0.2953 | 0.7406 | not significant |
| error → discovered | 3 | 2.0 | 1.31 | 0.3017 | 0.7453 | not significant |
| problems → agreement | 3 | 2.1 | 1.26 | 0.3354 | 0.7504 | not significant |
| problems → reduced | 3 | 2.2 | 1.19 | 0.3959 | 0.7619 | not significant |
| problem → investment | 3 | 2.4 | 0.89 | 0.4530 | 0.7817 | not significant |
| problema → descubrimiento | 3 | 2.6 | 0.99 | 0.5496 | 0.8328 | not significant |
| crisis → política | 3 | 2.6 | 1.16 | 0.6200 | 0.8697 | frequency artifact |
| problem → reduced | 3 | 3.8 | 0.98 | 0.8111 | 0.9573 | frequency artifact |
| problema → política | 3 | 3.2 | 1.09 | 0.8675 | 0.9689 | frequency artifact |
| errors → algorithm | 3 | 4.0 | 0.95 | 0.9310 | 0.9978 | frequency artifact |
| problem → architecture | 3 | 5.2 | 0.36 | 0.9821 | 1.0000 | frequency artifact |
| error → algorithms | 3 | 4.7 | 0.65 | 0.9856 | 1.0000 | frequency artifact |
| risk → prevention | 2 | 0.1 | 46.15 | 0.0014 | 0.0810 | not significant |
| monopoly → regulation | 2 | 0.2 | 16.36 | 0.0058 | 0.2795 | not significant |
| deuda → regulación | 2 | 0.2 | 9.61 | 0.0118 | 0.3542 | not significant |
| fracaso → descubrimiento | 2 | 0.3 | 5.86 | 0.0270 | 0.4473 | not significant |
| gap → reduced | 2 | 0.3 | 6.89 | 0.0294 | 0.4473 | not significant |
| limitations → solutions | 2 | 0.3 | 9.14 | 0.0324 | 0.4473 | not significant |
| risk → innovation | 2 | 0.3 | 5.90 | 0.0339 | 0.4473 | not significant |
| bottleneck → solution | 2 | 0.4 | 5.05 | 0.0442 | 0.5095 | not significant |
| waste → policy | 2 | 0.4 | 5.23 | 0.0455 | 0.5095 | not significant |
| pérdida → inversión | 2 | 0.5 | 3.75 | 0.0748 | 0.6209 | not significant |
| poverty → education | 2 | 0.8 | 1.75 | 0.0780 | 0.6209 | not significant |
| collapse → education | 2 | 0.6 | 2.63 | 0.0842 | 0.6209 | not significant |
| fraud → algorithms | 2 | 0.5 | 3.52 | 0.0883 | 0.6209 | not significant |
| collapse → solution | 2 | 0.5 | 3.57 | 0.0926 | 0.6209 | not significant |
| poverty → investment | 2 | 0.6 | 2.66 | 0.0929 | 0.6209 | not significant |
| collapse → algorithms | 2 | 0.6 | 3.05 | 0.0939 | 0.6209 | not significant |
| barrier → developed | 2 | 0.6 | 2.68 | 0.0954 | 0.6209 | not significant |
| problem → prevention | 2 | 0.6 | 3.34 | 0.0988 | 0.6209 | not significant |
| problema → arquitectura | 2 | 0.7 | 2.55 | 0.1053 | 0.6209 | not significant |
| failure → agreement | 2 | 0.6 | 5.81 | 0.1207 | 0.6229 | not significant |
| error → investment | 2 | 0.6 | 3.02 | 0.1255 | 0.6229 | not significant |
| gap → developed | 2 | 0.7 | 2.70 | 0.1474 | 0.6229 | not significant |
| corrupción → reforma | 2 | 0.8 | 2.04 | 0.1528 | 0.6259 | not significant |
| crisis → investment | 2 | 0.7 | 3.65 | 0.1569 | 0.6319 | not significant |
| errors → reduced | 2 | 0.8 | 2.25 | 0.1694 | 0.6319 | not significant |
| failures → fixed | 2 | 0.9 | 2.34 | 0.1803 | 0.6479 | not significant |
| crisis → discovered | 2 | 1.1 | 0.85 | 0.1907 | 0.6675 | not significant |
| failure → policy | 2 | 0.9 | 2.09 | 0.2102 | 0.6824 | not significant |
| decline → developed | 2 | 1.0 | 2.11 | 0.2314 | 0.6824 | not significant |
| conflict → algorithms | 2 | 1.1 | 1.95 | 0.2461 | 0.7051 | not significant |
| failures → developed | 2 | 1.0 | 1.86 | 0.2481 | 0.7051 | not significant |
| collapse → algorithm | 2 | 1.0 | 1.50 | 0.2497 | 0.7051 | not significant |
| fracaso → desarrollo | 2 | 1.1 | 1.80 | 0.2557 | 0.7097 | not significant |
| pobreza → desarrollo | 2 | 1.1 | 2.65 | 0.2756 | 0.7246 | not significant |
| debt → policy | 2 | 1.3 | 1.37 | 0.3078 | 0.7495 | not significant |
| crisis → innovación | 2 | 1.1 | 1.39 | 0.3178 | 0.7504 | not significant |
| limitations → algorithms | 2 | 1.2 | 2.50 | 0.3228 | 0.7504 | not significant |
| crisis → solución | 2 | 1.2 | 2.44 | 0.3327 | 0.7504 | not significant |
| crisis → solution | 2 | 1.4 | 0.59 | 0.3551 | 0.7549 | not significant |
| problema → reforma | 2 | 1.2 | 2.06 | 0.3580 | 0.7549 | not significant |
| risk → solution | 2 | 1.2 | 1.48 | 0.3602 | 0.7549 | not significant |
| error → education | 2 | 1.3 | 1.32 | 0.3719 | 0.7549 | not significant |
| failure → architecture | 2 | 1.3 | 1.43 | 0.3722 | 0.7549 | not significant |
| crisis → education | 2 | 1.3 | 1.18 | 0.3774 | 0.7549 | not significant |
| conflict → algorithm | 2 | 1.4 | 1.77 | 0.4015 | 0.7619 | not significant |
| failures → algorithms | 2 | 1.4 | 1.30 | 0.4038 | 0.7619 | not significant |
| waste → algorithm | 2 | 1.4 | 1.55 | 0.4059 | 0.7619 | not significant |
| bottleneck → algorithm | 2 | 1.3 | 1.17 | 0.4102 | 0.7619 | not significant |
| riesgo → acuerdo | 2 | 1.3 | 1.73 | 0.4104 | 0.7619 | not significant |
| conflict → policy | 2 | 1.4 | 1.62 | 0.4320 | 0.7745 | not significant |
| problems → innovation | 2 | 1.4 | 2.77 | 0.4370 | 0.7768 | not significant |
| problema → avance | 2 | 1.4 | 1.82 | 0.4405 | 0.7789 | not significant |
| desigualdad → política | 2 | 1.5 | 1.76 | 0.4460 | 0.7817 | not significant |
| problema → innovación | 2 | 1.5 | 1.17 | 0.4540 | 0.7817 | not significant |
| loss → solution | 2 | 1.5 | 1.47 | 0.4547 | 0.7817 | not significant |
| fracaso → política | 2 | 1.5 | 0.89 | 0.4607 | 0.7856 | not significant |
| inflación → política | 2 | 1.4 | 0.89 | 0.4767 | 0.8008 | not significant |
| problems → investment | 2 | 1.5 | 1.06 | 0.4876 | 0.8035 | not significant |
| crisis → acuerdo | 2 | 1.7 | 1.24 | 0.5503 | 0.8328 | not significant |
| problems → cooperation | 2 | 1.6 | 1.24 | 0.5563 | 0.8336 | not significant |
| deuda → desarrollo | 2 | 1.6 | 1.52 | 0.5680 | 0.8408 | not significant |
| failures → algorithm | 2 | 1.7 | 1.52 | 0.5783 | 0.8444 | not significant |
| crisis → developed | 2 | 1.8 | 1.06 | 0.5806 | 0.8444 | frequency artifact |
| error → acuerdo | 2 | 1.8 | 1.67 | 0.5950 | 0.8580 | frequency artifact |
| error → framework | 2 | 1.9 | 0.83 | 0.6208 | 0.8697 | frequency artifact |
| failure → algorithms | 2 | 2.0 | 0.89 | 0.6664 | 0.8931 | frequency artifact |
| loss → algorithms | 2 | 2.0 | 0.57 | 0.6983 | 0.9096 | frequency artifact |
| corrupción → política | 2 | 1.7 | 2.80 | 0.7220 | 0.9189 | frequency artifact |
| conflicto → política | 2 | 1.9 | 1.44 | 0.7253 | 0.9196 | frequency artifact |
| problema → inversión | 2 | 2.2 | 0.94 | 0.7497 | 0.9251 | frequency artifact |
| risk → algorithm | 2 | 2.2 | 1.07 | 0.7678 | 0.9251 | frequency artifact |
| errors → fixed | 2 | 2.3 | 0.83 | 0.7682 | 0.9251 | frequency artifact |
| error → política | 2 | 2.2 | 0.50 | 0.7751 | 0.9251 | frequency artifact |
| conflict → developed | 2 | 2.4 | 1.24 | 0.7792 | 0.9252 | frequency artifact |
| deuda → política | 2 | 2.0 | 0.84 | 0.8148 | 0.9573 | frequency artifact |
| error → solutions | 2 | 2.7 | 0.59 | 0.8246 | 0.9593 | frequency artifact |
| failure → algorithm | 2 | 2.6 | 0.95 | 0.8591 | 0.9689 | frequency artifact |
| errors → solution | 2 | 2.8 | 0.75 | 0.8593 | 0.9689 | frequency artifact |
| problem → cooperation | 2 | 2.2 | 1.77 | 0.9022 | 0.9779 | frequency artifact |
| crisis → desarrollo | 2 | 2.2 | 1.70 | 0.9644 | 1.0000 | frequency artifact |
| problema → acuerdo | 2 | 4.0 | 0.71 | 0.9885 | 1.0000 | frequency artifact |
| escasez → automatización | 1 | 0.0 | 150.00 | 0.0067 | 0.2795 | not significant |
| fraud → prevention | 1 | 0.0 | 225.27 | 0.0088 | 0.3302 | not significant |
| waste → reforma | 1 | 0.0 | 72.97 | 0.0137 | 0.3542 | not significant |
| vulnerabilidad → cooperación | 1 | 0.0 | 34.13 | 0.0290 | 0.4473 | not significant |
| inequality → algorithms | 1 | 0.0 | 32.60 | 0.0306 | 0.4473 | not significant |
| brecha → solución | 1 | 0.0 | 30.82 | 0.0320 | 0.4473 | not significant |
| limitations → reform | 1 | 0.1 | 17.84 | 0.0551 | 0.5972 | not significant |
| failures → prevention | 1 | 0.1 | 16.28 | 0.0600 | 0.6105 | not significant |
| limitación → solución | 1 | 0.1 | 15.10 | 0.0643 | 0.6209 | not significant |
| vulnerabilidad → descubrimiento | 1 | 0.1 | 15.01 | 0.0648 | 0.6209 | not significant |
| fraud → reforma | 1 | 0.1 | 14.59 | 0.0667 | 0.6209 | not significant |
| limitación → educación | 1 | 0.1 | 13.72 | 0.0707 | 0.6209 | not significant |
| shortage → developed | 1 | 0.1 | 12.51 | 0.0772 | 0.6209 | not significant |
| fraud → cooperation | 1 | 0.1 | 10.70 | 0.0900 | 0.6209 | not significant |
| colapso → reform | 1 | 0.1 | 10.46 | 0.0916 | 0.6209 | not significant |
| fraud → regulation | 1 | 0.1 | 10.31 | 0.0927 | 0.6209 | not significant |
| inflation → solution | 1 | 0.1 | 10.30 | 0.0929 | 0.6209 | not significant |
| shortage → solution | 1 | 0.1 | 9.80 | 0.0976 | 0.6209 | not significant |
| pobreza → arquitectura | 1 | 0.1 | 9.15 | 0.1040 | 0.6209 | not significant |
| failure → breakthrough | 1 | 0.1 | 9.10 | 0.1047 | 0.6209 | not significant |
| breach → solution | 1 | 0.1 | 8.79 | 0.1082 | 0.6229 | not significant |
| escasez → cooperación | 1 | 0.1 | 8.56 | 0.1109 | 0.6229 | not significant |
| poverty → initiative | 1 | 0.1 | 7.98 | 0.1191 | 0.6229 | not significant |
| declive → acuerdo | 1 | 0.1 | 7.80 | 0.1210 | 0.6229 | not significant |
| inequality → framework | 1 | 0.1 | 7.26 | 0.1298 | 0.6229 | not significant |
| loss → reform | 1 | 0.1 | 7.18 | 0.1314 | 0.6229 | not significant |
| decline → framework | 1 | 0.1 | 7.14 | 0.1316 | 0.6229 | not significant |
| monopolio → educación | 1 | 0.1 | 14.21 | 0.1319 | 0.6229 | not significant |
| barrier → policy | 1 | 0.1 | 7.06 | 0.1328 | 0.6229 | not significant |
| limitación → algoritmo | 1 | 0.1 | 6.88 | 0.1384 | 0.6229 | not significant |
| fraud → discovered | 1 | 0.1 | 6.65 | 0.1403 | 0.6229 | not significant |
| corrupción → iniciativa | 1 | 0.1 | 6.60 | 0.1414 | 0.6229 | not significant |
| problema → framework | 1 | 0.1 | 6.62 | 0.1431 | 0.6229 | not significant |
| error → breakthrough | 1 | 0.2 | 6.53 | 0.1432 | 0.6229 | not significant |
| debt → desarrollo | 1 | 0.1 | 6.32 | 0.1474 | 0.6229 | not significant |
| problema → automatización | 1 | 0.2 | 6.32 | 0.1502 | 0.6229 | not significant |
| vulnerability → algorithms | 1 | 0.2 | 5.66 | 0.1635 | 0.6319 | not significant |
| limitations → education | 1 | 0.2 | 5.62 | 0.1639 | 0.6319 | not significant |
| inflation → reform | 1 | 0.2 | 11.12 | 0.1663 | 0.6319 | not significant |
| failure → automation | 1 | 0.2 | 5.40 | 0.1705 | 0.6319 | not significant |
| declive → reforma | 1 | 0.2 | 5.37 | 0.1711 | 0.6319 | not significant |
| limitations → agreement | 1 | 0.2 | 5.25 | 0.1742 | 0.6364 | not significant |
| monopoly → architecture | 1 | 0.2 | 5.03 | 0.1813 | 0.6479 | not significant |
| poverty → framework | 1 | 0.2 | 4.99 | 0.1836 | 0.6495 | not significant |
| inequality → solution | 1 | 0.2 | 4.50 | 0.2007 | 0.6824 | not significant |
| waste → discovered | 1 | 0.2 | 4.40 | 0.2041 | 0.6824 | not significant |
| limitations → policy | 1 | 0.2 | 4.40 | 0.2046 | 0.6824 | not significant |
| bottleneck → developed | 1 | 0.2 | 4.17 | 0.2143 | 0.6824 | not significant |
| risk → innovación | 1 | 0.2 | 4.16 | 0.2154 | 0.6824 | not significant |
| waste → education | 1 | 0.2 | 4.13 | 0.2164 | 0.6824 | not significant |
| corruption → education | 1 | 0.2 | 8.14 | 0.2193 | 0.6824 | not significant |
| barrier → innovation | 1 | 0.2 | 4.06 | 0.2201 | 0.6824 | not significant |
| monopoly → fixed | 1 | 0.2 | 4.05 | 0.2203 | 0.6824 | not significant |
| escasez → acuerdo | 1 | 0.2 | 4.03 | 0.2212 | 0.6824 | not significant |
| desigualdad → descubrimiento | 1 | 0.2 | 3.95 | 0.2251 | 0.6824 | not significant |
| inflation → policy | 1 | 0.2 | 3.92 | 0.2263 | 0.6824 | not significant |
| desigualdad → cooperación | 1 | 0.2 | 3.92 | 0.2264 | 0.6824 | not significant |
| conflict → desarrollo | 1 | 0.2 | 3.93 | 0.2302 | 0.6824 | not significant |
| collapse → regulation | 1 | 0.3 | 3.80 | 0.2329 | 0.6824 | not significant |
| monopoly → agreement | 1 | 0.3 | 3.79 | 0.2335 | 0.6824 | not significant |
| waste → investment | 1 | 0.3 | 3.61 | 0.2441 | 0.7051 | not significant |
| errors → regulation | 1 | 0.3 | 6.76 | 0.2577 | 0.7097 | not significant |
| risk → regulation | 1 | 0.3 | 3.28 | 0.2637 | 0.7205 | not significant |
| decline → algorithm | 1 | 0.3 | 3.45 | 0.2710 | 0.7246 | not significant |
| limitation → solution | 1 | 0.3 | 3.17 | 0.2719 | 0.7246 | not significant |
| crisis → prevención | 1 | 0.3 | 3.18 | 0.2748 | 0.7246 | not significant |
| riesgo → reforma | 1 | 0.3 | 3.12 | 0.2760 | 0.7246 | not significant |
| inflación → inversión | 1 | 0.3 | 3.06 | 0.2805 | 0.7290 | not significant |
| pérdidas → inversión | 1 | 0.3 | 3.01 | 0.2842 | 0.7290 | not significant |
| decline → discovered | 1 | 0.3 | 2.97 | 0.2880 | 0.7330 | not significant |
| pérdida → cooperación | 1 | 0.3 | 2.93 | 0.2909 | 0.7350 | not significant |
| failures → education | 1 | 0.3 | 2.83 | 0.3000 | 0.7453 | not significant |
| error → reforma | 1 | 0.3 | 2.76 | 0.3062 | 0.7495 | not significant |
| desigualdad → innovación | 1 | 0.3 | 2.68 | 0.3130 | 0.7504 | not significant |
| crisis → framework | 1 | 0.4 | 2.69 | 0.3154 | 0.7504 | not significant |
| monopoly → solution | 1 | 0.4 | 2.62 | 0.3199 | 0.7504 | not significant |
| collapse → fixed | 1 | 0.4 | 2.55 | 0.3264 | 0.7504 | not significant |
| problems → breakthrough | 1 | 0.4 | 2.55 | 0.3308 | 0.7504 | not significant |
| conflict → política | 1 | 0.4 | 2.55 | 0.3330 | 0.7504 | not significant |
| problem → desarrollo | 1 | 0.4 | 2.52 | 0.3372 | 0.7504 | not significant |
| conflicto → solución | 1 | 0.4 | 4.89 | 0.3372 | 0.7504 | not significant |
| pérdidas → acuerdo | 1 | 0.4 | 2.37 | 0.3460 | 0.7527 | not significant |
| fracaso → reforma | 1 | 0.4 | 4.73 | 0.3472 | 0.7527 | not significant |
| debt → solutions | 1 | 0.4 | 2.42 | 0.3562 | 0.7549 | not significant |
| decline → reduced | 1 | 0.4 | 2.26 | 0.3598 | 0.7549 | not significant |
| bottleneck → architecture | 1 | 0.4 | 2.19 | 0.3697 | 0.7549 | not significant |
| riesgo → regulación | 1 | 0.4 | 2.15 | 0.3764 | 0.7549 | not significant |
| risk → reduced | 1 | 0.5 | 4.24 | 0.3774 | 0.7549 | not significant |
| conflict → investment | 1 | 0.4 | 2.06 | 0.3872 | 0.7619 | not significant |
| failures → policy | 1 | 0.5 | 2.03 | 0.3912 | 0.7619 | not significant |
| desigualdad → reforma | 1 | 0.4 | 2.01 | 0.3946 | 0.7619 | not significant |
| fraude → política | 1 | 0.4 | 2.02 | 0.3950 | 0.7619 | not significant |
| risk → education | 1 | 0.5 | 1.97 | 0.4006 | 0.7619 | not significant |
| inflación → acuerdo | 1 | 0.5 | 1.91 | 0.4104 | 0.7619 | not significant |
| loss → investment | 1 | 0.5 | 1.85 | 0.4203 | 0.7745 | not significant |
| failures → cooperation | 1 | 0.5 | 2.01 | 0.4237 | 0.7745 | not significant |
| loss → framework | 1 | 0.5 | 1.81 | 0.4272 | 0.7745 | not significant |
| collapse → reduced | 1 | 0.5 | 1.80 | 0.4288 | 0.7745 | not significant |
| limitations → framework | 1 | 0.5 | 1.79 | 0.4309 | 0.7745 | not significant |
| risk → solutions | 1 | 0.5 | 1.77 | 0.4334 | 0.7745 | not significant |
| risk → investment | 1 | 0.5 | 1.63 | 0.4630 | 0.7856 | not significant |
| monopoly → investment | 1 | 0.5 | 1.61 | 0.4658 | 0.7865 | not significant |
| crisis → iniciativa | 1 | 0.5 | 3.13 | 0.4802 | 0.8027 | not significant |
| crisis → regulación | 1 | 0.6 | 3.09 | 0.4833 | 0.8035 | not significant |
| failure → reform | 1 | 0.5 | 1.51 | 0.4903 | 0.8035 | not significant |
| gap → fixed | 1 | 0.6 | 1.49 | 0.4909 | 0.8035 | not significant |
| problem → breakthrough | 1 | 0.6 | 1.51 | 0.4926 | 0.8035 | not significant |
| corruption → cooperation | 1 | 0.5 | 1.69 | 0.5012 | 0.8131 | not significant |
| decline → investment | 1 | 0.6 | 2.90 | 0.5034 | 0.8131 | not significant |
| error → regulation | 1 | 0.7 | 1.42 | 0.5100 | 0.8182 | not significant |
| loss → reduced | 1 | 0.6 | 1.41 | 0.5114 | 0.8182 | not significant |
| conflict → reduced | 1 | 0.7 | 1.38 | 0.5176 | 0.8242 | not significant |
| loss → discovered | 1 | 0.7 | 1.36 | 0.5231 | 0.8290 | not significant |
| debt → framework | 1 | 0.6 | 1.44 | 0.5255 | 0.8290 | not significant |
| conflicto → cooperación | 1 | 0.7 | 1.33 | 0.5313 | 0.8328 | not significant |
| debt → reform | 1 | 0.6 | 1.37 | 0.5377 | 0.8328 | not significant |
| deuda → inversión | 1 | 0.6 | 1.31 | 0.5383 | 0.8328 | not significant |
| problems → automation | 1 | 0.7 | 2.60 | 0.5432 | 0.8328 | not significant |
| problema → iniciativa | 1 | 0.7 | 1.43 | 0.5462 | 0.8328 | not significant |
| problem → reform | 1 | 0.7 | 1.29 | 0.5470 | 0.8328 | not significant |
| corruption → algorithm | 1 | 0.6 | 1.28 | 0.5484 | 0.8328 | not significant |
| loss → cooperation | 1 | 0.6 | 1.42 | 0.5553 | 0.8336 | not significant |
| risk → framework | 1 | 0.7 | 1.24 | 0.5582 | 0.8336 | not significant |
| corruption → algorithms | 1 | 0.6 | 2.45 | 0.5643 | 0.8390 | not significant |
| monopoly → education | 1 | 0.6 | 2.40 | 0.5718 | 0.8426 | not significant |
| crisis → innovation | 1 | 0.8 | 1.19 | 0.5759 | 0.8444 | not significant |
| conflict → solutions | 1 | 0.8 | 1.15 | 0.5837 | 0.8454 | not significant |
| conflict → discovered | 1 | 0.8 | 1.11 | 0.5998 | 0.8586 | not significant |
| escasez → política | 1 | 0.8 | 1.13 | 0.6005 | 0.8586 | not significant |
| failures → framework | 1 | 0.7 | 1.08 | 0.6096 | 0.8679 | not significant |
| declive → desarrollo | 1 | 0.6 | 1.10 | 0.6124 | 0.8682 | not significant |
| waste → algorithms | 1 | 0.8 | 1.05 | 0.6232 | 0.8697 | not significant |
| conflicto → reforma | 1 | 0.8 | 5.13 | 0.6266 | 0.8697 | not significant |
| risk → fixed | 1 | 0.8 | 1.02 | 0.6293 | 0.8697 | not significant |
| problema → regulación | 1 | 0.8 | 1.03 | 0.6302 | 0.8697 | not significant |
| errors → policy | 1 | 0.9 | 1.01 | 0.6324 | 0.8697 | frequency artifact |
| monopolio → desarrollo | 1 | 0.7 | 1.03 | 0.6342 | 0.8697 | not significant |
| collapse → innovation | 1 | 0.7 | 2.97 | 0.6425 | 0.8765 | not significant |
| error → agreement | 1 | 0.9 | 0.98 | 0.6443 | 0.8765 | frequency artifact |
| collapse → reform | 1 | 0.7 | 0.95 | 0.6562 | 0.8861 | not significant |
| conflict → education | 1 | 0.9 | 0.95 | 0.6566 | 0.8861 | not significant |
| crisis → cooperation | 1 | 0.7 | 1.05 | 0.6699 | 0.8931 | not significant |
| monopoly → policy | 1 | 0.8 | 0.91 | 0.6719 | 0.8931 | not significant |
| decline → innovation | 1 | 0.7 | 0.91 | 0.6725 | 0.8931 | not significant |
| declive → política | 1 | 0.7 | 1.86 | 0.6760 | 0.8943 | not significant |
| conflict → framework | 1 | 0.9 | 0.89 | 0.6794 | 0.8953 | not significant |
| problems → regulation | 1 | 1.1 | 0.87 | 0.6889 | 0.9041 | frequency artifact |
| poverty → algorithm | 1 | 0.7 | 0.93 | 0.7002 | 0.9096 | not significant |
| risk → architecture | 1 | 1.0 | 0.83 | 0.7031 | 0.9096 | frequency artifact |
| riesgo → innovación | 1 | 0.8 | 0.84 | 0.7039 | 0.9096 | not significant |
| risk → cooperation | 1 | 0.8 | 0.93 | 0.7087 | 0.9124 | not significant |
| failure → solutions | 1 | 1.1 | 0.80 | 0.7176 | 0.9189 | frequency artifact |
| inequality → innovation | 1 | 0.7 | 0.80 | 0.7215 | 0.9189 | not significant |
| deuda → acuerdo | 1 | 1.0 | 2.31 | 0.7324 | 0.9218 | frequency artifact |
| conflicto → acuerdo | 1 | 1.1 | 0.77 | 0.7325 | 0.9218 | frequency artifact |
| inflación → desarrollo | 1 | 1.0 | 0.75 | 0.7482 | 0.9251 | frequency artifact |
| monopoly → innovation | 1 | 0.8 | 0.74 | 0.7482 | 0.9251 | not significant |
| failure → framework | 1 | 1.0 | 2.19 | 0.7508 | 0.9251 | frequency artifact |
| problems → reform | 1 | 1.0 | 1.47 | 0.7531 | 0.9251 | frequency artifact |
| conflict → reform | 1 | 0.8 | 0.73 | 0.7548 | 0.9251 | not significant |
| inequality → developed | 1 | 0.9 | 0.72 | 0.7563 | 0.9251 | not significant |
| poverty → política | 1 | 1.1 | 0.71 | 0.7669 | 0.9251 | frequency artifact |
| riesgo → inversión | 1 | 1.1 | 1.38 | 0.7713 | 0.9251 | frequency artifact |
| conflict → architecture | 1 | 1.0 | 0.68 | 0.7742 | 0.9251 | frequency artifact |
| gap → algorithm | 1 | 1.2 | 0.69 | 0.7764 | 0.9251 | frequency artifact |
| conflict → solution | 1 | 1.4 | 0.60 | 0.8130 | 0.9573 | frequency artifact |
| errors → developed | 1 | 1.4 | 1.16 | 0.8251 | 0.9593 | frequency artifact |
| riesgo → educación | 1 | 1.0 | 0.58 | 0.8304 | 0.9621 | frequency artifact |
| loss → architecture | 1 | 1.3 | 0.55 | 0.8428 | 0.9689 | frequency artifact |
| debt → regulation | 1 | 0.9 | 2.88 | 0.8483 | 0.9689 | frequency artifact |
| error → innovación | 1 | 0.9 | 1.08 | 0.8501 | 0.9689 | frequency artifact |
| error → inversión | 1 | 1.3 | 0.53 | 0.8547 | 0.9689 | frequency artifact |
| poverty → reform | 1 | 1.0 | 1.02 | 0.8660 | 0.9689 | frequency artifact |
| poverty → developed | 1 | 1.1 | 0.50 | 0.8698 | 0.9689 | frequency artifact |
| error → desarrollo | 1 | 1.6 | 0.50 | 0.8708 | 0.9689 | frequency artifact |
| error → descubrimiento | 1 | 1.1 | 1.97 | 0.8759 | 0.9712 | frequency artifact |
| debt → agreement | 1 | 1.0 | 2.50 | 0.8852 | 0.9779 | frequency artifact |
| loss → fixed | 1 | 1.4 | 0.91 | 0.8921 | 0.9779 | frequency artifact |
| desigualdad → desarrollo | 1 | 1.1 | 1.39 | 0.8937 | 0.9779 | frequency artifact |
| limitations → algorithm | 1 | 1.6 | 0.46 | 0.8991 | 0.9779 | frequency artifact |
| pérdida → política | 1 | 1.6 | 0.45 | 0.9014 | 0.9779 | frequency artifact |
| crisis → solutions | 1 | 1.1 | 0.45 | 0.9017 | 0.9779 | frequency artifact |
| risk → algorithms | 1 | 1.7 | 0.43 | 0.9090 | 0.9820 | frequency artifact |
| pobreza → política | 1 | 1.5 | 0.40 | 0.9251 | 0.9962 | frequency artifact |
| errors → architecture | 1 | 1.9 | 0.37 | 0.9334 | 0.9978 | frequency artifact |
| error → educación | 1 | 1.2 | 1.88 | 0.9358 | 0.9978 | frequency artifact |
| debt → fixed | 1 | 1.0 | 0.78 | 0.9384 | 0.9978 | frequency artifact |
| crisis → educación | 1 | 1.7 | 0.71 | 0.9447 | 1.0000 | frequency artifact |
| error → policy | 1 | 2.3 | 0.34 | 0.9512 | 1.0000 | frequency artifact |
| failure → developed | 1 | 2.3 | 0.65 | 0.9571 | 1.0000 | frequency artifact |
| corrupción → desarrollo | 1 | 1.4 | 1.62 | 0.9593 | 1.0000 | frequency artifact |
| problems → framework | 1 | 2.2 | 0.91 | 0.9657 | 1.0000 | frequency artifact |
| error → cooperation | 1 | 1.3 | 0.63 | 0.9754 | 1.0000 | frequency artifact |
| conflicto → desarrollo | 1 | 1.6 | 1.30 | 0.9815 | 1.0000 | frequency artifact |
| crisis → reforma | 1 | 1.5 | 1.27 | 0.9828 | 1.0000 | frequency artifact |
| problem → education | 1 | 3.2 | 0.23 | 0.9889 | 1.0000 | frequency artifact |
| problema → educación | 1 | 2.4 | 0.44 | 0.9925 | 1.0000 | frequency artifact |
| debt → reduced | 1 | 1.1 | 1.55 | 0.9964 | 1.0000 | frequency artifact |
| loss → algorithm | 1 | 2.6 | 0.37 | 0.9964 | 1.0000 | frequency artifact |
| error → architecture | 1 | 3.4 | 0.16 | 0.9981 | 1.0000 | frequency artifact |
| debt → developed | 1 | 1.4 | 0.84 | 0.9984 | 1.0000 | frequency artifact |
| problema → algoritmo | 1 | 1.0 | 2.95 | 0.9988 | 1.0000 | frequency artifact |
| conflict → cooperation | 1 | 1.0 | 0.86 | 1.0000 | 1.0000 | frequency artifact |

## 3. Pairs that survive the test

| Pair | Sources | Expected | Window lift | q |
|---|---|---|---|---|
| problem → solution | 18 | 8.5 | 6.64 | 7.8e-7 |
| problems → solutions | 11 | 3.2 | 6.00 | 7.2e-5 |
| problema → solución | 7 | 3.0 | 1.79 | 0.0009 |
| problem → solutions | 12 | 4.9 | 5.13 | 0.0010 |
| problems → algorithms | 9 | 4.8 | 2.25 | 0.0102 |

## 4. Can this graph ever contain a feedback loop?

No. Every edge goes from a tension word to a resolution word, and the two lists share no word. A graph whose edges only run from set T to a disjoint set R is bipartite and directed one way, so it is acyclic **by construction** — whatever the sources say. Observing 'zero loops' is therefore not evidence about the sources.

<details><summary>Sources confirming the top pair (problem → solution)</summary>

C_Book_2nd, Complexity_-_A_Guided_Tour, Computer-Networks---A-Tanenbaum---5th-edition, GEBen, Guns, Germs, and Steel The Fates of Human Societies - Jared Diamond, J Clark Scott - But How Do It Know_ - The Basic Principles of Computers for Everyone-John C. Scott (2009), Kuhn-SSR-2ndEd, Meadows-2008.-Thinking-in-Systems, No-Starch-Press-The-Rust, Operating-Systems-Design-and-Implementation-3rd-Edition, The Master Algorithm_ How the Quest for the Ultimate Learning Machine Will Remake Our World - Pedro Domingos - Basic Books (2015), The Pragmatic Programmer Your Journey to Mastery, 20th Anniversary Edition by Andrew Hunt David Hurst Thomas, Trump_ The Art of the Deal, Why-Nations-Fail_-The-Origins-o-Daron-Acemoglu, __Debt__The_First_5_000_Years, code-charles-petzold, designing-data-intensive-applications-the-big-ideas-behind-reliable-scalable-and-maintainable-systems, los-origenes-del-orden-politico-9788423424979

</details>
