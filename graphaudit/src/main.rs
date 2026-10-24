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

struct Args {
    edges: PathBuf,
    nodes: Option<PathBuf>,
    src: Option<String>,
    dst: Option<String>,
    rel: Option<String>,
    doc: Option<String>,
    nid: Option<String>,
    nlabel: Option<String>,
    fuzzy: f64,
    alpha: f64,
    top: usize,
    out: PathBuf,
}

const HELP: &str = "graphaudit — audit a knowledge graph's own numbers

USAGE: graphaudit <edges.csv> [options]

  --nodes FILE       node table (id + label); otherwise node names come from the edges
  --src/--dst COL    edge endpoint columns   (auto: source/subject/head/from, target/object/tail/to)
  --rel COL          relation column          (auto: relation/predicate/type)
  --doc COL          provenance column        (auto: doc/document/source_doc/text_unit_ids/chunk_id…)
                     multi-valued cells like 'a;b' or \"['a','b']\" are split
  --id/--label COL   node-table columns      (auto: id/node_id, label/name/title)
  --typos S          edit budget for 'likely duplicates' (default 1 = 1 edit up to 15 chars,
                     2 up to 30, 3 beyond; 0 = off; 2 = twice as loose)
  --alpha A          FDR level for source support (default 0.05)
  --top N            rows to show per table (default 15)
  --out DIR          output folder (default graphaudit_out)

