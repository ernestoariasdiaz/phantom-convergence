//! Audit 1: nodes that are the same thing written differently (Phantom Nodes).
//! Certain duplicates: identical after normalising case, accents, punctuation and
//! leading articles, and of the same type when the graph has types.
//! Likely duplicates (for review): singular/plural forms ("THE BOY" / "THE BOYS" can be
//! different characters) and one-typo variants; never pairs that differ in a number
//! ("v2" / "v3"), in an honorific ("MR. FEZZIWIG" / "MRS. FEZZIWIG") or in type.

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

const HONORIFICS: &[&str] = &["mr", "mrs", "ms", "miss", "dr", "sir", "lady", "lord", "madam", "sr", "sra", "srta", "don", "dona"];

/// Case, accents, punctuation, leading articles. Plurals are kept: they decide nothing alone.
pub fn normalize(label: &str) -> String {
    let lower: String = label.to_lowercase().chars().map(fold_char).collect();
    // '+' and '#' name different things: "Apple TV" / "Apple TV+", "C" / "C#" / "C++"
    let cleaned: String = lower.chars().map(|c| if c.is_alphanumeric() || c == '+' || c == '#' { c } else { ' ' }).collect();
    let mut toks: Vec<String> = cleaned.split_whitespace().map(|w| w.to_string()).collect();
    while toks.len() > 1 && ARTICLES.contains(&toks[0].as_str()) {
        toks.remove(0);
    }
    toks.join(" ")
}

/// `normalize` plus simple singularisation, used only to propose singular/plural pairs.
pub fn plural_key(norm: &str) -> String {
    norm.split(' ').map(singular).collect::<Vec<_>>().join(" ")
}

/// True if the two names differ in an honorific ("mr" vs "mrs"): different people.
fn honorific_differs(a: &str, b: &str) -> bool {
    let ta: HashSet<&str> = a.split(' ').collect();
    let tb: HashSet<&str> = b.split(' ').collect();
    ta.symmetric_difference(&tb).any(|w| HONORIFICS.contains(w))
}

/// Leading titles that name the same person with or without them ("KING EDWARD IV" / "EDWARD IV").
const TITLES: &[&str] = &["king", "queen", "prince", "princess", "emperor", "empress", "pope", "saint", "st",
    "president", "general", "captain", "rey", "reina", "san", "santa"];

/// Types that say nothing ("UNKNOWN") are compatible with any type. An empty type (a node missing
/// from the node table) is not: nothing is known about it at all.
pub fn type_compatible(a: &str, b: &str) -> bool {
    let vague = |t: &str| matches!(t.trim().to_lowercase().as_str(), "unknown" | "other" | "none" | "null" | "entity_type" | "entity");
    a == b || vague(a) || vague(b)
}

/// "MOVABLE PROPERTY" / "IMMOVABLE PROPERTY": one edit apart, opposite meanings.
fn negation_pair(a: &str, b: &str) -> bool {
    const NEG: &[&str] = &["im", "in", "il", "ir", "un", "non", "dis", "anti", "a"];
    let ta: Vec<&str> = a.split(' ').collect();
    let tb: Vec<&str> = b.split(' ').collect();
    ta.len() == tb.len()
        && ta.iter().zip(&tb).any(|(x, y)| {
            x != y && NEG.iter().any(|p| y.strip_prefix(p) == Some(*x) || x.strip_prefix(p) == Some(*y))
        })
}

/// Guards for same-meaning proposals: numbers and model markers may be missing on one side
/// ("HENRY TUDOR" / "HENRY VII") but never different on both ("HENRY VI" / "HENRY VII");
/// same honorifics; not opposites.
/// But a name that is the other plus a number is a numbered member of a series ("CATAREY" / "CATAREY I",
/// "ETAPA" / "II ETAPA"): never paired.
pub fn names_may_match(a_norm: &str, b_norm: &str) -> bool {
    let (da, db) = (digits(a_norm), digits(b_norm));
    let series = da != db && without_markers(a_norm) == without_markers(b_norm);
    (da == db || da.is_empty() || db.is_empty()) && !series && !honorific_differs(a_norm, b_norm) && !negation_pair(a_norm, b_norm)
}

