//! Step 2: find ideas from different books that say the same thing — and measure how
//! many such "links" the same procedure finds when the ideas are replaced by decoys.
//!
//! * Each idea is embedded with a multilingual sentence-embedding model (bge-m3 via
//!   llama-server --embeddings).
//! * A link is a pair of ideas from two different books that are each other's nearest
//!   neighbour across those two books, with similarity above a threshold τ.
//! * Decoys: the same idea with its words shuffled. Same words, same topic, no sentence
//!   meaning. The full procedure is run again on decoys: the number of decoy links is
//!   what word/topic overlap alone produces. Every real link also gets a "meaning lift":
//!   its similarity minus the similarity of its decoy versions.
//! * Positive control: can the embedder pick an idea's own source quote (Spanish for
//!   Marcus Aurelius) out of 20 quotes? If it can't, no link it finds means anything.

use crate::http::Endpoint;
use crate::json::{self, n, obj, s, Json};
use std::collections::{BTreeMap, HashMap};
use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::time::Duration;

pub struct Config {
    pub out: PathBuf,
    pub server: String,
    pub percentile: f64,
    pub decoys: usize,
    /// "words": shuffle every word. "content": keep function words (not, no, never,
    /// there, is, can...) in place and shuffle only content words, so a decoy keeps the
    /// grammatical form of the idea ("There is no X") but loses what it is about.
    pub decoy_mode: String,
    pub seed: u64,
    pub top: usize,
}

#[derive(Clone)]
struct Idea {
    id: String,
    book: String,
    claim: String,
    quote: String,
}

// ------------------------------------------------------------------ helpers

struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        self.0.wrapping_mul(0x2545F4914F6CDD1D)
    }
    fn below(&mut self, n: usize) -> usize {
        (self.next() % n.max(1) as u64) as usize
    }
}

fn fnv(t: &str) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    for b in t.as_bytes() {
        h ^= *b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}

const FUNCTION_WORDS: &[&str] = &[
    "a", "an", "the", "of", "to", "in", "on", "at", "by", "for", "with", "from", "as", "into", "onto", "over", "under",
    "about", "against", "between", "through", "without", "within", "upon", "is", "are", "was", "were", "be", "been",
    "being", "am", "can", "cannot", "could", "may", "might", "must", "should", "would", "will", "shall", "do", "does",
    "did", "has", "have", "had", "not", "no", "nor", "never", "none", "nobody", "nothing", "neither", "nowhere",
    "and", "or", "but", "if", "then", "than", "that", "this", "these", "those", "it", "its", "itself", "they",
    "their", "them", "themselves", "there", "here", "which", "who", "whom", "whose", "what", "when", "where", "why",
    "how", "all", "any", "each", "every", "some", "only", "also", "more", "most", "less", "least", "such", "so",
    "one", "one's", "oneself", "because", "while", "even", "just", "very", "too", "own", "same", "other", "another",
];

pub fn is_function_word(w: &str) -> bool {
    let l: String = w.trim_matches(|c: char| !c.is_alphanumeric() && c != '\'').to_lowercase();
    l.is_empty() || l.contains("n't") || FUNCTION_WORDS.contains(&l.as_str())
}

pub fn is_negated(t: &str) -> bool {
    t.split_whitespace().any(|w| {
        let l: String = w.trim_matches(|c: char| !c.is_alphanumeric() && c != '\'').to_lowercase();
        l.contains("n't") || ["not", "no", "never", "none", "nobody", "nothing", "neither", "nor", "cannot", "without", "nowhere"].contains(&l.as_str())
    })
}

