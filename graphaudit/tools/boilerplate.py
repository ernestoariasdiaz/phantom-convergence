"""Find boilerplate (letterheads, footers, licence text) in a GraphRAG index and recount each relationship's
sources without it.

A line that repeats in many text units is boilerplate. For every relationship and every text unit it cites:
if both entity names occur in the unit, but only together inside boilerplate lines, the unit stops counting as
a source. Units where a name does not occur literally (the model paraphrased it) cannot be judged and are kept. This approximates "what if the boilerplate had been removed
before indexing" without re-indexing; a real re-index could also change what gets extracted.

usage:
  python boilerplate.py <GraphRAG output folder> <relationships.csv from graphrag_to_csv.py> <out_folder> [--min-share 0.05]
then:
  graphaudit out_folder/relationships.csv --nodes <entities.csv> --id title --label title --doc text_unit_ids --undirected
Also writes out_folder/boilerplate_lines.txt (the repeated lines found, with how many units contain each).

  python boilerplate.py <GraphRAG output folder> --find "AURELIA"
prints every text unit that mentions a word, to check an entity against its source.

Requires: pip install pandas pyarrow
"""
import collections
import csv
import os
import re
import sys
import unicodedata
from pathlib import Path

import pandas as pd

csv.field_size_limit(10**9)


def fold(s):
    s = unicodedata.normalize("NFKD", str(s).lower())
    s = "".join(c for c in s if not unicodedata.combining(c))
    return re.sub(r"\s+", " ", re.sub(r"[^\w#]", " ", s)).strip()


def units(src):
    p = Path(src)
    for name in ("text_units.parquet", "create_final_text_units.parquet"):
        if (p / name).exists():
            t = pd.read_parquet(p / name)
            return dict(zip(t["id"].astype(str), t["text"].astype(str)))
    sys.exit(f"no text_units.parquet in {src}")


def main():
    args = sys.argv[1:]
    if len(args) >= 3 and args[1] == "--find":
        word = fold(args[2])
        hits = [(k, v) for k, v in units(args[0]).items() if word in fold(v)]
        print(f"{len(hits)} text units mention '{args[2]}'")
        for k, v in hits[:10]:
            i = fold(v).find(word)
            flat = re.sub(r"\s+", " ", v)
            print(f"\n--- {k}\n{flat[max(0, i - 400):i + 400]}")
        return
    if len(args) < 3:
        sys.exit(__doc__)
    src, rel_csv, out = args[0], args[1], args[2]
    share = float(args[args.index("--min-share") + 1]) if "--min-share" in args else 0.05
    tu = units(src)
    # boilerplate: lines (folded) present in at least `share` of text units, and in at least 3
    count = collections.Counter()
    lines_of = {}
    for k, v in tu.items():
        # compare lines without spaces: OCR splits and joins words ("superdebancos rd" / "superdebancosrd")
        ls = {fold(l).replace(" ", "") for l in v.splitlines() if len(fold(l)) >= 4}
        lines_of[k] = ls
        count.update(ls)
    need = max(3, int(share * len(tu)))
    boiler = {l for l, c in count.items() if c >= need}
    # the text itself is kept with spaces for name matching; a line is dropped if its spaceless form is boilerplate
    clean, full = {}, {}
    for k, v in tu.items():
        keep = [fold(l) for l in v.splitlines() if fold(l).replace(" ", "") not in boiler]
        clean[k] = " " + " ".join(keep) + " "
        full[k] = " " + fold(v) + " "
    os.makedirs(out, exist_ok=True)
    with open(os.path.join(out, "boilerplate_lines.txt"), "w", encoding="utf-8") as f:
        for l in sorted(boiler, key=lambda x: -count[x]):
            f.write(f"{count[l]}\t{l}\n")
    rows = list(csv.DictReader(open(rel_csv, encoding="utf-8")))
    kept_units = dropped_units = dropped_edges = unmatched = 0
    for r in rows:
        a, b = " " + fold(r["source"]) + " ", " " + fold(r["target"]) + " "
        ids = [x for x in r.get("text_unit_ids", "").split(";") if x]
        keep = []
        for x in ids:
            if x not in full:
                keep.append(x)  # text unknown: keep as it was
            elif not (a in full[x] and b in full[x]):
                keep.append(x)  # a name was paraphrased by the model: cannot judge, keep
                unmatched += 1
            elif a in clean[x] and b in clean[x]:
                keep.append(x)  # both names occur outside the boilerplate
            # else: the pair only meets inside boilerplate lines -> dropped
        dropped_units += len(ids) - len(keep)
        kept_units += len(keep)
        if ids and not keep:
            dropped_edges += 1
        r["text_unit_ids"] = ";".join(keep)
    with open(os.path.join(out, "relationships.csv"), "w", newline="", encoding="utf-8") as f:
        w = csv.DictWriter(f, fieldnames=list(rows[0].keys()))
        w.writeheader()
        w.writerows(rows)
    print(f"{len(tu)} text units · {len(boiler)} boilerplate lines (in >= {need} units each), listed in boilerplate_lines.txt")
    print(f"relationship sources: {kept_units} kept, {dropped_units} dropped (the pair only meets inside boilerplate lines)")
    print(f"  of the kept, {unmatched} could not be checked because a name does not occur literally in the unit (kept as they were)")
    print(f"{dropped_edges} relationships keep no source at all; graphaudit will report them as having no provenance")


if __name__ == "__main__":
    main()
