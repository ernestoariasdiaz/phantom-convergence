//! convaudit — audits "convergence" claims in co-occurrence knowledge graphs.
//!
//! The question it answers: when a system reports that "N independent sources
//! confirm failure -> algorithm", how many sources would confirm that pair if
//! the two words were placed at random within each source?  If the answer is
//! "about N", the convergence is a frequency artifact, not a finding.
//!
//! Std-only Rust. Input: a directory of .txt files (one file = one source).

use std::collections::{BTreeMap, HashMap, HashSet};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

// ---------------------------------------------------------------- vocabulary

/// Reconstruction of the February 2026 ANIMUS Business vocabulary
/// (tension words -> resolution words), English + Spanish.
const DEFAULT_VOCAB: &str = "\
tension: failure, failures, crisis, gap, limitation, limitations, collapse, risk, loss, problem, problems, waste, fraud, inflation, poverty, shortage, corruption, burnout, vulnerability, bottleneck, barrier, monopoly, inequality, debt, decline, breach, error, errors, conflict
tension: fracaso, crisis, brecha, limitación, colapso, riesgo, pérdida, pérdidas, problema, desperdicio, fraude, inflación, pobreza, escasez, corrupción, agotamiento, vulnerabilidad, monopolio, desigualdad, deuda, declive, error, conflicto
resolution: algorithm, algorithms, regulation, developed, discovered, breakthrough, framework, solution, solutions, innovation, policy, reform, education, cooperation, investment, initiative, automation, reduced, prevention, architecture, fixed, agreement
resolution: algoritmo, regulación, desarrollo, descubrimiento, avance, arquitectura, solución, innovación, política, reforma, educación, cooperación, inversión, iniciativa, automatización, prevención, acuerdo
";

#[derive(Clone)]
struct Vocab {
    tension: Vec<String>,
    resolution: Vec<String>,
}

fn parse_vocab(text: &str) -> Vocab {
    let mut tension = Vec::new();
    let mut resolution = Vec::new();
    let text = text.trim_start_matches('\u{feff}'); // UTF-8 BOM from Windows editors
    for line in text.lines() {
        let line = line.trim().trim_start_matches('\u{feff}');
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((kind, words)) = line.split_once(':') else { continue };
        let target = match kind.trim() {
            "tension" => &mut tension,
            "resolution" => &mut resolution,
            _ => continue,
        };
        for w in words.split(',') {
            let w = w.trim().to_lowercase();
            if !w.is_empty() && !target.contains(&w) {
                target.push(w);
            }
        }
    }
    Vocab { tension, resolution }
}

// ------------------------------------------------------------------- corpus

struct Source {
    name: String,
    /// window index -> set of word ids present in that window
    windows: Vec<HashSet<u32>>,
}

struct Corpus {
    sources: Vec<Source>,
    word_id: HashMap<String, u32>,
    words: Vec<String>,
}

fn tokenize(text: &str) -> Vec<String> {
    text.split(|c: char| !c.is_alphanumeric())
        .filter(|t| !t.is_empty())
        .map(|t| t.to_lowercase())
        .collect()
}

fn load_corpus(dir: &Path, window: usize) -> Corpus {
    let mut paths: Vec<PathBuf> = fs::read_dir(dir)
        .unwrap_or_else(|e| die(&format!("cannot read {}: {e}", dir.display())))
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().map(|x| x == "txt").unwrap_or(false))
        .collect();
    paths.sort();
    if paths.is_empty() {
        die(&format!("no .txt files in {}", dir.display()));
    }
    let mut word_id: HashMap<String, u32> = HashMap::new();
    let mut words: Vec<String> = Vec::new();
    let mut sources = Vec::new();
    for p in paths {
        let bytes = fs::read(&p).unwrap_or_default();
        let text = String::from_utf8_lossy(&bytes);
        let toks = tokenize(&text);
        let mut wins = Vec::new();
        for chunk in toks.chunks(window.max(1)) {
            let mut set = HashSet::new();
            for t in chunk {
                let id = *word_id.entry(t.clone()).or_insert_with(|| {
                    words.push(t.clone());
                    (words.len() - 1) as u32
                });
                set.insert(id);
            }
            wins.push(set);
        }
        if !wins.is_empty() {
            let name = p.file_stem().unwrap().to_string_lossy().to_string();
            sources.push(Source { name, windows: wins });
        }
    }
    Corpus { sources, word_id, words }
}