/// Keep function words where they are; shuffle the content words among content slots.
pub fn shuffle_content(t: &str, seed: u64) -> String {
    let w: Vec<&str> = t.split_whitespace().collect();
    let slots: Vec<usize> = (0..w.len()).filter(|&i| !is_function_word(w[i])).collect();
    if slots.len() < 2 {
        return shuffle_words(t, seed);
    }
    let mut content: Vec<&str> = slots.iter().map(|&i| w[i]).collect();
    let mut r = Rng(seed | 1);
    for attempt in 0..6 {
        for i in (1..content.len()).rev() {
            let j = r.below(i + 1);
            content.swap(i, j);
        }
        let mut out = w.clone();
        for (k, &i) in slots.iter().enumerate() {
            out[i] = content[k];
        }
        let o = out.join(" ");
        if o != t || attempt == 5 {
            return o;
        }
    }
    unreachable!()
}

/// Same words, random order. Guaranteed different from the original when possible.
pub fn shuffle_words(t: &str, seed: u64) -> String {
    let mut w: Vec<&str> = t.split_whitespace().collect();
    if w.len() < 2 {
        return t.to_string();
    }
    let mut r = Rng(seed | 1);
    for _ in 0..4 {
        for i in (1..w.len()).rev() {
            let j = r.below(i + 1);
            w.swap(i, j);
        }
        let out = w.join(" ");
        if out != t {
            return out;
        }
    }
    w.reverse();
    w.join(" ")
}

fn dot(a: &[f32], b: &[f32]) -> f32 {
    a.iter().zip(b).map(|(x, y)| x * y).sum()
}

fn normalize(mut v: Vec<f32>) -> Vec<f32> {
    let n = v.iter().map(|x| x * x).sum::<f32>().sqrt();
    if n > 0.0 {
        for x in &mut v {
            *x /= n;
        }
    }
    v
}

fn percentile(v: &mut [f32], p: f64) -> f32 {
    if v.is_empty() {
        return 1.0;
    }
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let i = ((p / 100.0) * (v.len() - 1) as f64).round() as usize;
    v[i.min(v.len() - 1)]
}

// ------------------------------------------------------------- embeddings

struct Embedder {
    ep: Endpoint,
    cache: HashMap<u64, Vec<f32>>,
    bin_path: PathBuf,
}

/// Binary cache record: u64 hash, u32 dim, dim × f32 (little endian).
fn write_record(f: &mut impl Write, h: u64, v: &[f32]) -> std::io::Result<()> {
    let mut buf = Vec::with_capacity(12 + v.len() * 4);
    buf.extend_from_slice(&h.to_le_bytes());
    buf.extend_from_slice(&(v.len() as u32).to_le_bytes());
    for x in v {
        buf.extend_from_slice(&x.to_le_bytes());
    }
    f.write_all(&buf)
}

fn read_bin(path: &std::path::Path, cache: &mut HashMap<u64, Vec<f32>>) -> Result<(), String> {
    let data = fs::read(path).map_err(|e| e.to_string())?;
    let mut i = 0usize;
    while i + 12 <= data.len() {
        let h = u64::from_le_bytes(data[i..i + 8].try_into().unwrap());
        let d = u32::from_le_bytes(data[i + 8..i + 12].try_into().unwrap()) as usize;
        let end = i + 12 + d * 4;
        if end > data.len() {
            break; // truncated last record (interrupted run): ignore it
        }
        let v: Vec<f32> = data[i + 12..end].chunks_exact(4).map(|c| f32::from_le_bytes(c.try_into().unwrap())).collect();
        cache.insert(h, v);
        i = end;
    }
    Ok(())
}

/// Fast reader for the old JSON-lines cache: {"h":"123","v":[0.1,-0.2,...]}
pub fn parse_jsonl_line(line: &str) -> Option<(u64, Vec<f32>)> {
    let hs = line.find("\"h\":\"")? + 5;
    let he = hs + line[hs..].find('"')?;
    let h: u64 = line[hs..he].parse().ok()?;
    let vs = line.find('[')? + 1;
    let ve = line.rfind(']')?;
    let v: Option<Vec<f32>> = line[vs..ve].split(',').map(|x| x.trim().parse::<f32>().ok()).collect();
    Some((h, v?))
}

