"""Convert GraphRAG output (entities / relationships parquet) to CSV for graphaudit.

usage:
  python graphrag_to_csv.py <folder with entities.parquet and relationships.parquet> [out_folder]
  python graphrag_to_csv.py --example [out_folder]   # downloads Microsoft's "Operation Dulce" example

Requires: pip install pandas pyarrow
Writes entities.csv and relationships.csv. Then:
  graphaudit relationships.csv --nodes entities.csv --id title --label title --doc text_unit_ids
"""
import sys
import urllib.request
from pathlib import Path

import pandas as pd

EXAMPLE = ("https://raw.githubusercontent.com/microsoft/graphrag/main/"
           "docs/examples_notebooks/inputs/operation%20dulce/")


def as_list(v):
    if v is None:
        return []
    try:
        return [str(x) for x in list(v)]
    except TypeError:
        return [str(v)]


def main():
    args = [a for a in sys.argv[1:] if a != "--example"]
    if "--example" in sys.argv:
        src = Path("graphrag_example")
        src.mkdir(exist_ok=True)
        for name in ("entities.parquet", "relationships.parquet"):
            print("downloading", name)
            urllib.request.urlretrieve(EXAMPLE + name, src / name)
        out = Path(args[0]) if args else src
    else:
        if not args:
            sys.exit(__doc__)
        src = Path(args[0])
        out = Path(args[1]) if len(args) > 1 else src
    out.mkdir(parents=True, exist_ok=True)

    ent = pd.read_parquet(src / "entities.parquet")
    rel = pd.read_parquet(src / "relationships.parquet")
    print("entities columns:", list(ent.columns))
    print("relationships columns:", list(rel.columns))

    keep_e = [c for c in ("id", "title", "type", "frequency", "degree") if c in ent.columns]
    e = ent[keep_e].copy()
    if "text_unit_ids" in ent.columns:
        e["text_unit_ids"] = ent["text_unit_ids"].map(lambda v: ";".join(as_list(v)))
    e.to_csv(out / "entities.csv", index=False)

    keep_r = [c for c in ("source", "target", "weight", "combined_degree") if c in rel.columns]
    r = rel[keep_r].copy()
    if "description" in rel.columns:
        r["description"] = rel["description"].astype(str).str.slice(0, 200)
    if "text_unit_ids" in rel.columns:
        r["text_unit_ids"] = rel["text_unit_ids"].map(lambda v: ";".join(as_list(v)))
    r.to_csv(out / "relationships.csv", index=False)
    print(f"wrote {len(e)} entities and {len(r)} relationships to {out.resolve()}")


if __name__ == "__main__":
    main()
