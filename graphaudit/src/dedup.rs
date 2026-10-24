//! Audit 1: nodes that are the same thing written differently (Phantom Nodes).
//! Certain duplicates: identical after normalisation (case, accents, punctuation,
//! articles, simple plurals). Likely duplicates: a
//! threshold, never merging labels whose numbers differ ("v2" vs "v3").

use std::collections::{HashMap, HashSet};

const ARTICLES: &[&str] = &["the", "a", "an", "el", "la", "los", "las", "un", "una", "unos", "unas", "le", "les", "der", "die", "das"];

fn fold_char(c: char) -> char {
    match c {
        'á' | 'à' | 'ä' | 'â' | 'ã' | 'å' => 'a',
        'é' | 'è' | 'ë' | 'ê' => 'e',
        'í' | 'ì' | 'ï' | 'î' => 'i',
        'ó' | 'ò' | 'ö' | 'ô' | 'õ' => 'o',
        'ú' | 'ù' | 'ü' | 'û' => 'u',
        'ñ' => 'n',
        'ç' => 'c',
        _ => c,
    }
}

fn singular(w: &str) -> String {
    let n = w.chars().count();
    if n <= 3 || w.chars().any(|c| c.is_ascii_digit()) {
        return w.to_string();
    }
    if let Some(s) = w.strip_suffix("ies") {
        return format!("{s}y");
    }
    for suf in ["sses", "shes", "ches", "xes"] {
        if w.ends_with(suf) {
            return w[..w.len() - 2].to_string();
        }
    }
    if w.ends_with('s') && !w.ends_with("ss") && !w.ends_with("us") && !w.ends_with("is") {
        return w[..w.len() - 1].to_string();
    }
    w.to_string()
}

pub fn normalize(label: &str) -> String {
    let lower: String = label.to_lowercase().chars().map(fold_char).collect();
    let cleaned: String = lower.chars().map(|c| if c.is_alphanumeric() { c } else { ' ' }).collect();
    let mut toks: Vec<String> = cleaned.split_whitespace().map(singular).collect();
    while toks.len() > 1 && ARTICLES.contains(&toks[0].as_str()) {
        toks.remove(0);
    }
    toks.join(" ")
}

fn digits(s: &str) -> String {
    s.split(|c: char| !c.is_ascii_digit()).filter(|x| !x.is_empty()).collect::<Vec<_>>().join(",")
}

fn trigrams(s: &str) -> HashSet<[char; 3]> {
    let p: Vec<char> = format!("  {s} ").chars().collect();
    p.windows(3).map(|w| [w[0], w[1], w[2]]).collect()
}

/// Levenshtein distance on chars, giving up (returns max+1) once it exceeds `max`.
pub fn levenshtein(a: &str, b: &str, max: usize) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    if a.len().abs_diff(b.len()) > max {
        return max + 1;
    }
    let mut prev: Vec<usize> = (0..=b.len()).collect();
    let mut cur = vec![0usize; b.len() + 1];
    for i in 1..=a.len() {
        cur[0] = i;
        let mut row_min = cur[0];
        for j in 1..=b.len() {
            let cost = if a[i - 1] == b[j - 1] { 0 } else { 1 };
            cur[j] = (prev[j] + 1).min(cur[j - 1] + 1).min(prev[j - 1] + cost);
            row_min = row_min.min(cur[j]);
        }
        if row_min > max {
            return max + 1;
        }
        std::mem::swap(&mut prev, &mut cur);
    }
    prev[b.len()]
}

/// Edits allowed for a label of this length: 1 up to 15 chars, 2 up to 30, then 3.
pub fn allowed_edits(len: usize) -> usize {
    if len < 6 { 0 } else if len <= 15 { 1 } else if len <= 30 { 2 } else { 3 }
}