// -------------------------------------------------------------- statistics

/// ln C(n, k)
fn ln_choose(n: u64, k: u64) -> f64 {
    if k > n {
        return f64::NEG_INFINITY;
    }
    ln_fact(n) - ln_fact(k) - ln_fact(n - k)
}

fn ln_fact(n: u64) -> f64 {
    // exact sum for small n, Stirling for large n
    if n < 256 {
        (2..=n).map(|i| (i as f64).ln()).sum()
    } else {
        let x = n as f64;
        x * x.ln() - x + 0.5 * (2.0 * std::f64::consts::PI * x).ln() + 1.0 / (12.0 * x)
    }
}

/// Probability that a source "confirms" a pair (shares at least one window)
/// if the b windows holding word r were chosen uniformly at random among the
/// W windows (word t occupies a windows). Hypergeometric: 1 - C(W-a,b)/C(W,b).
fn p_confirm(w: u64, a: u64, b: u64) -> f64 {
    if a == 0 || b == 0 {
        return 0.0;
    }
    if a + b > w {
        return 1.0;
    }
    1.0 - (ln_choose(w - a, b) - ln_choose(w, b)).exp()
}

/// Exact upper tail P(X >= k) for a Poisson-binomial X with success probs `ps`.
fn poisson_binomial_tail(ps: &[f64], k: usize) -> f64 {
    let mut dist = vec![0.0f64; ps.len() + 1];
    dist[0] = 1.0;
    for (i, &p) in ps.iter().enumerate() {
        for j in (0..=i + 1).rev() {
            let stay = dist[j] * (1.0 - p);
            let moved = if j > 0 { dist[j - 1] * p } else { 0.0 };
            dist[j] = stay + moved;
        }
    }
    dist[k.min(ps.len())..].iter().sum::<f64>().min(1.0)
}

#[derive(Clone)]
struct PairStat {
    t: String,
    r: String,
    sources_confirming: usize,
    sources_expected: f64,
    p_value: f64,
    q_value: f64,
    windows_observed: u64,
    windows_expected: f64,
    confirming_names: Vec<String>,
}

fn word_windows(src: &Source, id: u32) -> Vec<usize> {
    src.windows
        .iter()
        .enumerate()
        .filter(|(_, s)| s.contains(&id))
        .map(|(i, _)| i)
        .collect()
}

