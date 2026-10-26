//! graphaudit — check a knowledge graph's own numbers before believing them.
//!
//! 1. Duplicate nodes: how many "distinct" nodes are the same thing written differently.
//! 2. Edge inflation: how many edges collapse once duplicates are merged.
//! 3. Source support vs chance: for edges claimed by several documents, how many
//!    documents would mention both entities by chance alone.
//! 4. Structure: self-loops, degree concentration, and cycles that are impossible
//!    by construction (a graph whose sources and targets never overlap cannot loop).
//!
//! Input: an edge CSV (+ optional node CSV). Std-only Rust.

mod csv;
mod dedup;
mod embed;
mod http;
mod json;
mod judge;
mod stats;

use std::collections::{BTreeMap, HashMap, HashSet};
use std::env;
use std::fs;
use std::path::PathBuf;

const SRC: &[&str] = &["source", "subject", "head", "from", "src", "start_id", "start", "source_node", "node1", "entity1"];
const DST: &[&str] = &["target", "object", "tail", "to", "dst", "end_id", "end", "target_node", "node2", "entity2"];
const REL: &[&str] = &["relation", "predicate", "rel", "relationship", "type", "label", "edge_type", "relation_type"];
const DOC: &[&str] = &["doc", "document", "doc_id", "document_id", "source_doc", "provenance", "text_unit_ids", "chunk_id", "chunk", "sources", "evidence", "file", "book", "reference"];
const NID: &[&str] = &["id", "node_id", "entity_id", "uuid", "key"];
const NLABEL: &[&str] = &["label", "name", "title", "entity", "text", "value", "node"];
const NTYPE: &[&str] = &["type", "entity_type", "category", "kind", "class", "node_type"];

struct Args {
    edges: PathBuf,
    nodes: Option<PathBuf>,
    src: Option<String>,
    dst: Option<String>,
    rel: Option<String>,
    doc: Option<String>,
    nid: Option<String>,
    nlabel: Option<String>,
    ntype: Option<String>,
    undirected: bool,
    fuzzy: f64,
    alpha: f64,
    top: usize,
    out: PathBuf,
    embed: Option<String>,
    embed_min: f32,
    desc: Option<String>,
    judge: Option<String>,
    judge_max: usize,
    judge_all: bool,
}

const HELP: &str = "graphaudit — audit a knowledge graph's own numbers