impl Embedder {
    fn new(server: &str, out: &std::path::Path) -> Result<Self, String> {
        let bin_path = out.join("embeddings.bin");
        let jsonl = out.join("embeddings.jsonl");
        let mut cache = HashMap::new();
        if bin_path.exists() {
            read_bin(&bin_path, &mut cache)?;
            eprintln!("loaded {} cached embeddings", cache.len());
        }
        if jsonl.exists() {
            // one-time migration from the old, slow JSON cache
            eprintln!("converting {} to binary (one time only)...", jsonl.display());
            let f = fs::File::open(&jsonl).map_err(|e| e.to_string())?;
            let mut out_f = std::io::BufWriter::new(OpenOptions::new().create(true).append(true).open(&bin_path).map_err(|e| e.to_string())?);
            let mut added = 0usize;
            for line in BufReader::with_capacity(1 << 20, f).lines().map_while(Result::ok) {
                if let Some((h, v)) = parse_jsonl_line(&line) {
                    if !cache.contains_key(&h) {
                        write_record(&mut out_f, h, &v).map_err(|e| e.to_string())?;
                        cache.insert(h, v);
                        added += 1;
                    }
                }
            }
            out_f.flush().map_err(|e| e.to_string())?;
            drop(out_f);
            let done = out.join("embeddings.jsonl.converted");
            fs::rename(&jsonl, &done).map_err(|e| e.to_string())?;
            eprintln!("  {} embeddings converted; old file kept as {} (you can delete it)", added, done.display());
        }
        Ok(Embedder { ep: Endpoint::parse(server)?, cache, bin_path })
    }

    fn embed(&mut self, texts: &[String]) -> Result<Vec<Vec<f32>>, String> {
        let missing: Vec<&String> = {
            let mut seen = std::collections::HashSet::new();
            texts.iter().filter(|t| !self.cache.contains_key(&fnv(t)) && seen.insert(fnv(t))).collect()
        };
        if !missing.is_empty() {
            eprintln!("embedding {} texts ({} cached)...", missing.len(), texts.len() - missing.len());
        }
        let mut f = OpenOptions::new().create(true).append(true).open(&self.bin_path).map_err(|e| e.to_string())?;
        for (bi, batch) in missing.chunks(32).enumerate() {
            let body = obj(vec![("input", Json::Arr(batch.iter().map(|t| s(t)).collect()))]);
            let resp = self.ep.post_json("/v1/embeddings", &body.to_string(), Duration::from_secs(300)).map_err(|e| {
                format!("{e}\n(is the embedding server running with --embeddings on {}?)", "the --server port")
            })?;
            let v = json::parse(&resp).map_err(|e| format!("bad JSON from embedding server: {e}"))?;
            let data = match v.get("data") {
                Some(Json::Arr(d)) => d.clone(),
                _ => return Err(format!("unexpected embedding reply: {}", resp.chars().take(200).collect::<String>())),
            };
            if data.len() != batch.len() {
                return Err(format!("asked for {} embeddings, got {}", batch.len(), data.len()));
            }
            for (k, item) in data.iter().enumerate() {
                let idx = item.get("index").and_then(|x| x.as_f64()).map(|x| x as usize).unwrap_or(k);
                let emb = item.get("embedding").ok_or("embedding missing in reply")?;
                // pooling 'none' returns one vector per token: refuse instead of guessing
                if let Json::Arr(a) = emb {
                    if matches!(a.first(), Some(Json::Arr(_))) {
                        return Err("the server returned one vector per token; restart it with --pooling cls".into());
                    }
                }
                let e = normalize(to_vec(emb).ok_or("bad embedding vector")?);
                let t = batch.get(idx).ok_or("bad index in embedding reply")?;
                let h = fnv(t);
                write_record(&mut f, h, &e).map_err(|e| e.to_string())?;
                self.cache.insert(h, e);
            }
            if (bi + 1) % 25 == 0 {
                eprintln!("  {} / {}", ((bi + 1) * 32).min(missing.len()), missing.len());
            }
        }
        Ok(texts.iter().map(|t| self.cache[&fnv(t)].clone()).collect())
    }
}

