"""Compare the entity tables of two graphs built from the same text (two runs, two settings, two tools).

Names are compared after the same light normalisation graphaudit uses for certain duplicates (case,
punctuation, leading articles). A low overlap means which entities "exist" depends on how the graph was built.
Standard library only.

usage: python compare_entities.py a/entities.csv b/entities.csv [--label title] [--type type]
"""
import csv
import re
import sys
import unicodedata

csv.field_size_limit(10**9)


def norm(s):
    s = "".join(c for c in unicodedata.normalize("NFKD", s.upper()) if not unicodedata.combining(c))
    t = re.sub(r"[^\w ]", " ", s).split()
    while len(t) > 1 and t[0] in ("THE", "A", "AN", "EL", "LA", "LOS", "LAS"):
        t = t[1:]
    return " ".join(t)


def load(path, label, kind):
    rows = list(csv.DictReader(open(path, encoding="utf-8-sig")))
    return {(norm(r[label]), (r.get(kind) or "").strip('"<> ').upper()) for r in rows if r.get(label)}


def main():
    args = [a for a in sys.argv[1:]]
    if len(args) < 2:
        sys.exit(__doc__)
    label = args[args.index("--label") + 1] if "--label" in args else "title"
    kind = args[args.index("--type") + 1] if "--type" in args else "type"
    a, b = load(args[0], label, kind), load(args[1], label, kind)
    na, nb = {n for n, _ in a}, {n for n, _ in b}
    print(f"entities: {len(na)} vs {len(nb)} · shared {len(na & nb)} · Jaccard {len(na & nb) / max(1, len(na | nb)):.2f}")
    for t in sorted({t for _, t in a} | {t for _, t in b}):
        ta, tb = {n for n, x in a if x == t}, {n for n, x in b if x == t}
        if len(ta) + len(tb) >= 10:
            print(f"  {t or '(no type)'}: {len(ta)} vs {len(tb)} · shared {len(ta & tb)}")


if __name__ == "__main__":
    main()