Writes DIR/report.md, duplicates.csv, likely_duplicates.csv, edge_support.csv.";

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
        fuzzy: 1.0,
        alpha: 0.05,
        top: 15,
        out: PathBuf::from("graphaudit_out"),
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
            "--typos" => a.fuzzy = v("--typos").parse().unwrap_or_else(|_| die("bad --typos")),
            "--alpha" => a.alpha = v("--alpha").parse().unwrap_or_else(|_| die("bad --alpha")),
            "--top" => a.top = v("--top").parse().unwrap_or_else(|_| die("bad --top")),
            "--out" => a.out = v("--out").into(),
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
    let mut declared = 0usize;
    if let Some(np) = &a.nodes {
        let nt = read_table(np);
        let i_id = nt.col(a.nid.as_deref(), NID).unwrap_or_else(|| die(&format!("no id column in node file {:?}; use --id", nt.header)));
        let i_lab = nt.col(a.nlabel.as_deref(), NLABEL).filter(|&i| i != i_id).unwrap_or(i_id);
        for r in &nt.rows {
            let id = r.get(i_id).map(|s| s.trim().to_string()).unwrap_or_default();
            if id.is_empty() || idx.contains_key(&id) {
                continue;
            }
            idx.insert(id.clone(), ids.len());
            labels.push(r.get(i_lab).cloned().unwrap_or_else(|| id.clone()));
            ids.push(id);
        }
        declared = ids.len();
    }
    let mut dangling: HashSet<String> = HashSet::new();
    let mut node_of = |s: &str, ids: &mut Vec<String>, labels: &mut Vec<String>, idx: &mut HashMap<String, usize>| -> usize {
        let s = s.trim();
        if let Some(&i) = idx.get(s) {
            return i;
        }
        if a.nodes.is_some() {
            dangling.insert(s.to_string());
        }
        idx.insert(s.to_string(), ids.len());
        ids.push(s.to_string());
        labels.push(s.to_string());
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
        let u = node_of(s, &mut ids, &mut labels, &mut idx);
        let v = node_of(t, &mut ids, &mut labels, &mut idx);
        let rel = ci_rel.and_then(|i| r.get(i)).map(|x| x.trim().to_lowercase()).unwrap_or_default();
        let docs = ci_doc.and_then(|i| r.get(i)).map(|x| csv::split_multi(x)).unwrap_or_default();
        edges.push(E { u, v, rel, docs });
    }
    if edges.is_empty() {
        die("no usable edges");
    }
    let n_nodes = ids.len();

    // ---- 1. duplicate nodes
    let d = dedup::dedup(&labels, a.fuzzy);
    let g = |i: usize| d.group[i];
    let likely_groups = {
        let mut uf = dedup::UnionFind::new(d.n_groups);
        for &(x, y, _) in &d.likely {
            uf.union(g(x), g(y));
        }
        (0..d.n_groups).map(|i| uf.find(i)).collect::<HashSet<_>>().len()
    };

    // ---- 2. edge inflation
    let raw_keys: HashSet<(usize, usize, &str)> = edges.iter().map(|e| (e.u, e.v, e.rel.as_str())).collect();
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
        let k = (g(e.u), g(e.v), e.rel.clone());
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
    let in_cycles = nodes_on_cycles(&out_n);
    let both = srcs.intersection(&dsts).count();
    let isolated = if a.nodes.is_some() { (0..declared).filter(|&i| !deg.contains_key(&g(i))).count() } else { 0 };
    let mut degs: Vec<usize> = deg.values().cloned().collect();
    degs.sort_unstable_by(|x, y| y.cmp(x));
    let top_k = (degs.len() as f64 * 0.01).ceil().max(1.0) as usize;
    let top_share = degs.iter().take(top_k).sum::<usize>() as f64 / degs.iter().sum::<usize>().max(1) as f64;

    // ---- write CSVs
    fs::create_dir_all(&a.out).unwrap_or_else(|e| die(&e.to_string()));
    let mut s = String::from("group,label,normalized\n");
    for (gi, grp) in d.certain.iter().enumerate() {
        for &i in grp {
            s += &format!("{},{},{}\n", gi + 1, csv_cell(&labels[i]), csv_cell(&d.norm[i]));
        }
    }
    fs::write(a.out.join("duplicates.csv"), s).unwrap();
    let mut s = String::from("label_a,label_b,similarity\n");
    for &(x, y, j) in &d.likely {
        s += &format!("{},{},{:.3}\n", csv_cell(&labels[x]), csv_cell(&labels[y]), j);
    }
    fs::write(a.out.join("likely_duplicates.csv"), s).unwrap();
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
    if has_docs {
        p(&mut r, format!("| Edges claimed by ≥ 2 documents | {} | {} beat chance (FDR q < {}) · {} within what chance predicts |", sup.len(), sig.len(), a.alpha, artifacts));
    }
    p(&mut r, format!("| Directed cycles | – | {} nodes in cycles{} |\n", in_cycles,
        if in_cycles == 0 && both == 0 { " · **acyclic by construction**" } else { "" }));

    p(&mut r, "## 1. Duplicate nodes\n".into());
    p(&mut r, format!("**{} of {} nodes ({}) are certain duplicates**: identical after lower-casing, removing accents, punctuation, leading articles and simple plurals. {} further {} the same name up to a typo (1 character edit for names up to 15 characters, 2 up to 30); the same words in another order, or different numbers, are never paired.\n",
        n_nodes - d.n_groups, n_nodes, pct(n_nodes - d.n_groups, n_nodes), d.likely.len(), if d.likely.len() == 1 { "pair is" } else { "pairs are" }));
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
        p(&mut r, "| Label A | Label B | Similarity |\n|---|---|---|".into());
        for &(x, y, j) in d.likely.iter().take(a.top) {
            p(&mut r, format!("| {} | {} | {:.2} |", md(&labels[x]), md(&labels[y]), j));
        }
        p(&mut r, String::new());
    }
    p(&mut r, "Same meaning in different words (\"car\" / \"automobile\") is not detected here: that needs an embedding model.\n".into());

    p(&mut r, "## 2. Edge inflation\n".into());
    p(&mut r, format!("{} edge rows hold {} distinct edges; merging duplicate nodes leaves **{}**. {} edges became self-loops (a node linked to a copy of itself){}.\n",
        edges.len(), raw_keys.len(), merged.len(), self_loops_merge,
        if self_loops_raw > 0 { format!("; {self_loops_raw} were self-loops already") } else { String::new() }));
    let mut multi: Vec<(&(usize, usize, String), &usize)> = merged_rows.iter().filter(|(_, &c)| c > 1).collect();
    multi.sort_by(|x, y| y.1.cmp(x.1));
    if !multi.is_empty() {
        p(&mut r, "Edges stored most often:\n\n| Edge | Rows |\n|---|---|".into());
        for ((u, v, rel), c) in multi.iter().take(a.top.min(10)) {
            p(&mut r, format!("| {} → {}{} | {} |", md(&name(*u)), md(&name(*v)), if rel.is_empty() { String::new() } else { format!(" ({})", md(rel)) }, c));
        }
        p(&mut r, String::new());
    }

    p(&mut r, "## 3. Source support vs. chance\n".into());
    if !has_docs {
        p(&mut r, "Skipped: no provenance column (which document each edge came from). Add one with `--doc` to test \"N sources confirm X → Y\" claims.\n".into());
    } else {
        p(&mut r, format!("{} documents. For each edge stored from ≥ 2 documents: given that it is found once (which puts both entities in that document by construction), and that its entities appear in *a* and *b* of the *D* documents, independent placement predicts *1 + (a−1)(b−1)/(D−1)* documents (exact hypergeometric test; Benjamini–Hochberg across all {} edges with provenance, so picking the multi-source ones does not inflate significance). An edge whose support is close to that number is what two frequent entities produce anyway.\n", n_docs, n_tested));
        p(&mut r, format!("**{} of {} multi-source edges beat chance; {} are within what chance predicts.**\n", sig.len(), sup.len(), artifacts));
        if !sup.is_empty() {
            p(&mut r, "| Edge | Documents | Expected by chance | q | Verdict |\n|---|---|---|---|---|".into());
            for x in sup.iter().take(a.top) {
                let verdict = if x.q < a.alpha && x.support as f64 > x.expected { "beyond chance" } else if x.support as f64 <= x.expected * 1.5 + 0.5 { "frequency artifact" } else { "not significant" };
                p(&mut r, format!("| {} → {}{} | {} | {:.1} | {} | {} |", md(&name(x.u)), md(&name(x.v)), if x.rel.is_empty() { String::new() } else { format!(" ({})", md(&x.rel)) }, x.support, x.expected, fmt_p(x.q), verdict));
            }
            p(&mut r, String::new());
        }
    }

    p(&mut r, "## 4. Structure\n".into());
    p(&mut r, format!("- Directed cycles: {} nodes sit on a cycle.", in_cycles));
    if in_cycles == 0 {
        if both == 0 {
            p(&mut r, "- **No node is ever both a source and a target, so this graph cannot contain a cycle whatever the data say.** \"No feedback loops found\" is a property of how edges were built, not a finding.".into());
        } else {
            p(&mut r, format!("- The graph is acyclic although {} nodes are both sources and targets: that absence of loops is a real property of the data.", both));
        }
    }
    p(&mut r, format!("- Degree concentration: the top 1% of nodes ({}) hold {:.0}% of all edge endpoints.", top_k, 100.0 * top_share));
    if a.nodes.is_some() {
        p(&mut r, format!("- Nodes declared but never used in an edge: {} · edge endpoints missing from the node table: {}.", isolated, dangling.len()));
    }
    p(&mut r, "\nFiles: `duplicates.csv`, `likely_duplicates.csv`, `edge_support.csv`.".into());

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