fn to_vec(j: &Json) -> Option<Vec<f32>> {
    match j {
        Json::Arr(a) => a.iter().map(|x| x.as_f64().map(|f| f as f32)).collect(),
        _ => None,
    }
}

// ------------------------------------------------------------------ linking

#[derive(Clone)]
struct Link {
    i: usize,
    j: usize,
    sim: f32,
    lift: f32,
}

/// Mutual nearest neighbours across every pair of books, above `tau`.
fn mutual_links(vecs: &[Vec<f32>], books: &[usize], nbooks: usize, tau: f32) -> Vec<(usize, usize, f32)> {
    let mut by: Vec<Vec<usize>> = vec![Vec::new(); nbooks];
    for (i, b) in books.iter().enumerate() {
        by[*b].push(i);
    }
    let mut out = Vec::new();
    for a in 0..nbooks {
        for b in (a + 1)..nbooks {
            let (xa, xb) = (&by[a], &by[b]);
            if xa.is_empty() || xb.is_empty() {
                continue;
            }
            let best = |i: usize, pool: &[usize]| -> (usize, f32) {
                pool.iter().map(|&j| (j, dot(&vecs[i], &vecs[j]))).fold((usize::MAX, f32::MIN), |m, x| if x.1 > m.1 { x } else { m })
            };
            for &i in xa {
                let (j, sij) = best(i, xb);
                if sij < tau {
                    continue;
                }
                let (back, _) = best(j, xa);
                if back == i {
                    out.push((i, j, sij));
                }
            }
        }
    }
    out
}