fn analyze_pairs(c: &Corpus, tension: &[String], resolution: &[String]) -> Vec<PairStat> {
    // precompute window lists per source per vocab word
    let all: Vec<&String> = tension.iter().chain(resolution.iter()).collect();
    let mut occ: HashMap<&String, Vec<Vec<usize>>> = HashMap::new();
    for w in &all {
        let per = match c.word_id.get(*w) {
            Some(&id) => c.sources.iter().map(|s| word_windows(s, id)).collect(),
            None => vec![Vec::new(); c.sources.len()],
        };
        occ.insert(*w, per);
    }
    let mut out = Vec::new();
    for t in tension {
        for r in resolution {
            if t == r {
                continue;
            }
            let (ot, or) = (&occ[t], &occ[r]);
            let mut confirming = 0usize;
            let mut names = Vec::new();
            let mut ps = Vec::with_capacity(c.sources.len());
            let mut wobs = 0u64;
            let mut wexp = 0.0f64;
            for (si, s) in c.sources.iter().enumerate() {
                let w = s.windows.len() as u64;
                let (a, b) = (ot[si].len() as u64, or[si].len() as u64);
                ps.push(p_confirm(w, a, b));
                wexp += (a * b) as f64 / w as f64;
                if a > 0 && b > 0 {
                    let set: HashSet<usize> = ot[si].iter().copied().collect();
                    let shared = or[si].iter().filter(|x| set.contains(x)).count() as u64;
                    wobs += shared;
                    if shared > 0 {
                        confirming += 1;
                        names.push(s.name.clone());
                    }
                }
            }
            if confirming == 0 {
                continue;
            }
            let expected: f64 = ps.iter().sum();
            out.push(PairStat {
                t: t.clone(),
                r: r.clone(),
                sources_confirming: confirming,
                sources_expected: expected,
                p_value: poisson_binomial_tail(&ps, confirming),
                q_value: 1.0,
                windows_observed: wobs,
                windows_expected: wexp,
                confirming_names: names,
            });
        }
    }
    benjamini_hochberg(&mut out);
    out.sort_by(|a, b| {
        b.sources_confirming
            .cmp(&a.sources_confirming)
            .then(a.p_value.partial_cmp(&b.p_value).unwrap())
    });
    out
}

fn benjamini_hochberg(v: &mut [PairStat]) {
    let m = v.len() as f64;
    let mut idx: Vec<usize> = (0..v.len()).collect();
    idx.sort_by(|&a, &b| v[a].p_value.partial_cmp(&v[b].p_value).unwrap());
    let mut running = 1.0f64;
    for (rank, &i) in idx.iter().enumerate().rev() {
        let q = (v[i].p_value * m / (rank as f64 + 1.0)).min(1.0);
        running = running.min(q);
        v[i].q_value = running;
    }
}

// ------------------------------------------------- null-vocabulary baseline

struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        // xorshift64*
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        self.0.wrapping_mul(0x2545F4914F6CDD1D)
    }
    fn below(&mut self, n: usize) -> usize {
        (self.next() % n as u64) as usize
    }
}

/// Number of sources in which each word appears at least once.
fn source_freq(c: &Corpus) -> Vec<u32> {
    let mut f = vec![0u32; c.words.len()];
    for s in &c.sources {
        let mut seen: HashSet<u32> = HashSet::new();
        for w in &s.windows {
            seen.extend(w.iter().copied());
        }
        for id in seen {
            f[id as usize] += 1;
        }
    }
    f
}

/// Replace each vocabulary word by a random corpus word with similar
/// source frequency (excluding the vocabulary and very short tokens).
fn matched_random_vocab(
    c: &Corpus,
    sf: &[u32],
    v: &Vocab,
    rng: &mut Rng,
) -> Vocab {
    let vocab_set: HashSet<&String> = v.tension.iter().chain(v.resolution.iter()).collect();
    let pick = |w: &String, rng: &mut Rng, used: &mut HashSet<u32>| -> Option<String> {
        let id = *c.word_id.get(w)?;
        let target = sf[id as usize] as f64;
        let lo = (target * 0.8).floor() as u32;
        let hi = (target * 1.25).ceil() as u32;
        let cands: Vec<u32> = (0..c.words.len() as u32)
            .filter(|&i| {
                let f = sf[i as usize];
                f >= lo.max(1)
                    && f <= hi
                    && !used.contains(&i)
                    && c.words[i as usize].chars().count() >= 5
                    && c.words[i as usize].chars().all(|ch| ch.is_alphabetic())
                    && !vocab_set.contains(&c.words[i as usize])
            })
            .collect();
        if cands.is_empty() {
            return None;
        }
        let chosen = cands[rng.below(cands.len())];
        used.insert(chosen);
        Some(c.words[chosen as usize].clone())
    };
    let mut used = HashSet::new();
    let tension = v.tension.iter().filter_map(|w| pick(w, rng, &mut used)).collect();
    let resolution = v.resolution.iter().filter_map(|w| pick(w, rng, &mut used)).collect();
    Vocab { tension, resolution }
}

