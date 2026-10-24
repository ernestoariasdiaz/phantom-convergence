# graphaudit report

Input: `examples/synthetic_edges.csv` · 1722 edge rows · columns: source=`subject`, target=`object`, relation=`predicate`, provenance=`source_doc`

## Summary

| Measure | As stored | After audit |
|---|---|---|
| Nodes | 453 | 277 certain unique (176 duplicates, 38.9%) · 262 if likely duplicates also merge |
| Edges | 1722 rows · 1463 distinct | 1339 after merging duplicate nodes (124 fewer, 8.5%) |
| Edges claimed by ≥ 2 documents | 145 | 11 beat chance (FDR q < 0.05) · 133 within what chance predicts |
| Directed cycles | – | 217 nodes in cycles |

## 1. Duplicate nodes

**176 of 453 nodes (38.9%) are certain duplicates**: identical after lower-casing, removing accents, punctuation, leading articles and simple plurals. 15 further pairs are the same name up to a typo (1 character edit for names up to 15 characters, 2 up to 30); the same words in another order, or different numbers, are never paired.

| Copies | Written as |
|---|---|
| 4 | `contract credit`, `the contract credit`, `CONTRACT-CREDIT`, `Contract Credit` |
| 4 | `ALGORITHM-REPLICA`, `algorithm replica`, `algorithm replicas`, `the algorithm replica` |
| 4 | `model policy`, `MODEL-POLICY`, `model policys`, `Model Policy` |
| 4 | `the consensus loan`, `Consensus Loan`, `consensus loans`, `consensus loan` |
| 4 | `the value energy`, `Value Energy`, `value energys`, `value energy` |
| 4 | `the value entropy`, `Value Entropy`, `value entropys`, `value entropy` |
| 4 | `ALGORITHM-BORDER`, `algorithm border`, `Algorithm Border`, `algorithm borders` |
| 4 | `Law Inflation`, `the law inflation`, `LAW-INFLATION`, `law inflation` |
| 4 | `the power trust`, `power trusts`, `POWER-TRUST`, `Power Trust` |
| 4 | `model failures`, `the model failure`, `model failure`, `Model Failure` |
| 4 | `error tax`, `ERROR-TAX`, `error taxs`, `the error tax` |
| 3 | `the algorithm risk`, `algorithm risk`, `Algorithm Risk` |
| 3 | `cooperation systems`, `the cooperation system`, `cooperation system` |
| 3 | `loan genes`, `LOAN-GENE`, `loan gene` |
| 3 | `cooperation query`, `COOPERATION-QUERY`, `the cooperation query` |

Likely duplicates (review by hand):

| Label A | Label B | Similarity |
|---|---|---|
| INFLATION-CONSENSUS | inflation consensuss | 0.95 |
| cotract feedback | Contract Feedback | 0.94 |
| protocol feedback | protocol feedbak | 0.94 |
| Index Consensus | index consensuss | 0.94 |
| SYSTEM-CONTRACT | system cotract | 0.93 |
| DEBT-PROTOCOL | debt protcol | 0.92 |
| policy virus | policy viruss | 0.92 |
| replic price | the replica price | 0.92 |
| credit river | credit iver | 0.92 |
| Index Virus | index viruss | 0.92 |
| sate reward | state reward | 0.92 |
| feeback tax | the feedback tax | 0.92 |
| sate agent | State Agent | 0.91 |
| trust stte | TRUST-STATE | 0.91 |
| risk systm | risk system | 0.91 |

Same meaning in different words ("car" / "automobile") is not detected here: that needs an embedding model.

## 2. Edge inflation

1722 edge rows hold 1463 distinct edges; merging duplicate nodes leaves **1339**. 0 edges became self-loops (a node linked to a copy of itself).

Edges stored most often:

| Edge | Rows |
|---|---|
| Index Contract → cache index (causes) | 12 |
| Power Cooperation → credit contract (causes) | 12 |
| language crop → the memory risk (causes) | 12 |
| COOPERATION-PROOF → vaccine rivers (causes) | 12 |
| the system trust → reward algorithms (causes) | 12 |
| value consensus → VACCINE-CONSENSUS (causes) | 12 |
| Reward Consensus → the error trust (causes) | 12 |
| crop markets → TRUST-REGULATION (causes) | 12 |
| contract credit → entropy wage (related_to) | 10 |
| entropy wage → contract credit (related_to) | 10 |

## 3. Source support vs. chance

400 documents. For each edge stored from ≥ 2 documents: given that it is found once (which puts both entities in that document by construction), and that its entities appear in *a* and *b* of the *D* documents, independent placement predicts *1 + (a−1)(b−1)/(D−1)* documents (exact hypergeometric test; Benjamini–Hochberg across all 1339 edges with provenance, so picking the multi-source ones does not inflate significance). An edge whose support is close to that number is what two frequent entities produce anyway.

**11 of 145 multi-source edges beat chance; 133 are within what chance predicts.**

| Edge | Documents | Expected by chance | q | Verdict |
|---|---|---|---|---|
| Reward Consensus → the error trust (causes) | 12 | 1.3 | 1.8e-17 | beyond chance |
| the system trust → reward algorithms (causes) | 12 | 1.4 | 5.9e-17 | beyond chance |
| crop markets → TRUST-REGULATION (causes) | 12 | 1.4 | 4.7e-16 | beyond chance |
| COOPERATION-PROOF → vaccine rivers (causes) | 12 | 1.4 | 5.1e-16 | beyond chance |
| Power Cooperation → credit contract (causes) | 12 | 1.4 | 1.3e-15 | beyond chance |
| value consensus → VACCINE-CONSENSUS (causes) | 12 | 1.4 | 1.5e-15 | beyond chance |
| language crop → the memory risk (causes) | 12 | 1.5 | 1.1e-14 | beyond chance |
| Index Contract → cache index (causes) | 12 | 1.5 | 1.4e-14 | beyond chance |
| the replica query → Index Virus (causes) | 9 | 1.3 | 5.7e-10 | beyond chance |
| INFLATION-CONSENSUS → Labor Contract (causes) | 8 | 1.3 | 5.1e-9 | beyond chance |
| contract credit → entropy wage (related_to) | 8 | 34.9 | 1.0000 | frequency artifact |
| entropy wage → the bank wage (related_to) | 7 | 42.8 | 1.0000 | frequency artifact |
| entropy wage → the debt reward (related_to) | 6 | 31.3 | 1.0000 | frequency artifact |
| entropy wage → contract credit (related_to) | 6 | 34.9 | 1.0000 | frequency artifact |
| the bank wage → entropy wage (related_to) | 6 | 42.8 | 1.0000 | frequency artifact |

## 4. Structure

- Directed cycles: 217 nodes sit on a cycle.
- Degree concentration: the top 1% of nodes (3) hold 14% of all edge endpoints.

Files: `duplicates.csv`, `likely_duplicates.csv`, `edge_support.csv`.