/// Possible partial-name duplicates: a one-word node A that is the first or last word
/// of exactly one other node B (and the same type, when the graph
/// has types). "CRUZ" -> "TAYLOR CRUZ" is proposed; "BASE" with five longer "... BASE"
/// names is ambiguous and is not. Returns (A, B) as indices into `reps`' node ids.
pub fn partial_names(reps: &[usize], norm: &[String], types: &[String], typed: bool) -> Vec<(usize, usize)> {
    let toks: Vec<Vec<&str>> = reps.iter().map(|&i| norm[i].split(' ').filter(|w| !w.is_empty()).collect()).collect();
    let mut by_word: HashMap<&str, Vec<usize>> = HashMap::new();
    for (k, t) in toks.iter().enumerate() {
        for w in t.iter().collect::<HashSet<_>>() {
            by_word.entry(w).or_default().push(k);
        }
    }
    let mut out = Vec::new();
    for (a, ta) in toks.iter().enumerate() {
        // only single words used alone (a surname, a first name, a bare place name):
        // longer short names collide with qualifiers like "PANEL (IN DULCE BASE)"
        if ta.len() != 1 || ta[0].chars().count() < 3 {
            continue;
        }
        let mut hits = Vec::new();
        for &b in by_word.get(ta[0]).map(|v| v.as_slice()).unwrap_or(&[]) {
            let tb = &toks[b];
            if b == a || tb.len() <= ta.len() {
                continue;
            }
            if typed && types[reps[a]] != types[reps[b]] {
                continue;
            }
            if tb[..ta.len()] == ta[..] || tb[tb.len() - ta.len()..] == ta[..] {
                hits.push(b);
            }
        }
        if hits.len() == 1 {
            out.push((reps[a], reps[hits[0]]));
        }
    }
    // people first: a surname used alone is the commonest case and the easiest to confirm
    out.sort_by_key(|&(x, _)| (types[x] != "PERSON", norm[x].clone()));
    out
}

pub struct UnionFind {
    p: Vec<usize>,
}
impl UnionFind {
    pub fn new(n: usize) -> Self {
        UnionFind { p: (0..n).collect() }
    }
    pub fn find(&mut self, x: usize) -> usize {
        let mut r = x;
        while self.p[r] != r {
            r = self.p[r];
        }
        let mut x = x;
        while self.p[x] != r {
            let nx = self.p[x];
            self.p[x] = r;
            x = nx;
        }
        r
    }
    pub fn union(&mut self, a: usize, b: usize) {
        let (ra, rb) = (self.find(a), self.find(b));
        if ra != rb {
            self.p[ra.max(rb)] = ra.min(rb);
        }
    }
}

pub struct DedupResult {
    /// node index -> canonical (certain-duplicate) group id
    pub group: Vec<usize>,
    pub n_groups: usize,
    /// groups of node indices with ≥2 members (certain duplicates)
    pub certain: Vec<Vec<usize>>,
    /// likely duplicates: (group a representative, group b representative, similarity)
    pub likely: Vec<(usize, usize, f64)>,
    pub norm: Vec<String>,
}