USAGE: graphaudit <edges.csv> [options]

  --nodes FILE       node table (id + label); otherwise node names come from the edges
  --src/--dst COL    edge endpoint columns   (auto: source/subject/head/from, target/object/tail/to)
  --rel COL          relation column          (auto: relation/predicate/type)
  --doc COL          provenance column        (auto: doc/document/source_doc/text_unit_ids/chunk_id…)
                     multi-valued cells like 'a;b' or \"['a','b']\" are split
  --id/--label COL   node-table columns      (auto: id/node_id, label/name/title)
  --type COL         node-table type column   (auto: type/entity_type/category); partial-name
                     duplicates are only proposed between nodes of the same type
  --undirected       treat A→B and B→A as the same edge (GraphRAG builds undirected graphs)
  --typos S          edit budget for 'likely duplicates' (default 1 = 1 edit up to 15 chars,
                     2 up to 30, 3 beyond; 0 = off; 2 = twice as loose)
  --alpha A          FDR level for source support (default 0.05)
  --top N            rows to show per table (default 15)
  --out DIR          output folder (default graphaudit_out)
  --embed URL        experimental: list same-meaning candidates with an embedding server, e.g.
                     http://127.0.0.1:8081 (llama-server -m bge-m3 --embeddings)
  --embed-min C      cosine similarity to propose a pair (default 0.90)
  --desc COL         node-table description column, used by --embed and --judge
  --judge URL        ask an instruction model (llama-server, e.g. http://127.0.0.1:8080) whether
                     candidate pairs are the same entity, reading names, types and descriptions;
                     each pair is asked in both orders and counts only if both say SAME
  --judge-max N      most pairs to judge (default 300)
  --judge-all        in a typed graph, also judge abstract types (CONCEPT, METHOD…); by default only
                     named types are judged (PERSON, ORGANIZATION, GEO, EVENT…), where small models are reliable

Writes DIR/report.md, duplicates.csv, likely_duplicates.csv, partial_names.csv, edge_support.csv
(and semantic_duplicates.csv with --embed, judged_pairs.csv with --judge).";

fn die(m: &str) -> ! {
    eprintln!("graphaudit: {m}");
    std::process::exit(2)
}

fn parse_args() -> Args {
    let mut a = Args {
        edges: PathBuf::new(),
        nodes: None,
        src: None,
        dst: None,
        rel: None,
        doc: None,
        nid: None,
        nlabel: None,
        ntype: None,
        undirected: false,
        fuzzy: 1.0,
        alpha: 0.05,
        top: 15,
        out: PathBuf::from("graphaudit_out"),
        embed: None,
        embed_min: 0.90,
        desc: None,
        judge: None,
        judge_max: 300,
        judge_all: false,
    };
    let mut it = env::args().skip(1);
    let mut pos = None;
    while let Some(x) = it.next() {
        let mut v = |n: &str| it.next().unwrap_or_else(|| die(&format!("{n} needs a value")));
        match x.as_str() {
            "--nodes" => a.nodes = Some(v("--nodes").into()),
            "--src" => a.src = Some(v("--src")),
            "--dst" => a.dst = Some(v("--dst")),
            "--rel" => a.rel = Some(v("--rel")),
            "--doc" => a.doc = Some(v("--doc")),
            "--id" => a.nid = Some(v("--id")),
            "--label" => a.nlabel = Some(v("--label")),
            "--type" => a.ntype = Some(v("--type")),
            "--undirected" => a.undirected = true,
            "--typos" => a.fuzzy = v("--typos").parse().unwrap_or_else(|_| die("bad --typos")),
            "--alpha" => a.alpha = v("--alpha").parse().unwrap_or_else(|_| die("bad --alpha")),
            "--top" => a.top = v("--top").parse().unwrap_or_else(|_| die("bad --top")),
            "--out" => a.out = v("--out").into(),
            "--embed" => a.embed = Some(v("--embed")),
            "--embed-min" => a.embed_min = v("--embed-min").parse().unwrap_or_else(|_| die("bad --embed-min")),
            "--desc" => a.desc = Some(v("--desc")),
            "--judge" => a.judge = Some(v("--judge")),
            "--judge-all" => a.judge_all = true,
            "--judge-max" => a.judge_max = v("--judge-max").parse().unwrap_or_else(|_| die("bad --judge-max")),
            "-h" | "--help" => {
                println!("{HELP}");
                std::process::exit(0)
            }
            s if s.starts_with("--") => die(&format!("unknown option {s} (see --help)")),
            s => pos = Some(PathBuf::from(s)),
        }
    }
    a.edges = pos.unwrap_or_else(|| {
        println!("{HELP}");
        std::process::exit(2)
    });
    a
}

fn read_table(p: &PathBuf) -> csv::Table {
    let bytes = fs::read(p).unwrap_or_else(|e| die(&format!("cannot read {}: {e}", p.display())));
    csv::parse(&String::from_utf8_lossy(&bytes)).unwrap_or_else(|e| die(&format!("{}: {e}", p.display())))
}

/// Number of nodes that lie on at least one directed cycle: members of strongly
/// connected components with more than one node (Kosaraju, iterative).
fn nodes_on_cycles(out_n: &HashMap<usize, HashSet<usize>>) -> usize {
    let mut nodes: HashSet<usize> = out_n.keys().cloned().collect();
    for vs in out_n.values() {
        nodes.extend(vs.iter().cloned());
    }
    let mut rev: HashMap<usize, Vec<usize>> = HashMap::new();
    for (&u, vs) in out_n {
        for &v in vs {
            rev.entry(v).or_default().push(u);
        }
    }
    let empty = HashSet::new();
    let mut order = Vec::with_capacity(nodes.len());
    let mut seen: HashSet<usize> = HashSet::new();
    let mut sorted: Vec<usize> = nodes.iter().cloned().collect();
    sorted.sort_unstable();
    for &start in &sorted {
        if !seen.insert(start) {
            continue;
        }
        let mut stack: Vec<(usize, Vec<usize>)> = vec![(start, out_n.get(&start).unwrap_or(&empty).iter().cloned().collect())];
        while let Some((node, children)) = stack.last_mut() {
            if let Some(c) = children.pop() {
                if seen.insert(c) {
                    let next: Vec<usize> = out_n.get(&c).unwrap_or(&empty).iter().cloned().collect();
                    stack.push((c, next));
                }
            } else {
                order.push(*node);
                stack.pop();
            }
        }
    }
    let mut comp: HashSet<usize> = HashSet::new();
    let mut on_cycle = 0usize;
    for &start in order.iter().rev() {
        if !comp.insert(start) {
            continue;
        }
        let mut size = 0usize;
        let mut stack = vec![start];
        while let Some(x) = stack.pop() {
            size += 1;
            for &y in rev.get(&x).map(|v| v.as_slice()).unwrap_or(&[]) {
                if comp.insert(y) {
                    stack.push(y);
                }
            }
        }
        if size > 1 {
            on_cycle += size;
        }
    }
    on_cycle
}

fn csv_cell(s: &str) -> String {
    if s.contains([',', '"', '\n', ';']) {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

fn md(s: &str) -> String {
    let s = s.replace('|', "\\|").replace('\n', " ");
    if s.chars().count() > 60 {
        format!("{}…", s.chars().take(59).collect::<String>())
    } else {
        s
    }
}

fn fmt_p(p: f64) -> String {
    if p < 1e-4 { format!("{p:.1e}") } else { format!("{p:.4}") }
}

fn pct(a: usize, b: usize) -> String {
    if b == 0 { "–".into() } else { format!("{:.1}%", 100.0 * a as f64 / b as f64) }
}

fn main() {
    let a = parse_args();
    let et = read_table(&a.edges);
    let need = |c: Option<usize>, what: &str, flag: &str| {
        c.unwrap_or_else(|| die(&format!("no {what} column found in {:?}; name it with {flag}", et.header)))
    };
    let ci_src = need(et.col(a.src.as_deref(), SRC), "source", "--src");
    let ci_dst = need(et.col(a.dst.as_deref(), DST), "target", "--dst");
    let ci_rel = et.col(a.rel.as_deref(), REL).filter(|&i| i != ci_src && i != ci_dst);
    let ci_doc = et.col(a.doc.as_deref(), DOC).filter(|&i| i != ci_src && i != ci_dst);

    // ---- node universe: id -> label
    let mut ids: Vec<String> = Vec::new();
    let mut labels: Vec<String> = Vec::new();
    let mut idx: HashMap<String, usize> = HashMap::new();
    let mut types: Vec<String> = Vec::new();
    let mut descs: HashMap<usize, String> = HashMap::new();
    let mut declared = 0usize;
    if let Some(np) = &a.nodes {
        let nt = read_table(np);
        let i_id = nt.col(a.nid.as_deref(), NID).unwrap_or_else(|| die(&format!("no id column in node file {:?}; use --id", nt.header)));
        let i_lab = nt.col(a.nlabel.as_deref(), NLABEL).filter(|&i| i != i_id).unwrap_or(i_id);
        let i_typ = nt.col(a.ntype.as_deref(), NTYPE).filter(|&i| i != i_id && i != i_lab);
        let i_desc = a.desc.as_deref().map(|c| nt.col(Some(c), &[]).unwrap_or_else(|| die(&format!("no column {c} in node file {:?}", nt.header))));
        for r in &nt.rows {
            let id = r.get(i_id).map(|s| s.trim().to_string()).unwrap_or_default();
            if id.is_empty() || idx.contains_key(&id) {
                continue;
            }
            idx.insert(id.clone(), ids.len());
            labels.push(r.get(i_lab).map(|l| l.trim().trim_matches('"').to_string()).unwrap_or_else(|| id.clone()));
            types.push(i_typ.and_then(|i| r.get(i)).map(|t| t.trim().trim_matches('"').to_uppercase()).unwrap_or_default());
            if let Some(d) = i_desc.and_then(|i| r.get(i)) {
                descs.insert(ids.len(), d.trim().chars().take(400).collect());
            }
            ids.push(id);
        }
        declared = ids.len();
    }
    let mut dangling: HashSet<String> = HashSet::new();
    let mut node_of = |s: &str, ids: &mut Vec<String>, labels: &mut Vec<String>, types: &mut Vec<String>, idx: &mut HashMap<String, usize>| -> usize {
        let s = s.trim();
        if let Some(&i) = idx.get(s) {
            return i;
        }
        if a.nodes.is_some() {
            dangling.insert(s.to_string());
        }
        idx.insert(s.to_string(), ids.len());
        ids.push(s.to_string());
        labels.push(s.trim_matches('"').to_string());
        types.push(String::new());
        ids.len() - 1
    };

    struct E {
        u: usize,
        v: usize,
        rel: String,
        docs: Vec<String>,
    }
    let mut edges: Vec<E> = Vec::new();
    let mut skipped = 0usize;
    for r in &et.rows {
        let (s, t) = (r.get(ci_src).map(|x| x.trim()).unwrap_or(""), r.get(ci_dst).map(|x| x.trim()).unwrap_or(""));
        if s.is_empty() || t.is_empty() {
            skipped += 1;
            continue;
        }
        let u = node_of(s, &mut ids, &mut labels, &mut types, &mut idx);
        let v = node_of(t, &mut ids, &mut labels, &mut types, &mut idx);
        let rel = ci_rel.and_then(|i| r.get(i)).map(|x| x.trim().to_lowercase()).unwrap_or_default();
        let docs = ci_doc.and_then(|i| r.get(i)).map(|x| csv::split_multi(x)).unwrap_or_default();
        edges.push(E { u, v, rel, docs });
    }
    if edges.is_empty() {
        die("no usable edges");
    }
    let n_nodes = ids.len();

    // ---- 1. duplicate nodes
    let typed = types.iter().any(|t| !t.is_empty());
    let d = dedup::dedup(&labels, &types, a.fuzzy);
    let g = |i: usize| d.group[i];
    let likely_groups = {
        let mut uf = dedup::UnionFind::new(d.n_groups);
        for &(x, y, _, _) in &d.likely {
            uf.union(g(x), g(y));
        }
        (0..d.n_groups).map(|i| uf.find(i)).collect::<HashSet<_>>().len()
    };

    // partial names: "CRUZ" is the tail of exactly one other same-type name, "TAYLOR CRUZ"
    let reps_of: Vec<usize> = {
        let mut seen = HashSet::new();
        (0..n_nodes).filter(|&i| seen.insert(g(i))).collect()
    };
    let partial = dedup::partial_names(&reps_of, &d.norm, &labels, &types, typed);
    let linked: HashSet<(usize, usize)> = edges.iter().map(|e| (g(e.u).min(g(e.v)), g(e.u).max(g(e.v)))).collect();

    // same meaning in different words, through embeddings (optional)
    let text_of = |i: usize| -> String {
        match descs.get(&i) {
            Some(d) if !d.is_empty() => format!("{}: {}", labels[i], d),
            _ => labels[i].clone(),
        }
    };
    let vecs: Option<Vec<Vec<f32>>> = a.embed.as_ref().map(|url| {
        let texts: Vec<String> = reps_of.iter().map(|&i| text_of(i)).collect();
        embed::embed_texts(url, &texts).unwrap_or_else(|e| die(&e))
    });
    let mut known: HashSet<(usize, usize)> = HashSet::new();
    for &(x, y, _, _) in &d.likely {
        known.insert((g(x).min(g(y)), g(x).max(g(y))));
    }
    for &(x, y) in &partial {
        known.insert((g(x).min(g(y)), g(x).max(g(y))));
    }
    let vague = |t: &str| t.is_empty() || matches!(t.to_lowercase().as_str(), "unknown" | "other" | "none" | "null");
    let pair_ok = |x: usize, y: usize| -> bool {
        (!typed || vague(&types[x]) || vague(&types[y]) || types[x] == types[y]) && dedup::names_may_match(&d.norm[x], &d.norm[y])
    };
    let semantic: Vec<(usize, usize, f32)> = match &vecs {
        None => Vec::new(),
        Some(vecs) => embed::similar_pairs(vecs, a.embed_min, |p, q| {
            let (x, y) = (reps_of[p], reps_of[q]);
            pair_ok(x, y) && !known.contains(&(g(x).min(g(y)), g(x).max(g(y))))
        })
        .into_iter()
        .map(|(p, q, c)| (reps_of[p], reps_of[q], c))
        .collect(),
    };

    // an instruction model reads candidate pairs and says whether they are the same entity (optional)
    let mut judged: Vec<(usize, usize, &'static str, judge::Verdict, String)> = Vec::new();
    let mut judge_skipped = 0usize;
    if let Some(url) = &a.judge {
        if descs.is_empty() {
            eprintln!("graphaudit: --judge without --desc: the model sees names and types only");
        }
        let mut cands: Vec<(usize, usize, &'static str)> = Vec::new();
        let mut seen: HashSet<(usize, usize)> = HashSet::new();
        let mut add = |x: usize, y: usize, why: &'static str, cands: &mut Vec<(usize, usize, &'static str)>| {
            let k = (g(x).min(g(y)), g(x).max(g(y)));
            if k.0 != k.1 && seen.insert(k) {
                cands.push((x, y, why));
            }
        };
        for &(x, y, _, why) in &d.likely {
            add(x, y, why, &mut cands);
        }
        for &(x, y) in &partial {
            add(x, y, "partial name", &mut cands);
        }
        for &(x, y, _) in &semantic {
            add(x, y, "embedding", &mut cands);
        }
        for (x, y, _) in dedup::shared_word_pairs(&reps_of, &d.norm, &types, typed, 6) {
            if pair_ok(x, y) {
                add(x, y, "shared word", &mut cands);
            }
        }
        if let Some(vecs) = &vecs {
            // each name's three nearest neighbours
            for p in 0..vecs.len() {
                let mut best: Vec<(f32, usize)> = (0..vecs.len())
                    .filter(|&q| q != p)
                    .map(|q| (vecs[p].iter().zip(&vecs[q]).map(|(u, v)| u * v).sum::<f32>(), q))
                    .collect();
                best.sort_by(|u, v| v.0.partial_cmp(&u.0).unwrap());
                for &(_, q) in best.iter().take(3) {
                    let (x, y) = (reps_of[p], reps_of[q]);
                    if pair_ok(x, y) {
                        add(x, y, "nearest neighbour", &mut cands);
                    }
                }
            }
        }
        // in a typed graph, judge named things only (unless --judge-all): at least one side has a named
        // type and the other a named or vague type
        let mut skipped_abstract = 0usize;
        if typed && !a.judge_all {
            let named = |t: &str| judge::NAMED_TYPES.contains(&t.to_uppercase().as_str());
            let before = cands.len();
            cands.retain(|&(x, y, _)| {
                (named(&types[x]) || named(&types[y])) && (named(&types[x]) || vague(&types[x])) && (named(&types[y]) || vague(&types[y]))
            });
            skipped_abstract = before - cands.len();
        }
        if cands.len() > a.judge_max {
            eprintln!("graphaudit: {} candidate pairs, judging the first {} (raise with --judge-max)", cands.len(), a.judge_max);
            cands.truncate(a.judge_max);
        }
        let ep = judge::endpoint(url).unwrap_or_else(|e| die(&e));
        for (k, &(x, y, why)) in cands.iter().enumerate() {
            if k % 25 == 0 {
                eprintln!("judging {}/{} pairs…", k, cands.len());
            }
            let node = |i: usize| judge::Node { name: &labels[i], kind: &types[i], desc: descs.get(&i).map(|s| s.as_str()).unwrap_or("") };
            let (v, reason) = judge::judge(&ep, &node(x), &node(y)).unwrap_or_else(|e| die(&e));
            judged.push((x, y, why, v, reason));
        }
        judge_skipped = skipped_abstract;
    }

    // ---- 2. edge inflation
    let raw_keys: HashSet<(usize, usize, &str)> =
        edges.iter().map(|e| if a.undirected { (e.u.min(e.v), e.u.max(e.v), e.rel.as_str()) } else { (e.u, e.v, e.rel.as_str()) }).collect();
    // the same pair stored in both directions (regardless of relation label)
    let directed_pairs: HashSet<(usize, usize)> = edges.iter().map(|e| (g(e.u), g(e.v))).filter(|(u, v)| u != v).collect();
    let reciprocal = directed_pairs.iter().filter(|(u, v)| u < v && directed_pairs.contains(&(*v, *u))).count();
    let mut merged: HashMap<(usize, usize, String), HashSet<String>> = HashMap::new();
    let mut merged_rows: HashMap<(usize, usize, String), usize> = HashMap::new();
    let mut self_loops_raw = 0usize;
    let mut self_loops_merge = 0usize;
    for e in &edges {
        if e.u == e.v {
            self_loops_raw += 1;
        } else if g(e.u) == g(e.v) {
            self_loops_merge += 1;
        }
        let (gu, gv) = (g(e.u), g(e.v));
        let k = if a.undirected { (gu.min(gv), gu.max(gv), e.rel.clone()) } else { (gu, gv, e.rel.clone()) };
        merged.entry(k.clone()).or_default().extend(e.docs.iter().cloned());
        *merged_rows.entry(k).or_default() += 1;
    }

    // ---- 3. source support vs chance
    let has_docs = ci_doc.is_some() && edges.iter().any(|e| !e.docs.is_empty());
    let mut all_docs: HashSet<&str> = HashSet::new();
    let mut node_docs: HashMap<usize, HashSet<&str>> = HashMap::new();
    if has_docs {
        for e in &edges {
            for dd in &e.docs {
                all_docs.insert(dd);
                node_docs.entry(g(e.u)).or_default().insert(dd);
                node_docs.entry(g(e.v)).or_default().insert(dd);
            }
        }
    }
    let n_docs = all_docs.len() as u64;
    struct S {
        u: usize,
        v: usize,
        rel: String,
        support: usize,
        co: usize,
        expected: f64,
        p: f64,
        q: f64,
    }
    let mut sup: Vec<S> = Vec::new();
    let mut n_tested = 0usize;
    if has_docs {
        let empty = HashSet::new();
        // Every distinct edge is a test, not only the multi-source ones: picking the
        // edges with ≥ 2 documents first and correcting only among them would select on
        // the outcome. Single-source edges are tested too, then hidden from the table.
        for ((u, v, rel), docs) in &merged {
            if docs.is_empty() || u == v {
                continue;
            }
            let (du, dv) = (node_docs.get(u).unwrap_or(&empty), node_docs.get(v).unwrap_or(&empty));
            let (fu, fv) = (du.len() as u64, dv.len() as u64);
            let co = du.intersection(dv).count();
            // An edge guarantees its own first document: both entities are in it by
            // construction. The evidence is in the documents beyond that one, so the
            // test conditions on it: remove one shared document from both counts.
            let k = docs.len() as u64;
            let expected = 1.0 + (fu.saturating_sub(1)) as f64 * (fv.saturating_sub(1)) as f64 / (n_docs.saturating_sub(1)).max(1) as f64;
            let p = if k <= 1 { 1.0 } else { stats::hypergeom_tail(n_docs - 1, fu - 1, fv - 1, k - 1) };
            sup.push(S { u: *u, v: *v, rel: rel.clone(), support: docs.len(), co, expected, p, q: 1.0 });
        }
        let q = stats::bh(&sup.iter().map(|s| s.p).collect::<Vec<_>>());
        for (s, qv) in sup.iter_mut().zip(q) {
            s.q = qv;
        }
        n_tested = sup.len();
        sup.retain(|s| s.support >= 2);
        sup.sort_by(|x, y| y.support.cmp(&x.support).then(x.p.partial_cmp(&y.p).unwrap()));
    }
    let sig: Vec<&S> = sup.iter().filter(|s| s.q < a.alpha && s.support as f64 > s.expected).collect();
    let artifacts = sup.iter().filter(|s| s.support as f64 <= s.expected * 1.5 + 0.5).count();

    // ---- 4. structure (on the merged graph)
    let mut out_n: HashMap<usize, HashSet<usize>> = HashMap::new();
    let mut indeg: HashMap<usize, usize> = HashMap::new();
    let mut deg: HashMap<usize, usize> = HashMap::new();
    let mut srcs: HashSet<usize> = HashSet::new();
    let mut dsts: HashSet<usize> = HashSet::new();
    for (u, v, _) in merged.keys() {
        *deg.entry(*u).or_default() += 1;
        *deg.entry(*v).or_default() += 1;
        if u == v {
            continue;
        }
        srcs.insert(*u);
        dsts.insert(*v);
        if out_n.entry(*u).or_default().insert(*v) {
            *indeg.entry(*v).or_default() += 1;
        }
        indeg.entry(*u).or_default();
    }
    let in_cycles = if a.undirected { 0 } else { nodes_on_cycles(&out_n) };
    let both = srcs.intersection(&dsts).count();
    let isolated = if a.nodes.is_some() { (0..declared).filter(|&i| !deg.contains_key(&g(i))).count() } else { 0 };
    let mut degs: Vec<usize> = deg.values().cloned().collect();
    degs.sort_unstable_by(|x, y| y.cmp(x));
    let top_k = (degs.len() as f64 * 0.01).ceil().max(1.0) as usize;
    let top_share = degs.iter().take(top_k).sum::<usize>() as f64 / degs.iter().sum::<usize>().max(1) as f64;
    // connected pieces (ignoring direction) among nodes that have at least one edge
    let mut ids: HashMap<usize, usize> = HashMap::new();
    for k in deg.keys() {
        let n = ids.len();
        ids.insert(*k, n);
    }
    let mut cuf = dedup::UnionFind::new(ids.len());
    for (u, v, _) in merged.keys() {
        cuf.union(ids[u], ids[v]);
    }
    let mut comp: HashMap<usize, usize> = HashMap::new();
    for i in 0..ids.len() {
        *comp.entry(cuf.find(i)).or_default() += 1;
    }
    let mut comp_sizes: Vec<usize> = comp.values().cloned().collect();
    comp_sizes.sort_unstable_by(|x, y| y.cmp(x));

    // ---- write CSVs
    fs::create_dir_all(&a.out).unwrap_or_else(|e| die(&e.to_string()));
    let mut s = String::from("group,label,normalized\n");
    for (gi, grp) in d.certain.iter().enumerate() {
        for &i in grp {
            s += &format!("{},{},{}\n", gi + 1, csv_cell(&labels[i]), csv_cell(&d.norm[i]));
        }
    }
    fs::write(a.out.join("duplicates.csv"), s).unwrap();
    let mut s = String::from("label_a,label_b,similarity,reason\n");
    for &(x, y, j, why) in &d.likely {
        s += &format!("{},{},{:.3},{}\n", csv_cell(&labels[x]), csv_cell(&labels[y]), j, why);
    }
    fs::write(a.out.join("likely_duplicates.csv"), s).unwrap();
    let mut s = String::from("short_label,full_label,type,linked_by_an_edge\n");
    for &(x, y) in &partial {
        s += &format!("{},{},{},{}\n", csv_cell(&labels[x]), csv_cell(&labels[y]), csv_cell(&types[x]), linked.contains(&(g(x).min(g(y)), g(x).max(g(y)))));
    }
    fs::write(a.out.join("partial_names.csv"), s).unwrap();
    if a.embed.is_some() {
        let mut s = String::from("label_a,label_b,cosine,type_a,type_b,linked_by_an_edge\n");
        for &(x, y, c) in &semantic {
            s += &format!("{},{},{:.3},{},{},{}\n", csv_cell(&labels[x]), csv_cell(&labels[y]), c, csv_cell(&types[x]), csv_cell(&types[y]), linked.contains(&(g(x).min(g(y)), g(x).max(g(y)))));
        }
        fs::write(a.out.join("semantic_duplicates.csv"), s).unwrap();
    }
    if a.judge.is_some() {
        let mut s = String::from("label_a,label_b,verdict,proposed_by,reason,linked_by_an_edge\n");
        for (x, y, why, v, reason) in &judged {
            s += &format!("{},{},{},{},{},{}\n", csv_cell(&labels[*x]), csv_cell(&labels[*y]), v.as_str(), why, csv_cell(reason), linked.contains(&(g(*x).min(g(*y)), g(*x).max(g(*y)))));
        }
        fs::write(a.out.join("judged_pairs.csv"), s).unwrap();
    }
    let few_docs = has_docs && n_docs < 20;
    let rep = |grp: usize| -> &str {
        let i = (0..n_nodes).find(|&i| g(i) == grp).unwrap();
        &labels[i]
    };
    let mut rep_cache: HashMap<usize, String> = HashMap::new();
    let mut name = |grp: usize| -> String { rep_cache.entry(grp).or_insert_with(|| rep(grp).to_string()).clone() };
    let mut s = String::from("source,relation,target,documents,co_mention_documents,expected_by_chance,p,q\n");
    for x in &sup {
        s += &format!("{},{},{},{},{},{:.2},{:.3e},{:.3e}\n", csv_cell(&name(x.u)), csv_cell(&x.rel), csv_cell(&name(x.v)), x.support, x.co, x.expected, x.p, x.q);
    }
    fs::write(a.out.join("edge_support.csv"), s).unwrap();

    // ---- report
    let arrow = if a.undirected { "—" } else { "→" };
    let mut r = String::new();
    let p = |r: &mut String, t: String| {
        r.push_str(&t);
        r.push('\n');
    };
    p(&mut r, "# graphaudit report\n".into());
    p(&mut r, format!("Input: `{}`{} · {} edge rows{} · columns: source=`{}`, target=`{}`, relation={}, provenance={}\n",
        a.edges.display(),
        a.nodes.as_ref().map(|n| format!(" + `{}`", n.display())).unwrap_or_default(),
        edges.len(),
        if skipped > 0 { format!(" ({skipped} rows without both endpoints skipped)") } else { String::new() },
        et.header[ci_src], et.header[ci_dst],
        ci_rel.map(|i| format!("`{}`", et.header[i])).unwrap_or("none".into()),
        ci_doc.map(|i| format!("`{}`", et.header[i])).unwrap_or("none".into())));

    p(&mut r, "## Summary\n".into());
    p(&mut r, "| Measure | As stored | After audit |\n|---|---|---|".into());
    p(&mut r, format!("| Nodes | {} | {} certain unique ({} duplicates, {}) · {} if likely duplicates also merge |", n_nodes, d.n_groups, n_nodes - d.n_groups, pct(n_nodes - d.n_groups, n_nodes), likely_groups));
    p(&mut r, format!("| Edges | {} rows · {} distinct | {} after merging duplicate nodes ({} fewer, {}) |", edges.len(), raw_keys.len(), merged.len(), raw_keys.len() - merged.len(), pct(raw_keys.len() - merged.len(), raw_keys.len())));
    if !partial.is_empty() {
        let ex = partial.iter().take(2).map(|&(x, y)| format!("{} ↔ {}", md(&labels[x]), md(&labels[y]))).collect::<Vec<_>>().join("; ");
        p(&mut r, format!("| Possible partial-name duplicates | – | {} to review ({}) |", partial.len(), ex));
    }
    if !a.undirected {
        p(&mut r, format!("| Same pair stored in both directions | {} pairs | {} |", reciprocal,
            if reciprocal > 0 { "if the graph is undirected (GraphRAG's is), these are duplicate edges: rerun with `--undirected`" } else { "none" }));
    }
    if has_docs {
        if few_docs {
            p(&mut r, format!("| Edges claimed by ≥ 2 documents | {} | not testable: only {} documents |", sup.len(), n_docs));
        } else {
            p(&mut r, format!("| Edges claimed by ≥ 2 documents | {} | {} beat chance (FDR q < {}) · {} within what chance predicts |", sup.len(), sig.len(), a.alpha, artifacts));
        }
    }
    if a.nodes.is_some() && !dangling.is_empty() {
        p(&mut r, format!("| Edge endpoints missing from the node table | {} | edges pointing to nodes that do not exist |", dangling.len()));
    }
    if a.undirected {
        p(&mut r, "| Cycles | – | not assessed (graph treated as undirected) |\n".into());
    } else {
        p(&mut r, format!("| Directed cycles | – | {} nodes in cycles{} |\n", in_cycles,
            if in_cycles == 0 && both == 0 { " · **acyclic by construction**" } else if reciprocal > 0 && in_cycles > 0 { " (two-way duplicate pairs count as cycles)" } else { "" }));
    }

    p(&mut r, "## 1. Duplicate nodes\n".into());
    p(&mut r, format!("**{} of {} nodes ({}) are certain duplicates**: identical after lower-casing, removing accents, punctuation and leading articles{}. {} further {} for review: singular/plural forms of the same name, a leading title (King, Saint), the same name with an UNKNOWN type, or one typo apart (1 character edit up to 15 characters, 2 up to 30). Reordered words, different numbers, different honorifics (Mr / Mrs), opposites (movable / immovable){} are never paired.\n",
        n_nodes - d.n_groups, n_nodes, pct(n_nodes - d.n_groups, n_nodes), if typed { ", and of the same type" } else { "" }, d.likely.len(), if d.likely.len() == 1 { "pair is" } else { "pairs are" }, if typed { " and different types" } else { "" }));
    if !d.certain.is_empty() {
        let mut groups = d.certain.clone();
        groups.sort_by_key(|grp| std::cmp::Reverse(grp.len()));
        p(&mut r, "| Copies | Written as |\n|---|---|".into());
        for grp in groups.iter().take(a.top) {
            let names: Vec<String> = grp.iter().take(6).map(|&i| format!("`{}`", md(&labels[i]))).collect();
            p(&mut r, format!("| {} | {}{} |", grp.len(), names.join(", "), if grp.len() > 6 { ", …" } else { "" }));
        }
        p(&mut r, String::new());
    }
    if !d.likely.is_empty() {
        p(&mut r, "Likely duplicates (review by hand):\n".into());
        p(&mut r, "| Label A | Label B | Why |\n|---|---|---|".into());
        for &(x, y, _, why) in d.likely.iter().take(a.top) {
            p(&mut r, format!("| {} | {} | {} |", md(&labels[x]), md(&labels[y]), why));
        }
        p(&mut r, String::new());
    }
    if !partial.is_empty() {
        p(&mut r, format!("**{} possible partial-name duplicates**: a one-word name that is the first or last word of exactly one other {}name, such as a surname used alone. Some will be genuinely different things (a town and the base named after it); review them by hand.\n", partial.len(), if typed { "same-type " } else { "" }));
        p(&mut r, "| Short name | Full name | Type | Linked to each other by an edge |\n|---|---|---|---|".into());
        for &(x, y) in partial.iter().take(a.top) {
            let l = linked.contains(&(g(x).min(g(y)), g(x).max(g(y))));
            p(&mut r, format!("| {} | {} | {} | {} |", md(&labels[x]), md(&labels[y]), if types[x].is_empty() { "–".into() } else { md(&types[x]) }, if l { "yes" } else { "no" }));
        }
        p(&mut r, String::new());
    }
    if a.judge.is_some() {
        let same: Vec<_> = judged.iter().filter(|j| j.3 == judge::Verdict::Same).collect();
        let unsure = judged.iter().filter(|j| j.3 == judge::Verdict::Disagree).count();
        p(&mut r, format!("**Judged by a language model: {} of {} candidate pairs are the same entity** (both orders answered SAME{}); {} got opposite answers in the two orders. Candidates come from the lists above, names sharing a rare word{}. {}A SAME whose reason describes a hierarchy (\"a type of\", \"a subset of\") is counted as uncertain. A model can be wrong: check before merging.\n",
            same.len(), judged.len(), if descs.is_empty() { ", names and types only" } else { ", reading descriptions" }, unsure,
            if vecs.is_some() { " and each name's nearest embedding neighbours" } else { "" },
            if judge_skipped > 0 { format!("{} pairs of abstract types (concepts, methods) were not judged; add `--judge-all` to include them. ", judge_skipped) } else { String::new() }));
        if !same.is_empty() {
            p(&mut r, "| Label A | Label B | Proposed by | Model's reason |\n|---|---|---|---|".into());
            for j in same.iter().take(a.top.max(30)) {
                p(&mut r, format!("| {} | {} | {} | {} |", md(&labels[j.0]), md(&labels[j.1]), j.2, md(&j.4)));
            }
            p(&mut r, String::new());
        }
    }
    if a.embed.is_some() {
        p(&mut r, format!("**{} possible same-meaning duplicates** (embedding cosine ≥ {:.2}{}, not already listed above; numbers, honorifics, opposites and types are guarded as above). Experimental: on a real graph with bge-m3, related but different things (gold / silver, two royal houses) scored above true duplicates. Skim this list; never merge from it.\n",
            semantic.len(), a.embed_min, if descs.is_empty() { ", names only" } else { ", names with descriptions" }));
        if !semantic.is_empty() {
            p(&mut r, "| Label A | Label B | Cosine |\n|---|---|---|".into());
            for &(x, y, c) in semantic.iter().take(a.top) {
                p(&mut r, format!("| {} | {} | {:.3} |", md(&labels[x]), md(&labels[y]), c));
            }
            p(&mut r, String::new());
        }
    } else {
        p(&mut r, "Same meaning in different words (\"car\" / \"automobile\") is not detected here: rerun with `--embed URL` and an embedding server.\n".into());
    }

    p(&mut r, "## 2. Edge inflation\n".into());
    p(&mut r, format!("{} edge rows hold {} distinct edges; merging duplicate nodes leaves **{}**. {} edges became self-loops (a node linked to a copy of itself){}.\n",
        edges.len(), raw_keys.len(), merged.len(), self_loops_merge,
        if self_loops_raw > 0 { format!("; {self_loops_raw} were self-loops already") } else { String::new() }));
    if !a.undirected && reciprocal > 0 {
        p(&mut r, format!("{} node pairs are linked in both directions (A → B and B → A). In a directed graph that can be meaningful; in an undirected one (GraphRAG builds undirected graphs) each pair is one relationship stored twice. Rerun with `--undirected` to count them once.\n", reciprocal));
    }
    let mut multi: Vec<(&(usize, usize, String), &usize)> = merged_rows.iter().filter(|(_, &c)| c > 1).collect();
    multi.sort_by(|x, y| y.1.cmp(x.1));
    if !multi.is_empty() {
        p(&mut r, "Edges stored most often:\n\n| Edge | Rows |\n|---|---|".into());
        for ((u, v, rel), c) in multi.iter().take(a.top.min(10)) {
            p(&mut r, format!("| {} {} {}{} | {} |", md(&name(*u)), arrow, md(&name(*v)), if rel.is_empty() { String::new() } else { format!(" ({})", md(rel)) }, c));
        }
        p(&mut r, String::new());
    }

    p(&mut r, "## 3. Source support vs. chance\n".into());
    if !has_docs {
        p(&mut r, "Skipped: no provenance column (which document each edge came from). Add one with `--doc` to test \"N sources confirm X → Y\" claims.\n".into());
    } else {
        p(&mut r, format!("{} documents. For each edge stored from ≥ 2 documents: given that it is found once (which puts both entities in that document by construction), and that its entities appear in *a* and *b* of the *D* documents, independent placement predicts *1 + (a−1)(b−1)/(D−1)* documents (exact hypergeometric test; Benjamini–Hochberg across all {} edges with provenance, so picking the multi-source ones does not inflate significance). An edge whose support is close to that number is what two frequent entities produce anyway.\n", n_docs, n_tested));
        if few_docs {
            p(&mut r, format!("**Not testable here: with only {} documents, even an edge found in every document could not be told apart from chance after correcting for the number of edges tested.** The table is shown for transparency only. Source counts in graphs built from so few documents carry no evidence either way.\n", n_docs));
        } else {
            p(&mut r, format!("**{} of {} multi-source edges beat chance; {} are within what chance predicts.**\n", sig.len(), sup.len(), artifacts));
        }
        if !sup.is_empty() {
            p(&mut r, "| Edge | Documents | Expected by chance | q | Verdict |\n|---|---|---|---|---|".into());
            for x in sup.iter().take(a.top) {
                let verdict = if few_docs { "not testable" } else if x.q < a.alpha && x.support as f64 > x.expected { "beyond chance" } else if x.support as f64 <= x.expected * 1.5 + 0.5 { "frequency artifact" } else { "not significant" };
                p(&mut r, format!("| {} {} {}{} | {} | {:.1} | {} | {} |", md(&name(x.u)), arrow, md(&name(x.v)), if x.rel.is_empty() { String::new() } else { format!(" ({})", md(&x.rel)) }, x.support, x.expected, fmt_p(x.q), verdict));
            }
            p(&mut r, String::new());
        }
    }

    p(&mut r, "## 4. Structure\n".into());
    if a.undirected {
        p(&mut r, "- Cycles: not assessed, because the graph is treated as undirected.".into());
    } else {
        p(&mut r, format!("- Directed cycles: {} nodes sit on a cycle{}.", in_cycles,
            if reciprocal > 0 && in_cycles > 0 { format!("; {} two-way pairs (A → B and B → A) are cycles of length 2", reciprocal) } else { String::new() }));
    }
    if !a.undirected && in_cycles == 0 {
        if both == 0 {
            p(&mut r, "- **No node is ever both a source and a target, so this graph cannot contain a cycle whatever the data say.** \"No feedback loops found\" is a property of how edges were built, not a finding.".into());
        } else {
            p(&mut r, format!("- The graph is acyclic although {} nodes are both sources and targets: that absence of loops is a real property of the data.", both));
        }
    }
    if let Some(&big) = comp_sizes.first() {
        let linked: usize = comp_sizes.iter().sum();
        if comp_sizes.len() == 1 {
            p(&mut r, "- Connected pieces: 1. Every linked node can be reached from every other.".into());
        } else {
            p(&mut r, format!("- Connected pieces: **{}**. The largest holds {} of {} linked nodes ({}); a traversal from it never reaches the other {} (piece sizes: {}).",
                comp_sizes.len(), big, linked, pct(big, linked), linked - big,
                comp_sizes.iter().skip(1).take(12).map(|x| x.to_string()).collect::<Vec<_>>().join(", ")
                    + if comp_sizes.len() > 13 { ", …" } else { "" }));
        }
    }
    p(&mut r, format!("- Degree concentration: the top 1% of nodes ({}) hold {:.0}% of all edge endpoints.", top_k, 100.0 * top_share));
    if a.nodes.is_some() {
        let mut dl: Vec<&String> = dangling.iter().collect();
        dl.sort();
        p(&mut r, format!("- Nodes declared but never used in an edge: {} · edge endpoints missing from the node table: {}{}.", isolated, dangling.len(),
            if dl.is_empty() { String::new() } else { format!(" ({})", dl.iter().take(10).map(|x| format!("`{}`", md(x))).collect::<Vec<_>>().join(", ")) }));
    }
    p(&mut r, format!("\nFiles: `duplicates.csv`, `likely_duplicates.csv`, `partial_names.csv`, `edge_support.csv`{}{}.",
        if a.embed.is_some() { ", `semantic_duplicates.csv`" } else { "" }, if a.judge.is_some() { ", `judged_pairs.csv`" } else { "" }));

    fs::write(a.out.join("report.md"), &r).unwrap();
    let mut by_kind: BTreeMap<&str, String> = BTreeMap::new();
    by_kind.insert("nodes", format!("{} → {} unique", n_nodes, d.n_groups));
    by_kind.insert("edges", format!("{} → {}", raw_keys.len(), merged.len()));
    eprintln!(
        "nodes {} · edges {}{} · report: {}",
        by_kind["nodes"],
        by_kind["edges"],
        if has_docs { format!(" · multi-source edges beating chance: {}/{}", sig.len(), sup.len()) } else { String::new() },
        a.out.join("report.md").display()
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cycle_count_excludes_descendants() {
        // 0 <-> 1 is a cycle; 1 -> 2 -> 3 hangs off it; 4 -> 5 -> 6 -> 4 is another cycle
        let mut g: HashMap<usize, HashSet<usize>> = HashMap::new();
        for (u, v) in [(0, 1), (1, 0), (1, 2), (2, 3), (4, 5), (5, 6), (6, 4)] {
            g.entry(u).or_default().insert(v);
        }
        assert_eq!(nodes_on_cycles(&g), 5);
        let mut dag: HashMap<usize, HashSet<usize>> = HashMap::new();
        dag.entry(0).or_default().insert(1);
        assert_eq!(nodes_on_cycles(&dag), 0);
    }
}
