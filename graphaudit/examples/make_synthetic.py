import random, csv, json
random.seed(7)
words = ["market","failure","network","policy","algorithm","regulation","trust","cooperation","entropy","feedback",
 "memory","protocol","consensus","replica","inflation","debt","risk","model","signal","gene","protein","energy",
 "language","grammar","proof","system","agent","reward","error","cache","index","query","value","price","power",
 "state","border","river","crop","virus","vaccine","contract","credit","bank","loan","tax","labor","wage","city","law"]
concepts = []
for i in range(300):
    a, b = random.sample(words, 2)
    concepts.append(f"{a} {b}")
concepts = sorted(set(concepts))
def variant(c, k):
    w = c.split()
    if k == 0: return c
    if k == 1: return c.title()
    if k == 2: return "the " + c
    if k == 3: return w[0] + " " + w[1] + "s"
    if k == 4: return c.replace(" ", "-").upper()
    return c
truth_variants = {}
for c in concepts:
    n = random.choice([1,1,1,2,2,3,4])
    truth_variants[c] = [variant(c, k) for k in random.sample(range(5), n)]
# a few typo variants (likely duplicates)
typos = {}
for c in random.sample(concepts, 15):
    i = random.randrange(1, len(c)-1)
    if c[i] != " ":
        t = c[:i] + c[i+1:]
        typos[c] = t
        truth_variants[c].append(t)
# zipf popularity
pop = {c: 1.0/(r+1)**0.9 for r, c in enumerate(random.sample(concepts, len(concepts)))}
planted = random.sample([c for c in concepts if pop[c] < 0.01], 20)
planted_pairs = [(planted[2*i], planted[2*i+1]) for i in range(10)]
D = 400
rows = []
cl = list(pop.keys()); wts = [pop[c] for c in cl]
for d in range(D):
    ment = set(random.choices(cl, weights=wts, k=random.randint(4, 12)))
    ment = list(ment)
    for _ in range(random.randint(2, 6)):
        if len(ment) < 2: break
        a, b = random.sample(ment, 2)
        rows.append((random.choice(truth_variants[a]), "related_to", random.choice(truth_variants[b]), f"doc{d}"))
for a, b in planted_pairs:
    for d in random.sample(range(D), 12):
        rows.append((random.choice(truth_variants[a]), "causes", random.choice(truth_variants[b]), f"doc{d}"))
random.shuffle(rows)
with open("edges.csv", "w", newline="") as f:
    w = csv.writer(f); w.writerow(["subject","predicate","object","source_doc"]); w.writerows(rows)
n_surface = len({x for r in rows for x in (r[0], r[2])})
used = {c for c in concepts if any(v in {x for r in rows for x in (r[0], r[2])} for v in truth_variants[c])}
json.dump({"typo_map": typos, "surface_nodes": n_surface, "true_concepts_used": len(used), "typos": len(typos),
           "planted": [list(p) for p in planted_pairs]}, open("truth.json","w"), indent=1)
print(open("truth.json").read()[:400], len(rows), "rows")
