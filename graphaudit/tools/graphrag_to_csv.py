"""Convert GraphRAG output (entities / relationships parquet) to CSV for graphaudit.

Works with current GraphRAG output (entities.parquet, relationships.parquet) and with
older versions (create_final_entities.parquet, create_final_relationships.parquet,
where the entity name column is `name` instead of `title`).

usage:
  python graphrag_to_csv.py <folder with the parquet files> [out_folder]
  python graphrag_to_csv.py --example [out_folder]   # downloads Microsoft's "Operation Dulce" example

Requires: pip install pandas pyarrow
Writes entities.csv and relationships.csv (entity names in column `title`). Then:
  graphaudit relationships.csv --nodes entities.csv --id title --label title --doc text_unit_ids --undirected
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


def find(src: Path, *names):
    for n in names:
        if (src / n).exists():
            return src / n
    sys.exit(f"none of {names} found in {src}")


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

    ent = pd.read_parquet(find(src, "entities.parquet", "create_final_entities.parquet"))
    rel = pd.read_parquet(find(src, "relationships.parquet", "create_final_relationships.parquet"))
    print("entities columns:", list(ent.columns))
    print("relationships columns:", list(rel.columns))

    e = pd.DataFrame()
    name_col = "title" if "title" in ent.columns else "name"
    e["title"] = ent[name_col].astype(str)
    for c in ("id", "type", "frequency", "degree"):
        if c in ent.columns:
            e[c] = ent[c]
    if "description" in ent.columns:
        e["description"] = ent["description"].astype(str).str.slice(0, 400)
    if "text_unit_ids" in ent.columns:
        e["text_unit_ids"] = ent["text_unit_ids"].map(lambda v: ";".join(as_list(v)))
    e.to_csv(out / "entities.csv", index=False)

    r = pd.DataFrame()
    r["source"] = rel["source"].astype(str)
    r["target"] = rel["target"].astype(str)
    for c in ("weight", "combined_degree", "rank"):
        if c in rel.columns:
            r[c] = rel[c]
    if "description" in rel.columns:
        r["description"] = rel["description"].astype(str).str.slice(0, 200)
    if "text_unit_ids" in rel.columns:
        r["text_unit_ids"] = rel["text_unit_ids"].map(lambda v: ";".join(as_list(v)))
    r.to_csv(out / "relationships.csv", index=False)
    print(f"wrote {len(e)} entities and {len(r)} relationships to {out.resolve()}")


if __name__ == "__main__":
    main()