pub fn run(cfg: &Config) -> Result<(), String> {
    // ---- load verified ideas
    let path = cfg.out.join("claims.jsonl");
    let f = fs::File::open(&path).map_err(|e| format!("cannot open {} (run extract first): {e}", path.display()))?;
    let mut ideas = Vec::new();
    for line in BufReader::new(f).lines().map_while(Result::ok) {
        let Ok(v) = json::parse(&line) else { continue };
        if v.get("verified").and_then(|x| x.as_bool()) != Some(true) {
            continue;
        }
        let g = |k: &str| v.get(k).and_then(|x| x.as_str()).unwrap_or("").to_string();
        ideas.push(Idea { id: g("id"), book: g("book"), claim: g("claim"), quote: g("quote") });
    }
    if ideas.len() < 10 {
        return Err(format!("only {} verified ideas in {}; need more to link", ideas.len(), path.display()));
    }
    let mut book_ids: BTreeMap<String, usize> = BTreeMap::new();
    for i in &ideas {
        let k = book_ids.len();
        book_ids.entry(i.book.clone()).or_insert(k);
    }
    let nbooks = book_ids.len();
    if nbooks < 2 {
        return Err("ideas come from a single book; linking needs at least two".into());
    }
    let books: Vec<usize> = ideas.iter().map(|i| book_ids[&i.book]).collect();
    let nb = ideas.len();

    // ---- texts: claims, quotes, decoys
    let claims: Vec<String> = ideas.iter().map(|i| i.claim.clone()).collect();
    let quotes: Vec<String> = ideas.iter().map(|i| i.quote.clone()).collect();
    let decoy_txt: Vec<Vec<String>> = (0..cfg.decoys)
        .map(|k| {
            ideas
                .iter()
                .map(|i| {
                    let seed = cfg.seed ^ fnv(&i.id) ^ (k as u64 + 1) * 0x9E37;
                    if cfg.decoy_mode == "content" { shuffle_content(&i.claim, seed) } else { shuffle_words(&i.claim, seed) }
                })
                .collect()
        })
        .collect();

    let mut emb = Embedder::new(&cfg.server, &cfg.out)?;
    let vc = emb.embed(&claims)?;
    let vq = emb.embed(&quotes)?;
    let vd: Vec<Vec<Vec<f32>>> = decoy_txt.iter().map(|t| emb.embed(t)).collect::<Result<_, _>>()?;

    // ---- positive control: own quote among 20
    let mut rng = Rng(cfg.seed | 1);
    let mut hits = vec![(0usize, 0usize); nbooks];
    for i in 0..nb {
        let own = dot(&vc[i], &vq[i]);
        let mut beaten = false;
        let mut tried = 0;
        while tried < 19 {
            let j = rng.below(nb);
            let same_passage = ideas[j].id.rsplitn(2, '#').nth(1) == ideas[i].id.rsplitn(2, '#').nth(1);
            if j == i || same_passage {
                continue;
            }
            tried += 1;
            if dot(&vc[i], &vq[j]) >= own {
                beaten = true;
            }
        }
        hits[books[i]].1 += 1;
        if !beaten {
            hits[books[i]].0 += 1;
        }
    }
    let total_hit: usize = hits.iter().map(|h| h.0).sum();
    let pos_acc = total_hit as f64 / nb as f64;
    let decoy_closeness: f64 = (0..nb).map(|i| dot(&vc[i], &vd[0][i]) as f64).sum::<f64>() / nb as f64;

    // ---- threshold from the real cross-book similarity distribution
    let mut cross: Vec<f32> = Vec::new();
    for i in 0..nb {
        for j in (i + 1)..nb {
            if books[i] != books[j] {
                cross.push(dot(&vc[i], &vc[j]));
            }
        }
    }
    let n_cross = cross.len();
    let tau = percentile(&mut cross, cfg.percentile);
    let median = cross[cross.len() / 2];

    // ---- real links and decoy links, same procedure, same τ
    let real = mutual_links(&vc, &books, nbooks, tau);
    let decoy_counts: Vec<usize> = vd.iter().map(|d| mutual_links(d, &books, nbooks, tau).len()).collect();
    let decoy_mean = decoy_counts.iter().sum::<usize>() as f64 / decoy_counts.len().max(1) as f64;

    let mut links: Vec<Link> = real
        .iter()
        .map(|&(i, j, sim)| {
            let mut ds = 0.0f32;
            for d in &vd {
                ds += (dot(&d[i], &vc[j]) + dot(&vc[i], &d[j])) / 2.0;
            }
            let lift = sim - ds / vd.len().max(1) as f32;
            Link { i, j, sim, lift }
        })
        .collect();
    links.sort_by(|a, b| b.lift.partial_cmp(&a.lift).unwrap());

    // ---- write links.jsonl
    let suffix = if cfg.decoy_mode == "content" { "_content" } else { "" };
    let lp = cfg.out.join(format!("links{suffix}.jsonl"));
    let mut lf = fs::File::create(&lp).map_err(|e| e.to_string())?;
    for l in &links {
        let (a, b) = (&ideas[l.i], &ideas[l.j]);
        writeln!(
            lf,
            "{}",
            obj(vec![
                ("a", s(&a.id)),
                ("b", s(&b.id)),
                ("sim", n((l.sim as f64 * 1000.0).round() / 1000.0)),
                ("lift", n((l.lift as f64 * 1000.0).round() / 1000.0)),
                ("claim_a", s(&a.claim)),
                ("claim_b", s(&b.claim)),
                ("quote_a", s(&a.quote)),
                ("quote_b", s(&b.quote)),
            ])
            .to_string()
        )
        .map_err(|e| e.to_string())?;
    }

    // ---- report
    let book_names: Vec<&String> = {
        let mut v: Vec<(&String, &usize)> = book_ids.iter().collect();
        v.sort_by_key(|x| *x.1);
        v.into_iter().map(|x| x.0).collect()
    };
    let short = |b: &str| -> String { b.chars().take(28).collect() };
    let mut r = String::new();
    r.push_str("# ANIMUS v5 — cross-book links\n\n");
    r.push_str(&format!("- {} verified ideas from {} books · {} cross-book pairs compared\n\n", nb, nbooks, n_cross));

    r.push_str("## 1. Can the embedder recognise meaning at all? (positive control)\n\n");
    r.push_str("Each idea must pick its own source quote out of 20 quotes (its own + 19 random). Chance = 5%.\n\n");
    r.push_str("| Book | Correct | Ideas |\n|---|---|---|\n");
    for (b, (h, t)) in hits.iter().enumerate() {
        r.push_str(&format!("| {} | {:.0}% | {} |\n", short(book_names[b]), 100.0 * *h as f64 / (*t).max(1) as f64, t));
    }
    r.push_str(&format!("| **All** | **{:.0}%** | {} |\n\n", 100.0 * pos_acc, nb));
    if pos_acc < 0.5 {
        r.push_str("**Warning:** the embedder does not reliably recognise meaning; treat every link below as unproven.\n\n");
    }
    r.push_str(&format!(
        "Decoy closeness: an idea and its decoy have mean similarity **{:.2}**{}\n\n",
        decoy_closeness,
        if decoy_closeness > 0.9 { " — very high: this embedder barely notices word order, so the decoy test is weak." } else { "." }
    ));

    r.push_str(&format!(
        "Decoy mode: **{}**{}\n\n",
        cfg.decoy_mode,
        if cfg.decoy_mode == "content" {
            " — function words (not, no, never, there, is, can…) stay in place; only content words are shuffled. A decoy keeps the sentence's grammatical form but not its subject."
        } else {
            " — every word shuffled."
        }
    ));
    r.push_str("## 2. Links found vs. links that word overlap alone produces\n\n");
    r.push_str(&format!(
        "τ = {:.3} (the {}th percentile of all cross-book similarities; median is {:.3}). A link = mutual nearest neighbours across two books with similarity ≥ τ.\n\n",
        tau, cfg.percentile, median
    ));
    r.push_str("| | Links |\n|---|---|\n");
    r.push_str(&format!("| Real ideas | **{}** |\n", links.len()));
    let dlabel = if cfg.decoy_mode == "content" { "content words shuffled, form kept" } else { "all words shuffled" };
    r.push_str(&format!("| Decoys ({dlabel}), mean of {} runs | {:.1} (runs: {:?}) |\n\n", decoy_counts.len(), decoy_mean, decoy_counts));
    let verdict = if links.is_empty() {
        "No pair of ideas from different books is close enough to count as the same idea."
    } else if (links.len() as f64) <= decoy_mean * 1.5 + 1.0 {
        "Decoys produce about as many links as real ideas: these links are explained by shared words and topic, not by shared meaning."
    } else {
        "Real ideas produce clearly more links than decoys. Read the table: links with high lift are the ones whose meaning, not just vocabulary, matches."
    };
    r.push_str(&format!("**{verdict}**\n\n"));

    // negation check: are the strongest links just "there is no X" matching "no one can Y"?
    let base_neg = ideas.iter().filter(|i| is_negated(&i.claim)).count() as f64 / nb as f64;
    let both_neg = |ls: &[Link]| -> (usize, usize) {
        (ls.iter().filter(|l| is_negated(&ideas[l.i].claim) && is_negated(&ideas[l.j].claim)).count(), ls.len())
    };
    let top100: Vec<Link> = links.iter().take(100).cloned().collect();
    let (tn, tt) = both_neg(&top100);
    let (an, at) = both_neg(&links);
    r.push_str("## Negation check\n\n");
    r.push_str(&format!("- Ideas containing a negation (not, no, never, nobody…): **{:.0}%**\n", 100.0 * base_neg));
    r.push_str(&format!("- If negation played no role, a link would have both sides negated about **{:.0}%** of the time\n", 100.0 * base_neg * base_neg));
    r.push_str(&format!("- All links with both sides negated: **{:.0}%** ({} of {})\n", 100.0 * an as f64 / at.max(1) as f64, an, at));
    r.push_str(&format!("- Top 100 links by lift with both sides negated: **{:.0}%** ({} of {})\n\n", 100.0 * tn as f64 / tt.max(1) as f64, tn, tt));

    r.push_str("## 3. Links, best meaning lift first\n\n");
    r.push_str(if cfg.decoy_mode == "content" {
        "`lift` = similarity minus the similarity of the same ideas with their content words shuffled (grammar kept). Near 0 → the match is sentence form (\"there is no X\"), not subject.\n\n"
    } else {
        "`lift` = similarity minus the similarity of the same ideas with their words shuffled. Near 0 → the match is word overlap.\n\n"
    });
    for (k, l) in links.iter().take(cfg.top).enumerate() {
        let (a, b) = (&ideas[l.i], &ideas[l.j]);
        r.push_str(&format!("### {}. sim {:.3} · lift {:+.3}\n\n", k + 1, l.sim, l.lift));
        r.push_str(&format!("- **{}** — {}\n  > {}\n", short(&a.book), a.claim, a.quote));
        r.push_str(&format!("- **{}** — {}\n  > {}\n\n", short(&b.book), b.claim, b.quote));
    }
    let rp = cfg.out.join(format!("links_report{suffix}.md"));
    fs::write(&rp, &r).map_err(|e| e.to_string())?;
    eprintln!(
        "{} links (decoys [{}]: {:.1}); positive control {:.0}%; top-100 both negated {}/{}; report: {}",
        links.len(),
        cfg.decoy_mode,
        decoy_mean,
        100.0 * pos_acc,
        tn,
        tt,
        rp.display()
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn shuffle_keeps_words() {
        let t = "cooperation needs a real chance of meeting again";
        let d = shuffle_words(t, 7);
        assert_ne!(d, t);
        let mut a: Vec<&str> = t.split(' ').collect();
        let mut b: Vec<&str> = d.split(' ').collect();
        a.sort();
        b.sort();
        assert_eq!(a, b);
    }
    #[test]
    fn content_shuffle_keeps_form() {
        let t = "There is no theory-independent way to reconstruct reality.";
        let d = shuffle_content(t, 3);
        let w: Vec<&str> = d.split(' ').collect();
        assert_eq!(&w[..3], &["There", "is", "no"]);
        assert_ne!(d, t);
        assert!(is_negated(t) && is_negated(&d));
        assert!(is_negated("A program doesn't care.") && !is_negated("Reciprocity sustains cooperation."));
    }

    #[test]
    fn old_cache_line_parses_and_bin_roundtrips() {
        let (h, v) = parse_jsonl_line(r#"{"h":"1234567890123","v":[0.5,-0.25,1e-3,0]}"#).unwrap();
        assert_eq!(h, 1234567890123);
        assert_eq!(v, vec![0.5, -0.25, 0.001, 0.0]);
        let dir = std::env::temp_dir().join(format!("animus5_bin_{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let p = dir.join("e.bin");
        let mut f = fs::File::create(&p).unwrap();
        write_record(&mut f, h, &v).unwrap();
        write_record(&mut f, 7, &[1.0, 2.0]).unwrap();
        f.write_all(&[1, 2, 3]).unwrap(); // truncated tail must be ignored
        drop(f);
        let mut c = HashMap::new();
        read_bin(&p, &mut c).unwrap();
        assert_eq!(c.len(), 2);
        assert_eq!(c[&7], vec![1.0, 2.0]);
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn mutual_nn() {
        let v = vec![normalize(vec![1.0, 0.0]), normalize(vec![0.0, 1.0]), normalize(vec![0.9, 0.1]), normalize(vec![0.1, 0.9])];
        let books = vec![0, 0, 1, 1];
        let l = mutual_links(&v, &books, 2, 0.5);
        assert_eq!(l.len(), 2);
        assert!(l.iter().any(|x| x.0 == 0 && x.1 == 2));
    }
}