// ------------------------------------------------------------------- report

fn fmt_p(p: f64) -> String {
    if p < 1e-4 {
        format!("{:.1e}", p)
    } else {
        format!("{:.4}", p)
    }
}

fn die(msg: &str) -> ! {
    eprintln!("convaudit: {msg}");
    std::process::exit(2);
}

struct Args {
    corpus: PathBuf,
    vocab: Option<PathBuf>,
    window: usize,
    nulls: usize,
    top: usize,
    seed: u64,
    out: Option<PathBuf>,
    alpha: f64,
}

fn parse_args() -> Args {
    let mut a = Args {
        corpus: PathBuf::new(),
        vocab: None,
        window: 300,
        nulls: 50,
        top: 15,
        seed: 20260221,
        out: None,
        alpha: 0.05,
    };
    let mut it = env::args().skip(1);
    let mut positional = None;
    while let Some(arg) = it.next() {
        let mut val = |name: &str| it.next().unwrap_or_else(|| die(&format!("{name} needs a value")));
        match arg.as_str() {
            "--vocab" => a.vocab = Some(val("--vocab").into()),
            "--window" => a.window = val("--window").parse().unwrap_or_else(|_| die("bad --window")),
            "--nulls" => a.nulls = val("--nulls").parse().unwrap_or_else(|_| die("bad --nulls")),
            "--top" => a.top = val("--top").parse().unwrap_or_else(|_| die("bad --top")),
            "--seed" => a.seed = val("--seed").parse().unwrap_or_else(|_| die("bad --seed")),
            "--alpha" => a.alpha = val("--alpha").parse().unwrap_or_else(|_| die("bad --alpha")),
            "--out" => a.out = Some(val("--out").into()),
            "-h" | "--help" => {
                println!("{}", HELP);
                std::process::exit(0);
            }
            s if s.starts_with("--") => die(&format!("unknown option {s}")),
            s => positional = Some(PathBuf::from(s)),
        }
    }
    a.corpus = positional.unwrap_or_else(|| die("missing corpus directory (see --help)"));
    a
}

const HELP: &str = "convaudit — audit 'N sources confirm X -> Y' claims against chance

USAGE: convaudit <corpus_dir> [options]
  corpus_dir      directory of .txt files; each file is one source
  --vocab FILE    lines 'tension: w1, w2' / 'resolution: w1, w2'
                  (default: reconstructed ANIMUS Feb-2026 vocabulary)
  --window N      tokens per window/page (default 300)
  --nulls K       random frequency-matched vocabularies (default 50)
  --top N         pairs to list (default 15)
  --alpha A       FDR level for 'real association' (default 0.05)
  --seed S        RNG seed (default 20260221)
  --out FILE      write the Markdown report here (default: stdout)";

