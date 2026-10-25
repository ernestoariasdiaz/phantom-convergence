"""Convert a LightRAG graph (graph_chunk_entity_relation.graphml) to CSV for graphaudit.

LightRAG keeps its knowledge graph in rag_storage/graph_chunk_entity_relation.graphml.
Each node carries entity_id, entity_type and source_id (chunk ids joined by <SEP>);
each edge carries source_id the same way.  Standard library only.

usage:
  python lightrag_to_csv.py <path to graph_chunk_entity_relation.graphml or rag_storage folder> [out_folder]

Writes entities.csv and relationships.csv. Then:
  graphaudit relationships.csv --nodes entities.csv --id title --label title --type type --doc source_id --undirected
"""
import csv
import sys
import xml.etree.ElementTree as ET
from pathlib import Path

NS = "{http://graphml.graphdrawing.org/xmlns}"


def main():
    if len(sys.argv) < 2:
        sys.exit(__doc__)
    src = Path(sys.argv[1])
    if src.is_dir():
        src = src / "graph_chunk_entity_relation.graphml"
    out = Path(sys.argv[2]) if len(sys.argv) > 2 else src.parent
    out.mkdir(parents=True, exist_ok=True)

    root = ET.parse(src).getroot()
    keys = {k.get("id"): (k.get("for"), k.get("attr.name")) for k in root.iter(NS + "key")}
    graph = root.find(NS + "graph")

    def attrs(el):
        return {keys.get(d.get("key"), (None, d.get("key")))[1]: (d.text or "") for d in el.findall(NS + "data")}

    def chunks(v):
        return ";".join(c.strip() for c in v.split("<SEP>") if c.strip())

    n = e = 0
    with open(out / "entities.csv", "w", newline="", encoding="utf-8") as f:
        w = csv.writer(f)
        w.writerow(["title", "type", "source_id", "file_path"])
        for node in graph.iter(NS + "node"):
            a = attrs(node)
            w.writerow([a.get("entity_id", node.get("id")), a.get("entity_type", ""),
                        chunks(a.get("source_id", "")), a.get("file_path", "")])
            n += 1
    with open(out / "relationships.csv", "w", newline="", encoding="utf-8") as f:
        w = csv.writer(f)
        w.writerow(["source", "target", "keywords", "weight", "source_id", "description"])
        for edge in graph.iter(NS + "edge"):
            a = attrs(edge)
            w.writerow([edge.get("source"), edge.get("target"), a.get("keywords", ""), a.get("weight", ""),
                        chunks(a.get("source_id", "")), a.get("description", "")[:200]])
            e += 1
    print(f"wrote {n} entities and {e} relationships to {out.resolve()}")


if __name__ == "__main__":
    main()
