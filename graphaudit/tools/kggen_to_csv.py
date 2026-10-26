"""Convert a kg-gen graph (JSON with entities, relations, *_chunk_ids) to CSV for graphaudit.

usage: python kggen_to_csv.py graph.json out_folder
Then: graphaudit out_folder/relationships.csv --nodes out_folder/entities.csv --id title --label title --doc doc
"""
import json,csv,sys,os
src,out=sys.argv[1],sys.argv[2]; os.makedirs(out,exist_ok=True)
d=json.load(open(src))
ec=d.get('entities_chunk_ids') or {}; rc=d.get('relations_chunk_ids') or {}
def ch(v): return ';'.join(sorted({f"{p.rsplit('/',1)[-1]}#{i}" for p,i in v}))
with open(out+'/entities.csv','w',newline='') as f:
    w=csv.writer(f); w.writerow(['title','doc'])
    for e in d['entities']: w.writerow([e,ch(ec.get(e,[]))])
with open(out+'/relationships.csv','w',newline='') as f:
    w=csv.writer(f); w.writerow(['source','relation','target','doc'])
    for s,r,t in d['relations']: w.writerow([s,r,t,ch(rc.get(f'{s}-{r}-{t}',[]))])
print(len(d['entities']),'entities',len(d['relations']),'relations', len({c for v in rc.values() for c in map(tuple,v)}),'chunks')