fn possessive(raw: &str) -> bool {
    let l = raw.to_lowercase();
    l.contains("'s ") || l.contains("’s ") || l.ends_with("'s") || l.ends_with("’s") || l.contains("s' ")
}

/// A regnal or ordinal Roman numeral from 1 to 39 ("ii", "vii", "xiv"), not a word like "civil" or "mix".
fn roman(w: &str) -> bool {
    const ONES: [&str; 10] = ["", "i", "ii", "iii", "iv", "v", "vi", "vii", "viii", "ix"];
    // OCR reads "III" as "ILL" and "II" as "LL"
    (0..4).any(|t| ONES.iter().any(|o| !(t == 0 && o.is_empty()) && w == format!("{}{}", "x".repeat(t), o)))
        || matches!(w, "ill" | "il" | "ll" | "lll")
}

/// The name with its numbers and model markers removed: "CATAREY I" -> "catarey".
fn without_markers(norm: &str) -> String {
    norm.split(' ')
        .filter(|w| !(w.chars().any(|c| c.is_ascii_digit() || c == '+' || c == '#') || roman(w) || w.chars().count() == 1))
        .collect::<Vec<_>>()
        .join(" ")
}

/// Numbers and model markers in a name: tokens with a digit, '+' or '#', and Roman numerals.
/// Single letters too ("GROUP A" / "GROUP B").
/// "GPT-3" / "GPT-4", "HENRY VI" / "HENRY VII", "IPHONE 4" / "IPHONE 4S", "ICLOUD" / "ICLOUD+" never pair.
fn digits(s: &str) -> String {
    let mut out: Vec<String> = s
        .split(' ')
        .filter(|w| w.chars().any(|c| c.is_ascii_digit() || c == '+' || c == '#') || roman(w) || w.chars().count() == 1)
        .map(String::from)
        .collect();
    out.sort();
    out.join(",")
}

fn trigrams(s: &str) -> HashSet<[char; 3]> {
    let p: Vec<char> = format!("  {s} ").chars().collect();
    p.windows(3).map(|w| [w[0], w[1], w[2]]).collect()
}

