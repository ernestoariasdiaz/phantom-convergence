"""Convert an itext2kg (ATOM) knowledge-graph snapshot (.pkl) to CSV for graphaudit.

Pickles can run code when loaded, so this reader admits only itext2kg's model classes (replaced here by
inert containers) and NumPy arrays, and refuses anything else. Each relationship's atomic facts become its
provenance. Requires numpy.

usage:  python itext2kg_to_csv.py snapshot.pkl out_folder
then:   graphaudit out_folder/relationships.csv --nodes out_folder/entities.csv --id title --label title --type type --doc doc
"""
import csv
import hashlib
import os
import pickle
import sys

import numpy

MODELS = {("itext2kg.atom.models.knowledge_graph", "KnowledgeGraph"), ("itext2kg.atom.models.entity", "Entity"),
          ("itext2kg.atom.models.entity", "EntityProperties"), ("itext2kg.atom.models.relationship", "Relationship"),
          ("itext2kg.atom.models.relationship", "RelationshipProperties")}


class Inert:
    """Holds a pickled object's state; runs nothing."""
    def __init__(self, *args, **kwargs):
        pass

    def __setstate__(self, state):
        self.__dict__["state"] = state


def fields(o):
    st = o.__dict__.get("state") if isinstance(o, Inert) else o
    return st.get("__dict__", st) if isinstance(st, dict) else {}


class Restricted(pickle.Unpickler):
    def find_class(self, module, name):
        if (module, name) in MODELS:
            return type(name, (Inert,), {})
        if module in ("numpy._core.multiarray", "numpy.core.multiarray") and name == "_reconstruct":
            core = getattr(numpy, "_core", None) or numpy.core
            return core.multiarray._reconstruct
        if module == "numpy" and name in ("ndarray", "dtype"):
            return getattr(numpy, name)
        raise pickle.UnpicklingError(f"refused {module}.{name}")


def main():
    if len(sys.argv) < 3:
        sys.exit(__doc__)
    with open(sys.argv[1], "rb") as f:
        obj = Restricted(f).load()
    kg = obj.get("knowledge_graph", obj) if isinstance(obj, dict) else obj
    g = fields(kg)
    out = sys.argv[2]
    os.makedirs(out, exist_ok=True)
    with open(os.path.join(out, "entities.csv"), "w", newline="", encoding="utf-8") as f:
        w = csv.writer(f)
        w.writerow(["title", "type"])
        for e in g["entities"]:
            fe = fields(e)
            w.writerow([fe["name"], fe["label"]])
    facts = set()
    with open(os.path.join(out, "relationships.csv"), "w", newline="", encoding="utf-8") as f:
        w = csv.writer(f)
        w.writerow(["source", "relation", "target", "doc"])
        for r in g["relationships"]:
            fr = fields(r)
            fa = fields(fr["properties"]).get("atomic_facts") or []
            facts.update(fa)
            ids = ";".join(hashlib.sha1(x.encode()).hexdigest()[:10] for x in fa)
            w.writerow([fields(fr["startEntity"])["name"], fr["name"], fields(fr["endEntity"])["name"], ids])
    print(f"wrote {len(g['entities'])} entities and {len(g['relationships'])} relationships ({len(facts)} atomic facts) to {out}")


if __name__ == "__main__":
    main()