pub fn dedup(labels: &[String], strictness: f64) -> DedupResult {
    let n = labels.len();
    let norm: Vec<String> = labels.iter().map(|l| normalize(l)).collect();
    let mut uf = UnionFind::new(n);
    let mut first: HashMap<&str, usize> = HashMap::new();
    for (i, k) in norm.iter().enumerate() {
        if k.is_empty() {
            continue;
        }
        match first.get(k.as_str()) {
            Some(&j) => uf.union(i, j),
            None => {
                first.insert(k, i);
            }
        }
    }
    let mut map: HashMap<usize, usize> = HashMap::new();
    let mut group = vec![0usize; n];
    for i in 0..n {
        let r = uf.find(i);
        let next = map.len();
        group[i] = *map.entry(r).or_insert(next);
    }
    let n_groups = map.len();
    let mut members: Vec<Vec<usize>> = vec![Vec::new(); n_groups];
    for i in 0..n {
        members[group[i]].push(i);
    }
    let certain: Vec<Vec<usize>> = members.iter().filter(|m| m.len() > 1).cloned().collect();

    // likely duplicates among group representatives: candidates share many character
    // trigrams (fast blocking); a pair is reported only if a few single-character edits
    // turn one into the other (typos), which keeps word swaps like "credit contract" /
    // "contract credit" apart. `strictness` scales the edit budget (1.0 = default).
    let mut likely = Vec::new();
    if strictness > 0.0 {
        let reps: Vec<usize> = members.iter().map(|m| m[0]).collect();
        let grams: Vec<HashSet<[char; 3]>> = reps.iter().map(|&i| trigrams(&norm[i])).collect();
        let mut index: HashMap<[char; 3], Vec<usize>> = HashMap::new();
        for (g, set) in grams.iter().enumerate() {
            for t in set {
                index.entry(*t).or_default().push(g);
            }
        }
        let cap = 2000usize.max(n_groups / 50);
        for g in 0..reps.len() {
            let a = &norm[reps[g]];
            let la = a.chars().count();
            let mut shared: HashMap<usize, usize> = HashMap::new();
            for t in &grams[g] {
                if let Some(list) = index.get(t) {
                    if list.len() > cap {
                        continue;
                    }
                    for &h in list {
                        if h > g {
                            *shared.entry(h).or_default() += 1;
                        }
                    }
                }
            }
            for (h, cnt) in shared {
                if (cnt as f64) < 0.4 * grams[g].len().min(grams[h].len()) as f64 {
                    continue;
                }
                let b = &norm[reps[h]];
                if digits(a) != digits(b) {
                    continue;
                }
                let lb = b.chars().count();
                let budget = ((allowed_edits(la.min(lb)) as f64) * strictness).round() as usize;
                if budget == 0 {
                    continue;
                }
                let dist = levenshtein(a, b, budget);
                if dist <= budget {
                    likely.push((reps[g], reps[h], 1.0 - dist as f64 / la.max(lb) as f64));
                }
            }
        }
        likely.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap());
    }
    DedupResult { group, n_groups, certain, likely, norm }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn normalizes() {
        assert_eq!(normalize("The Neural Networks"), normalize("neural network"));
        assert_eq!(normalize("Regulación"), normalize("regulacion"));
        assert_eq!(normalize("Market-Failures!"), normalize("market failure"));
        assert_eq!(normalize("policies"), "policy");
        assert_ne!(normalize("GPT-3"), normalize("GPT-4"));
        assert_eq!(normalize("process"), "process");
    }
    #[test]
    fn partial_names_unambiguous_same_type() {
        let labels = ["CRUZ", "TAYLOR CRUZ", "BASE", "DULCE BASE", "MISSION TO DULCE BASE", "ELEVATOR", "ELEVATOR DESCENT"];
        let types = ["PERSON", "PERSON", "GEO", "GEO", "EVENT", "", "EVENT"].map(String::from).to_vec();
        let norm: Vec<String> = labels.iter().map(|l| normalize(l)).collect();
        let reps: Vec<usize> = (0..labels.len()).collect();
        let p = partial_names(&reps, &norm, &types, true);
        assert!(p.contains(&(0, 1)));          // CRUZ -> TAYLOR CRUZ
        assert!(p.contains(&(2, 3)));          // BASE -> DULCE BASE (only GEO superset)
        assert!(!p.iter().any(|&(a, _)| a == 3)); // two-word names are never the short side
        assert!(!p.iter().any(|&(a, _)| a == 5)); // untyped ELEVATOR vs EVENT: no
        let untyped = vec![String::new(); labels.len()];
        let q = partial_names(&reps, &norm, &untyped, false);
        assert!(!q.iter().any(|&(a, _)| a == 2)); // BASE is ambiguous without types
    }

    #[test]
    fn word_swap_is_not_a_typo() {
        let l: Vec<String> = ["credit contract", "contract credit", "market failure", "market falure"].iter().map(|s| s.to_string()).collect();
        let r = dedup(&l, 1.0);
        assert_eq!(r.likely.len(), 1);
        assert_eq!(l[r.likely[0].1], "market falure");
        assert_eq!(levenshtein("kitten", "sitting", 5), 3);
    }

    #[test]
    fn finds_certain_and_likely() {
        let l: Vec<String> = ["Machine Learning", "machine-learning", "Machine learnin", "Deep Learning", "Version 2", "Version 3"]
            .iter().map(|s| s.to_string()).collect();
        let r = dedup(&l, 1.0);
        assert_eq!(r.n_groups, 5);
        assert_eq!(r.certain.len(), 1);
        assert!(r.likely.iter().any(|&(a, b, _)| (l[a].starts_with("Machine") || l[a].starts_with("machine")) && l[b] == "Machine learnin"));
        assert!(!r.likely.iter().any(|&(a, b, _)| l[a].starts_with("Version") && l[b].starts_with("Version")));
    }
}