/// Same stem, a different last letter: another word, not a typo ("ACTIVIST" / "ACTIVISM",
/// "ORGANIZER" / "ORGANIZED"). Only the last letter: "SYSTM", "FEEDBAK" (letters dropped) and
/// "GENTLEMEN" / "GENTLEMAN" (a vowel inside) stay proposed.
fn other_suffix(x: &str, y: &str) -> bool {
    let (a, b): (Vec<char>, Vec<char>) = (x.chars().collect(), y.chars().collect());
    let common = a.iter().zip(&b).take_while(|(p, q)| p == q).count();
    let (ra, rb) = (a.len() - common, b.len() - common);
    common >= 5 && ra == 1 && rb == 1
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
pub fn partial_names(reps: &[usize], norm: &[String], raw: &[String], types: &[String], typed: bool) -> Vec<(usize, usize)> {
    let toks: Vec<Vec<&str>> = reps.iter().map(|&i| norm[i].split(' ').filter(|w| !w.is_empty()).collect()).collect();
    let mut by_word: HashMap<&str, Vec<usize>> = HashMap::new();
    for (k, t) in toks.iter().enumerate() {
        for w in t.iter().collect::<HashSet<_>>() {
            by_word.entry(w).or_default().push(k);
        }
    }
    let all_lower = reps.iter().all(|&i| !raw[i].chars().any(|c| c.is_uppercase()));
    let mut out = Vec::new();
    for (a, ta) in toks.iter().enumerate() {
        // only single words used alone (a surname, a first name, a bare place name):
        // longer short names collide with qualifiers like "PANEL (IN DULCE BASE)"
        // a title alone ("LORD", "KING") is not a short name for whoever holds it
        if ta.len() != 1 || ta[0].chars().count() < 3 || ta[0].chars().all(|c| c.is_ascii_digit())
            || HONORIFICS.contains(&ta[0]) || TITLES.contains(&ta[0])
        {
            continue;
        }
        // a name, not a common noun: "Jobs", "CRUZ" yes; "ads", "business" no. A graph written all in
        // lower case carries no such signal: tried on five such graphs, all 27 proposals were a general
        // and a specific concept ("communications" / "global communications"), so there only names typed
        // as people are checked ("musk" / "elon musk").
        let person = |t: &str| matches!(t.to_uppercase().as_str(), "PERSON" | "PEOPLE" | "CHARACTER");
        if (all_lower && !(typed && person(&types[reps[a]]))) || (!all_lower && !raw[reps[a]].trim().chars().next().map_or(false, |c| c.is_uppercase())) {
            continue;
        }
        let mut hits = Vec::new();
        for &b in by_word.get(ta[0]).map(|v| v.as_slice()).unwrap_or(&[]) {
            let tb = &toks[b];
            if b == a || tb.len() <= ta.len() {
                continue;
            }
            if typed && !type_compatible(&types[reps[a]], &types[reps[b]]) {
                continue;
            }
            // "CAROLINE'S HUSBAND" is someone else, related to Caroline
            if possessive(&raw[reps[b]]) {
                continue;
            }
            if tb[..ta.len()] == ta[..] || tb[tb.len() - ta.len()..] == ta[..] {
                hits.push(b);
            }
        }
        // "JOBS" -> "STEVE JOBS" and "APPLE CO-FOUNDER STEVE JOBS": if every longer
        // name contains the shortest one, they are one name with more qualifiers
        if hits.len() > 1 {
            hits.sort_by_key(|&b| toks[b].len());
            let base = &toks[hits[0]];
            let nested = hits[1..].iter().all(|&b| toks[b].windows(base.len()).any(|w| w == &base[..]))
                && hits[1..].iter().all(|&b| toks[b].len() > base.len());
            if nested {
                hits.truncate(1);
            }
        }
        // "ETAPA" / "II ETAPA", "DAVID" / "DAVID 0607010100401": the longer name only adds a number, so it
        // is a numbered member of a series. It still counts above (so "HENRY" with "HENRY VI", "HENRY VII"
        // and "HENRY TUDOR" stays ambiguous) but is never proposed.
        if hits.len() == 1 && without_markers(&norm[reps[hits[0]]]) != norm[reps[a]] {
            out.push((reps[a], reps[hits[0]]));
        }
    }
    // people first: a surname used alone is the commonest case and the easiest to confirm
    out.sort_by_key(|&(x, _)| (types[x] != "PERSON", norm[x].clone()));
    out
}

/// Candidate pairs for a judge: two names of compatible type that share a rare word
/// ("HENRY TUDOR" / "HENRY VII", "DUKE OF YORK" / "RICHARD, DUKE OF YORK"). A word shared by
/// more than `max_df` names ("LAW", "PROPERTY") is too common to suggest anything.
pub fn shared_word_pairs(reps: &[usize], norm: &[String], types: &[String], typed: bool, max_df: usize) -> Vec<(usize, usize, usize)> {
    const STOP: &[&str] = &["of", "and", "the", "in", "on", "for", "to", "de", "la", "el", "los", "las", "del", "y", "a", "an"];
    let mut by_word: HashMap<&str, Vec<usize>> = HashMap::new();
    for &r in reps {
        for w in norm[r].split(' ').collect::<HashSet<_>>() {
            if w.chars().count() >= 3 && !STOP.contains(&w) && !roman(w) {
                by_word.entry(w).or_default().push(r);
            }
        }
    }
    let mut seen: HashSet<(usize, usize)> = HashSet::new();
    let mut out = Vec::new();
    for v in by_word.values() {
        if v.len() < 2 || v.len() > max_df {
            continue;
        }
        for i in 0..v.len() {
            for j in (i + 1)..v.len() {
                let (x, y) = (v[i].min(v[j]), v[i].max(v[j]));
                if typed && !type_compatible(&types[x], &types[y]) {
                    continue;
                }
                if seen.insert((x, y)) {
                    out.push((x, y, v.len()));
                }
            }
        }
    }
    out.sort_by_key(|&(x, y, df)| (df, x, y));
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
    /// likely duplicates: (group a representative, group b representative, similarity, reason)
    pub likely: Vec<(usize, usize, f64, &'static str)>,
    pub norm: Vec<String>,
}

pub fn dedup(labels: &[String], types: &[String], strictness: f64) -> DedupResult {
    let n = labels.len();
    let typed = types.iter().any(|t| !t.is_empty());
    let ty = |i: usize| -> &str { if typed { types[i].as_str() } else { "" } };
    let norm: Vec<String> = labels.iter().map(|l| normalize(l)).collect();
    let mut uf = UnionFind::new(n);
    let mut first: HashMap<(String, String), usize> = HashMap::new();
    for (i, k) in norm.iter().enumerate() {
        if k.is_empty() {
            continue;
        }
        match first.get(&(k.clone(), ty(i).to_string())) {
            Some(&j) => uf.union(i, j),
            None => {
                first.insert((k.clone(), ty(i).to_string()), i);
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
    let mut likely: Vec<(usize, usize, f64, &'static str)> = Vec::new();
    let reps: Vec<usize> = members.iter().map(|m| m[0]).collect();
    // singular / plural forms of the same name and type: often the same thing, sometimes
    // not ("THE BOY" and "THE BOYS" are different characters), so they are only proposed
    // (also: the same name where one copy has a vague type such as UNKNOWN)
    let mut by_plural: HashMap<String, Vec<usize>> = HashMap::new();
    for &r in &reps {
        by_plural.entry(plural_key(&norm[r])).or_default().push(r);
    }
    for v in by_plural.values() {
        for i in 0..v.len() {
            for j in (i + 1)..v.len() {
                if !type_compatible(ty(v[i]), ty(v[j])) {
                    continue;
                }
                let why = if norm[v[i]] == norm[v[j]] { "same name, type unknown" } else { "singular/plural" };
                likely.push((v[i], v[j], 1.0, why));
            }
        }
    }
    // a company with and without its legal form: "APPLE" / "APPLE INC."
    const LEGAL: &[&str] = &["inc", "incorporated", "corp", "corporation", "ltd", "limited", "llc", "plc", "gmbh", "ag", "sa", "srl", "co", "company"];
    let by_norm0: HashMap<&str, Vec<usize>> = reps.iter().fold(HashMap::new(), |mut m, &r| {
        m.entry(norm[r].as_str()).or_insert_with(Vec::new).push(r);
        m
    });
    for &r in &reps {
        let toks: Vec<&str> = norm[r].split(' ').collect();
        // "Fortune 500 company" is a description, not a legal name: the form must be capitalised
        let last_raw = labels[r].split(|c: char| !c.is_alphanumeric()).filter(|w| !w.is_empty()).last().unwrap_or("");
        if toks.len() >= 2 && LEGAL.contains(toks.last().unwrap()) && last_raw.chars().next().map_or(false, |c| c.is_uppercase()) {
            let rest = toks[..toks.len() - 1].join(" ");
            for &o in by_norm0.get(rest.as_str()).map(|v| v.as_slice()).unwrap_or(&[]) {
                if o != r && type_compatible(ty(r), ty(o)) {
                    likely.push((o, r, 1.0, "legal form"));
                }
            }
        }
    }
    // a leading title: "KING EDWARD IV" / "EDWARD IV"
    let by_norm: HashMap<&str, Vec<usize>> = reps.iter().fold(HashMap::new(), |mut m, &r| {
        m.entry(norm[r].as_str()).or_insert_with(Vec::new).push(r);
        m
    });
    for &r in &reps {
        let toks: Vec<&str> = norm[r].split(' ').collect();
        if toks.len() >= 2 && TITLES.contains(&toks[0]) {
            let rest = toks[1..].join(" ");
            for &o in by_norm.get(rest.as_str()).map(|v| v.as_slice()).unwrap_or(&[]) {
                if o != r && type_compatible(ty(r), ty(o)) {
                    likely.push((o, r, 1.0, "title"));
                }
            }
        }
    }
    if strictness > 0.0 {
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
                if digits(a) != digits(b) || !type_compatible(ty(reps[g]), ty(reps[h])) || honorific_differs(a, b) || negation_pair(a, b) {
                    continue;
                }
                if plural_key(a) == plural_key(b) {
                    continue; // already proposed as singular/plural
                }
                let lb = b.chars().count();
                let budget = ((allowed_edits(la.min(lb)) as f64) * strictness).round() as usize;
                if budget == 0 {
                    continue;
                }
                // word by word: a typo changes a word a little, not into another word
                // ("SOUTH AMERICAN" / "NORTH AMERICAN" is two edits in a five-letter word)
                let (wa, wb): (Vec<&str>, Vec<&str>) = (a.split(' ').collect(), b.split(' ').collect());
                if wa.len() == wb.len()
                    && wa.iter().zip(&wb).any(|(x, y)| x != y && (levenshtein(x, y, 2) > if x.chars().count().max(y.chars().count()) <= 7 { 1 } else { 2 } || other_suffix(x, y) || (x.chars().count() <= 3 && y.chars().count() <= 3)))
                {
                    continue;
                }
                let dist = levenshtein(a, b, budget);
                if dist <= budget {
                    likely.push((reps[g], reps[h], 1.0 - dist as f64 / la.max(lb) as f64, "typo"));
                }
            }
        }
    }
    likely.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap());
    DedupResult { group, n_groups, certain, likely, norm }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn normalizes() {
        assert_eq!(plural_key(&normalize("The Neural Networks")), plural_key(&normalize("neural network")));
        assert_ne!(normalize("The Boys"), normalize("the boy"));
        assert_eq!(normalize("The Ghost of Christmas Past"), normalize("GHOST OF CHRISTMAS PAST"));
        assert_eq!(normalize("Regulación"), normalize("regulacion"));
        assert_eq!(normalize("Market-Failure!"), normalize("market failure"));
        assert_eq!(plural_key("policies"), "policy");
        assert_ne!(normalize("GPT-3"), normalize("GPT-4"));
        assert_eq!(normalize("process"), "process");
    }
    #[test]
    fn partial_names_unambiguous_same_type() {
        let labels = ["CRUZ", "TAYLOR CRUZ", "BASE", "DULCE BASE", "MISSION TO DULCE BASE", "ELEVATOR", "ELEVATOR DESCENT"];
        let types = ["PERSON", "PERSON", "GEO", "GEO", "EVENT", "", "EVENT"].map(String::from).to_vec();
        let norm: Vec<String> = labels.iter().map(|l| normalize(l)).collect();
        let reps: Vec<usize> = (0..labels.len()).collect();
        let raw: Vec<String> = labels.iter().map(|s| s.to_string()).collect();
        let p = partial_names(&reps, &norm, &raw, &types, true);
        assert!(p.contains(&(0, 1)));          // CRUZ -> TAYLOR CRUZ
        assert!(p.contains(&(2, 3)));          // BASE -> DULCE BASE (only GEO superset)
        assert!(!p.iter().any(|&(a, _)| a == 3)); // two-word names are never the short side
        assert!(!p.iter().any(|&(a, _)| a == 5)); // untyped ELEVATOR vs EVENT: no
        let untyped = vec![String::new(); labels.len()];
        let q = partial_names(&reps, &norm, &raw, &untyped, false);
        assert!(q.contains(&(2, 3))); // untyped: MISSION TO DULCE BASE contains DULCE BASE, so one name
        let l2 = ["CAROLINE", "CAROLINE'S HUSBAND", "BOB", "BOB CRATCHIT"];
        let n2: Vec<String> = l2.iter().map(|l| normalize(l)).collect();
        let r2: Vec<String> = l2.iter().map(|s| s.to_string()).collect();
        let t2 = vec!["PERSON".to_string(); 4];
        let p2 = partial_names(&[0, 1, 2, 3], &n2, &r2, &t2, true);
        assert_eq!(p2, vec![(2, 3)]); // BOB -> BOB CRATCHIT, but not CAROLINE -> her husband
    }

    #[test]
    fn honorifics_types_and_plurals() {
        let l: Vec<String> = ["MR. FEZZIWIG", "MRS. FEZZIWIG", "THE PHANTOM", "THE PHANTOMS", "THE BOY", "THE BOYS", "The Spirit", "SPIRIT"]
            .iter().map(|s| s.to_string()).collect();
        let t: Vec<String> = ["PERSON", "PERSON", "PERSON", "GEO", "PERSON", "PERSON", "PERSON", "PERSON"].iter().map(|s| s.to_string()).collect();
        let r = dedup(&l, &t, 1.0);
        assert_eq!(r.certain.len(), 1); // only THE SPIRIT / SPIRIT
        assert!(!r.likely.iter().any(|&(a, b, _, _)| l[a].contains("FEZZIWIG") && l[b].contains("FEZZIWIG")));
        assert!(!r.likely.iter().any(|&(a, b, _, _)| l[a].contains("PHANTOM") && l[b].contains("PHANTOM"))); // types differ
        assert!(r.likely.iter().any(|&(a, b, _, why)| l[a] == "THE BOY" && l[b] == "THE BOYS" && why == "singular/plural"));
    }

    #[test]
    fn negation_titles_and_unknown_types() {
        let l: Vec<String> = ["Movable Property", "Immovable Property", "Edward IV", "King Edward IV", "Henry", "Henry VI", "Henry VII", "Margaret", "Margaret of Anjou"]
            .iter().map(|s| s.to_string()).collect();
        let t: Vec<String> = ["concept", "concept", "person", "UNKNOWN", "UNKNOWN", "person", "UNKNOWN", "person", "person"]
            .iter().map(|s| s.to_string()).collect();
        let r = dedup(&l, &t, 1.0);
        assert!(!r.likely.iter().any(|&(a, b, _, _)| (a.min(b), a.max(b)) == (0, 1)));
        assert!(r.likely.iter().any(|&(a, b, _, w)| (a.min(b), a.max(b)) == (2, 3) && w == "title"));
        let reps: Vec<usize> = (0..l.len()).collect();
        let p = partial_names(&reps, &r.norm, &l, &t, true);
        assert!(!r.likely.iter().any(|&(a, b, _, _)| (a.min(b), a.max(b)) == (5, 6))); // Henry VI / VII
        assert!(roman("vii") && roman("xiv") && !roman("civil") && !roman("mix") && roman("x"));
        assert!(p.contains(&(7, 8)));
        assert!(!p.iter().any(|&(a, _)| a == 4)); // Henry: VI, VII -> ambiguous
    }

    #[test]
    fn plus_legal_form_and_nested_names() {
        assert_ne!(normalize("Apple TV+"), normalize("Apple TV"));
        assert_ne!(normalize("C#"), normalize("C++"));
        let l: Vec<String> = ["Apple", "Apple Inc.", "Jobs", "Steve Jobs", "Apple co-founder Steve Jobs", "ads", "Power Mac ads", "2020", "September 2020"]
            .iter().map(|s| s.to_string()).collect();
        let t = vec![String::new(); l.len()];
        let r = dedup(&l, &t, 1.0);
        assert!(r.likely.iter().any(|&(a, b, _, w)| (a.min(b), a.max(b)) == (0, 1) && w == "legal form"));
        let reps: Vec<usize> = (0..l.len()).collect();
        let p = partial_names(&reps, &r.norm, &l, &t, false);
        assert!(p.contains(&(2, 3)));                 // Jobs -> Steve Jobs
        let l3: Vec<String> = ["iPhone 4", "iPhone 4s", "iCloud", "iCloud+", "Fortune 500", "Fortune 500 company"].iter().map(|s| s.to_string()).collect();
        let r3 = dedup(&l3, &vec![String::new(); 6], 1.0);
        assert!(r3.likely.is_empty(), "{:?}", r3.likely);
        let l4: Vec<String> = ["Group A", "Group B", "South American teams", "North American teams", "trust state", "trust stte"].iter().map(|s| s.to_string()).collect();
        let r4 = dedup(&l4, &vec![String::new(); 6], 1.0);
        assert_eq!(r4.likely.iter().map(|&(a, b, _, _)| (a.min(b), a.max(b))).collect::<Vec<_>>(), vec![(4, 5)]);
        assert!(!p.iter().any(|&(a, _)| a == 5 || a == 7)); // common noun, bare year
    }

    #[test]
    fn semantic_guards() {
        assert!(names_may_match(&normalize("Henry Tudor"), &normalize("Henry VII")));
        assert!(!names_may_match(&normalize("Henry VI"), &normalize("Henry VII")));
        assert!(!names_may_match(&normalize("Mr. Fezziwig"), &normalize("Mrs. Fezziwig")));
    }

    #[test]
    fn shared_rare_words() {
        let l = ["Henry Tudor", "Henry VII", "Property Law", "Civil Law", "Roman Law"];
        let norm: Vec<String> = l.iter().map(|x| normalize(x)).collect();
        let t = vec![String::new(); 5];
        let p = shared_word_pairs(&[0, 1, 2, 3, 4], &norm, &t, false, 2);
        assert_eq!(p.iter().map(|&(a, b, _)| (a, b)).collect::<Vec<_>>(), vec![(0, 1)]); // "law" is in 3 names
    }

    #[test]
    fn numbered_series_and_ocr_numerals() {
        assert!(!names_may_match(&normalize("Catarey"), &normalize("Catarey I")));
        assert!(!names_may_match(&normalize("Parque del Este ILL"), &normalize("Parque del Este I")));
        assert!(!names_may_match(&normalize("Parque del Este ILL"), &normalize("Parque del Este IL")));
        assert!(names_may_match(&normalize("Henry Tudor"), &normalize("Henry VII")));
        let l: Vec<String> = ["Etapa", "II Etapa"].iter().map(|s| s.to_string()).collect();
        let n: Vec<String> = l.iter().map(|x| normalize(x)).collect();
        assert!(partial_names(&[0, 1], &n, &l, &vec![String::new(); 2], false).is_empty());
    }

    #[test]
    fn lowercase_graphs_and_word_endings() {
        let l: Vec<String> = ["curie", "marie curie", "activist", "activism", "trust state", "trust stte"].iter().map(|s| s.to_string()).collect();
        let t = vec![String::new(); l.len()];
        let r = dedup(&l, &t, 1.0);
        assert!(!r.likely.iter().any(|&(a, b, _, _)| (a.min(b), a.max(b)) == (2, 3)));
        assert!(r.likely.iter().any(|&(a, b, _, _)| (a.min(b), a.max(b)) == (4, 5)));
        let reps: Vec<usize> = (0..l.len()).collect();
        assert!(partial_names(&reps, &r.norm, &l, &t, false).is_empty()); // all lower case, untyped: not assessed
        let tp: Vec<String> = ["person", "person", "concept", "concept", "concept", "concept"].iter().map(|s| s.to_string()).collect();
        assert_eq!(partial_names(&reps, &r.norm, &l, &tp, true), vec![(0, 1)]); // people only
        let g: Vec<String> = ["uk government", "us government", "bob cratchit", "boy cratchit"].iter().map(|s| s.to_string()).collect();
        assert!(dedup(&g, &vec![String::new(); 4], 1.0).likely.is_empty());
        let lm: Vec<String> = ["LORD", "LORD MAYOR"].iter().map(|s| s.to_string()).collect();
        let nl: Vec<String> = lm.iter().map(|x| normalize(x)).collect();
        assert!(partial_names(&[0, 1], &nl, &lm, &vec![String::new(); 2], false).is_empty());
    }

    #[test]
    fn word_swap_is_not_a_typo() {
        let l: Vec<String> = ["credit contract", "contract credit", "market failure", "market falure"].iter().map(|s| s.to_string()).collect();
        let r = dedup(&l, &vec![String::new(); l.len()], 1.0);
        assert_eq!(r.likely.len(), 1);
        assert_eq!(l[r.likely[0].1], "market falure");
        assert_eq!(levenshtein("kitten", "sitting", 5), 3);
    }

    #[test]
    fn finds_certain_and_likely() {
        let l: Vec<String> = ["Machine Learning", "machine-learning", "Machine learnin", "Deep Learning", "Version 2", "Version 3"]
            .iter().map(|s| s.to_string()).collect();
        let r = dedup(&l, &vec![String::new(); l.len()], 1.0);
        assert_eq!(r.n_groups, 5);
        assert_eq!(r.certain.len(), 1);
        assert!(r.likely.iter().any(|&(a, b, _, _)| (l[a].starts_with("Machine") || l[a].starts_with("machine")) && l[b] == "Machine learnin"));
        assert!(!r.likely.iter().any(|&(a, b, _, _)| l[a].starts_with("Version") && l[b].starts_with("Version")));
    }
}
