"""Convert an ai-knowledge-graph output (JSON list of subject/predicate/object triples) to CSV for graphaudit.

github.com/robert-mcdermott/ai-knowledge-graph writes one JSON list per document. Extracted triples carry
`chunk` (and the source sentence); inferred triples carry `inferred: true` and `method`, and no chunk, so
graphaudit reports them as edges with no provenance. Standard library only.

usage:  python akg_to_csv.py graph.json out_folder
then:   graphaudit out_folder/relationships.csv --nodes out_folder/entities.csv --id title --label title --type type --doc doc
"""
import csv
import json
import os
import sys


def main():
    if len(sys.argv) < 3:
        sys.exit(__doc__)
    src, out = sys.argv[1], sys.argv[2]
    os.makedirs(out, exist_ok=True)
    triples = json.load(open(src, encoding="utf-8"))
    types = {}
    for t in triples:
        for k in ("subject", "object"):
            if not types.get(t[k]):
                types[t[k]] = t.get(k + "_type", "")
    with open(os.path.join(out, "entities.csv"), "w", newline="", encoding="utf-8") as f:
        w = csv.writer(f)
        w.writerow(["title", "type"])
        for name, kind in types.items():
            w.writerow([name, kind])
    with open(os.path.join(out, "relationships.csv"), "w", newline="", encoding="utf-8") as f:
        w = csv.writer(f)
        w.writerow(["source", "relation", "target", "doc", "method"])
        for t in triples:
            w.writerow([t["subject"], t["predicate"], t["object"],
                        f"chunk{t['chunk']}" if "chunk" in t else "", t.get("method", "")])
    inferred = sum(1 for t in triples if t.get("inferred"))
    print(f"wrote {len(types)} entities and {len(triples)} relationships ({inferred} inferred) to {out}")


if __name__ == "__main__":
    main()