fn main() {
    let args = parse_args();
    let vocab = match &args.vocab {
        Some(p) => parse_vocab(&fs::read_to_string(p).unwrap_or_else(|e| die(&format!("{e}")))),
        None => parse_vocab(DEFAULT_VOCAB),
    };
    if vocab.tension.is_empty() || vocab.resolution.is_empty() {
        die(&format!(
            "vocabulary has {} tension and {} resolution words; both lists need at least one \
             (lines must start with 'tension:' and 'resolution:')",
            vocab.tension.len(),
            vocab.resolution.len()
        ));
    }
    let corpus = load_corpus(&args.corpus, args.window);
    let n_src = corpus.sources.len();
    let n_win: usize = corpus.sources.iter().map(|s| s.windows.len()).sum();

    let pairs = analyze_pairs(&corpus, &vocab.tension, &vocab.resolution);
    let significant = pairs.iter().filter(|p| p.q_value < args.alpha).count();

    // null vocabularies
    let sf = source_freq(&corpus);
    let mut rng = Rng(args.seed | 1);
    let mut null_max = Vec::new();
    let mut null_ge10 = Vec::new();
    let mut null_sig = Vec::new();
    let mut null_example: Option<PairStat> = None;
    for _ in 0..args.nulls {
        let rv = matched_random_vocab(&corpus, &sf, &vocab, &mut rng);
        let ps = analyze_pairs(&corpus, &rv.tension, &rv.resolution);
        null_max.push(ps.first().map(|p| p.sources_confirming).unwrap_or(0));
        null_ge10.push(ps.iter().filter(|p| p.sources_confirming >= 10).count());
        null_sig.push(ps.iter().filter(|p| p.q_value < args.alpha).count());
        if null_example.is_none() {
            null_example = ps.first().cloned();
        }
    }
    let mean = |v: &[usize]| v.iter().sum::<usize>() as f64 / v.len().max(1) as f64;

    let real_max = pairs.first().map(|p| p.sources_confirming).unwrap_or(0);
    let real_ge10 = pairs.iter().filter(|p| p.sources_confirming >= 10).count();

    let overlap: Vec<&String> = vocab.tension.iter().filter(|w| vocab.resolution.contains(w)).collect();

    // --------------------------------------------------------------- render
    let mut r = String::new();
    let push = |r: &mut String, s: String| {
        r.push_str(&s);
        r.push('\n');
    };
    push(&mut r, "# convaudit report\n".into());
    push(&mut r, format!("- Corpus: `{}` — {} sources, {} windows of {} tokens", args.corpus.display(), n_src, n_win, args.window));
    push(&mut r, format!("- Vocabulary: {} tension × {} resolution words → {} pairs observed at least once", vocab.tension.len(), vocab.resolution.len(), pairs.len()));
    push(&mut r, format!("- Null baseline: {} random vocabularies, each word matched to a corpus word of similar source frequency\n", args.nulls));

    push(&mut r, "## 1. Does the vocabulary beat random words?\n".into());
    push(&mut r, "| Metric | Real vocabulary | Random vocabularies (mean) |".into());
    push(&mut r, "|---|---|---|".into());
    push(&mut r, format!("| Strongest pair: # sources 'confirming' | {} | {:.1} (range {}–{}) |", real_max, mean(&null_max), null_max.iter().min().unwrap_or(&0), null_max.iter().max().unwrap_or(&0)));
    push(&mut r, format!("| Pairs 'confirmed' by ≥10 sources | {} | {:.1} |", real_ge10, mean(&null_ge10)));
    push(&mut r, format!("| Pairs significant after FDR (q<{}) | {} | {:.1} |", args.alpha, significant, mean(&null_sig)));
    if let Some(ex) = &null_example {
        push(&mut r, format!("\nExample from a random vocabulary: **{} → {}** is 'confirmed' by {} sources.\n", ex.t, ex.r, ex.sources_confirming));
    }

    push(&mut r, "## 2. Top pairs: observed vs. expected by chance\n".into());
    push(&mut r, "`expected` = sources that would 'confirm' the pair if both words were scattered at random inside each source (exact hypergeometric). `p` = exact Poisson-binomial tail; `q` = Benjamini–Hochberg.\n".into());
    push(&mut r, "| Pair | Sources confirming | Expected by chance | Window lift | p | q | Verdict |".into());
    push(&mut r, "|---|---|---|---|---|---|---|".into());
    for p in pairs.iter().take(args.top) {
        let lift = if p.windows_expected > 0.0 { p.windows_observed as f64 / p.windows_expected } else { 0.0 };
        let verdict = if p.q_value < args.alpha && lift > 1.0 {
            "association beyond chance"
        } else if p.sources_confirming as f64 <= p.sources_expected * 1.15 {
            "frequency artifact"
        } else {
            "not significant"
        };
        push(&mut r, format!("| {} → {} | {} | {:.1} | {:.2} | {} | {} | {} |", p.t, p.r, p.sources_confirming, p.sources_expected, lift, fmt_p(p.p_value), fmt_p(p.q_value), verdict));
    }

    let mut sig: Vec<&PairStat> = pairs.iter().filter(|p| p.q_value < args.alpha && p.windows_observed as f64 > p.windows_expected).collect();
    sig.sort_by(|a, b| a.p_value.partial_cmp(&b.p_value).unwrap());
    push(&mut r, "\n## 3. Pairs that survive the test\n".into());
    if sig.is_empty() {
        push(&mut r, "None. No pair co-occurs more than chance predicts once multiple testing is accounted for.".into());
    } else {
        push(&mut r, "| Pair | Sources | Expected | Window lift | q |".into());
        push(&mut r, "|---|---|---|---|---|".into());
        for p in sig.iter().take(args.top) {
            push(&mut r, format!("| {} → {} | {} | {:.1} | {:.2} | {} |", p.t, p.r, p.sources_confirming, p.sources_expected, p.windows_observed as f64 / p.windows_expected.max(1e-9), fmt_p(p.q_value)));
        }
    }

    push(&mut r, "\n## 4. Can this graph ever contain a feedback loop?\n".into());
    if overlap.is_empty() {
        push(&mut r, "No. Every edge goes from a tension word to a resolution word, and the two lists share no word. A graph whose edges only run from set T to a disjoint set R is bipartite and directed one way, so it is acyclic **by construction** — whatever the sources say. Observing 'zero loops' is therefore not evidence about the sources.".into());
    } else {
        push(&mut r, format!("Possibly: these words are in both lists: {}.", overlap.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(", ")));
    }

    // per-source contribution of the top pair, for transparency
    if let Some(p) = pairs.first() {
        let mut by: BTreeMap<String, ()> = BTreeMap::new();
        for n in &p.confirming_names {
            by.insert(n.clone(), ());
        }
        push(&mut r, format!("\n<details><summary>Sources confirming the top pair ({} → {})</summary>\n\n{}\n\n</details>", p.t, p.r, by.keys().cloned().collect::<Vec<_>>().join(", ")));
    }

    match &args.out {
        Some(path) => {
            fs::write(path, &r).unwrap_or_else(|e| die(&format!("{e}")));
            eprintln!("report written to {}", path.display());
        }
        None => print!("{r}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn p_confirm_edges() {
        assert_eq!(p_confirm(10, 0, 3), 0.0);
        assert_eq!(p_confirm(10, 6, 5), 1.0); // pigeonhole
        // W=4, a=1, b=1 -> 1/4
        assert!((p_confirm(4, 1, 1) - 0.25).abs() < 1e-12);
        // W=10,a=2,b=3: 1 - C(8,3)/C(10,3) = 1 - 56/120
        assert!((p_confirm(10, 2, 3) - (1.0 - 56.0 / 120.0)).abs() < 1e-9);
    }

    #[test]
    fn poisson_binomial_matches_binomial() {
        let ps = vec![0.5; 4];
        // P(X>=3) for Bin(4,.5) = 5/16
        assert!((poisson_binomial_tail(&ps, 3) - 5.0 / 16.0).abs() < 1e-12);
        assert!((poisson_binomial_tail(&ps, 0) - 1.0).abs() < 1e-12);
    }

    #[test]
    fn vocab_with_bom_parses() {
        let v = parse_vocab("\u{feff}tension: failure\nresolution: algorithm\n");
        assert_eq!(v.tension, vec!["failure".to_string()]);
        assert_eq!(v.resolution, vec!["algorithm".to_string()]);
    }

    #[test]
    fn vocab_parses() {
        let v = parse_vocab(DEFAULT_VOCAB);
        assert!(v.tension.contains(&"fracaso".to_string()));
        assert!(v.resolution.contains(&"algorithm".to_string()));
    }
}
