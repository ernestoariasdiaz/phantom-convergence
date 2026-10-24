//! Step 1: read passages with the local model and extract the general ideas they assert.
//!
//! Verification is mechanical, not a second opinion from the same model: for every idea
//! the model must copy, word for word, the sentence of the passage that states it, and
//! Rust checks that the sentence really is in the passage. (v0.1 asked the model
//! "does the passage say this?" and it approved 267 of 269 ideas: a rubber stamp.)
//! Everything is written to disk with its evidence, including what was rejected.

use crate::chunk::{even_sample, passages, reject_reason};
use crate::http::Endpoint;
use crate::json::{self, n, obj, s, Json};
use std::collections::{BTreeMap, HashSet};
use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

pub struct Config {
    pub corpus: PathBuf,
    pub out: PathBuf,
    pub books: Vec<String>,
    pub per_book: usize,
    pub words: usize,
    pub server: String,
    pub verify: bool,
    pub max_tokens: usize,
    pub timeout: Duration,
}

/// Minimum share of the quote's word trigrams that must appear in the passage.
pub const GROUNDING_MIN: f64 = 0.8;
/// Quotes shorter than this (in words) prove nothing.
pub const QUOTE_MIN_WORDS: usize = 6;

const EXTRACT_SYSTEM: &str = "You read passages from books and report the general ideas they state. You never add ideas of your own and you copy quotations exactly.";

fn extract_prompt(passage: &str) -> String {
    format!(
        "Passage:\n\"\"\"\n{passage}\n\"\"\"\n\n\
Report the general ideas this passage states: claims about how people, societies, nature, systems, \
machines or knowledge work, that would still make sense in a book on a completely different subject.\n\
Rules:\n\
- Most passages contain zero or one such idea. Never invent ideas to reach a number.\n\
- Not general ideas: story events, biography, examples, instructions, definitions, descriptions of a product or of the book itself.\n\
- Write each idea as one sentence in English (even if the passage is in another language), at most 25 words, without names of people, places, books or products.\n\
- After the idea write || and then the sentence of the passage that states it, copied word for word in the passage's own language.\n\
- At most 3 ideas. If there is none, answer exactly: NONE\n\
Format, one idea per line, for example:\n\
- Repeated interaction makes cooperation pay. || So long as the players expect to meet again, it pays to cooperate."
    )
}

fn chat(ep: &Endpoint, system: &str, user: &str, max_tokens: usize, timeout: Duration) -> Result<String, String> {
    let body = obj(vec![
        (
            "messages",
            Json::Arr(vec![
                obj(vec![("role", s("system")), ("content", s(system))]),
                obj(vec![("role", s("user")), ("content", s(user))]),
            ]),
        ),
        ("temperature", n(0.0)),
        ("max_tokens", n(max_tokens as f64)),
        ("stream", Json::Bool(false)),
        // thinking models (Gemma 4, Qwen3...): answer directly, no hidden reasoning
        ("chat_template_kwargs", obj(vec![("enable_thinking", Json::Bool(false))])),
    ]);
    let resp = ep.post_json("/v1/chat/completions", &body.to_string(), timeout)?;
    let v = json::parse(&resp).map_err(|e| format!("bad JSON from server: {e}"))?;
    let msg = v
        .get("choices")
        .and_then(|c| c.idx(0))
        .and_then(|c| c.get("message"))
        .ok_or_else(|| format!("unexpected server reply: {}", resp.chars().take(200).collect::<String>()))?;
    let content = strip_thinking(msg.get("content").and_then(|c| c.as_str()).unwrap_or(""));
    if content.is_empty() {
        let thought = msg.get("reasoning_content").and_then(|c| c.as_str()).map(|t| !t.trim().is_empty()).unwrap_or(false);
        return Err(if thought {
            "the model spent its whole answer on hidden reasoning and returned no text; \
             restart llama-server adding: --reasoning-budget 0"
                .to_string()
        } else {
            "the model returned an empty answer".to_string()
        });
    }
    Ok(content)
}

fn strip_thinking(t: &str) -> String {
    let mut out = t.to_string();
    for (open, close) in [("<think>", "</think>"), ("<thinking>", "</thinking>")] {
        while let (Some(a), Some(b)) = (out.find(open), out.find(close)) {
            if b < a {
                break;
            }
            out.replace_range(a..b + close.len(), "");
        }
    }
    out.trim().to_string()
}

#[derive(Debug, Clone)]
pub struct Proposed {
    pub claim: String,
    pub quote: String,
}

pub fn parse_claims(reply: &str) -> Vec<Proposed> {
    let mut seen = HashSet::new();
    let mut out = Vec::new();
    for line in reply.lines() {
        // Small models echo the template ("<idea> ... </idea>", "<...>"), number lines,
        // or use bullets. Any line with "||" is an idea; otherwise only bulleted lines.
        let mut l = line.trim().to_string();
        for tag in ["</idea>", "</idea", "<idea>", "<idea"] {
            l = l.replace(tag, " ");
        }
        let l = l.trim();
        let l = l
            .trim_start_matches(|c: char| c == '-' || c == '*' || c == '•' || c == '<' || c.is_whitespace())
            .trim_end_matches(|c: char| c == '>' || c.is_whitespace());
        let l = match l.find(|c: char| c == '.' || c == ')') {
            Some(pos) if pos > 0 && l[..pos].chars().all(|c| c.is_ascii_digit()) => l[pos + 1..].trim(),
            _ => l,
        };
        let (claim, quote) = match l.split_once("||") {
            Some((c, q)) => (c, q),
            None if line.trim_start().starts_with(['-', '*', '•']) => (l, ""),
            None => continue,
        };
        let clean = |x: &str| {
            x.trim()
                .trim_matches(|c: char| c == '"' || c == '“' || c == '”' || c == '<' || c == '>' || c.is_whitespace())
                .to_string()
        };
        let claim = clean(claim);
        let quote = clean(quote);
        let wc = claim.split_whitespace().count();
        if claim.eq_ignore_ascii_case("none") || !(5..=40).contains(&wc) {
            continue;
        }
        if seen.insert(claim.to_lowercase()) {
            out.push(Proposed { claim, quote });
        }
        if out.len() == 3 {
            break;
        }
    }
    out
}

fn words(t: &str) -> Vec<String> {
    let t = t.replace("- ", "").replace("\u{2010} ", "").replace("\u{2011} ", "");
    t.split(|c: char| !c.is_alphanumeric())
        .filter(|w| !w.is_empty())
        .map(|w| w.to_lowercase())
        .collect()
}

/// Share of the quote's word trigrams found in the passage (0..1). Robust to
/// punctuation, case and small PDF artefacts; a paraphrase or invention scores low.
pub fn grounding(quote: &str, passage: &str) -> f64 {
    let q = words(quote);
    let p = words(passage);
    if q.len() < 3 || p.len() < 3 {
        return 0.0;
    }
    let pg: HashSet<(&str, &str, &str)> = p.windows(3).map(|w| (w[0].as_str(), w[1].as_str(), w[2].as_str())).collect();
    let qg: Vec<(&str, &str, &str)> = q.windows(3).map(|w| (w[0].as_str(), w[1].as_str(), w[2].as_str())).collect();
    qg.iter().filter(|g| pg.contains(g)).count() as f64 / qg.len() as f64
}

/// None = grounded; Some(reason) = rejected.
pub fn check(p: &Proposed, passage: &str) -> (Option<&'static str>, f64) {
    if p.quote.is_empty() {
        return (Some("no quote given"), 0.0);
    }
    if words(&p.quote).len() < QUOTE_MIN_WORDS {
        return (Some("quote too short"), 0.0);
    }
    let g = grounding(&p.quote, passage);
    if g < GROUNDING_MIN {
        return (Some("quote not in passage"), g);
    }
    (None, g)
}

fn list_books(cfg: &Config) -> Result<Vec<PathBuf>, String> {
    let mut paths: Vec<PathBuf> = fs::read_dir(&cfg.corpus)
        .map_err(|e| format!("cannot read {}: {e}", cfg.corpus.display()))?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().map(|x| x == "txt").unwrap_or(false))
        .collect();
    paths.sort();
    if !cfg.books.is_empty() {
        paths.retain(|p| {
            let name = p.file_stem().unwrap().to_string_lossy().to_lowercase();
            cfg.books.iter().any(|b| name.contains(b))
        });
    }
    if paths.is_empty() {
        return Err("no matching .txt books".into());
    }
    Ok(paths)
}

fn load_done(path: &Path) -> HashSet<String> {
    let mut done = HashSet::new();
    if let Ok(f) = fs::File::open(path) {
        for line in BufReader::new(f).lines().map_while(Result::ok) {
            if let Ok(v) = json::parse(&line) {
                let ok = v.get("status").and_then(|x| x.as_str()).map(|s| s != "error").unwrap_or(false);
                if ok {
                    if let Some(k) = v.get("key").and_then(|x| x.as_str()) {
                        done.insert(k.to_string());
                    }
                }
            }
        }
    }
    done
}

fn append(path: &Path, v: &Json) {
    let mut f = OpenOptions::new().create(true).append(true).open(path).expect("cannot open output");
    writeln!(f, "{}", v.to_string()).expect("cannot write output");
}

#[derive(Default)]
struct Stats {
    passages_total: usize,
    sampled: usize,
    skipped: BTreeMap<&'static str, usize>,
    sent: usize,
    resumed: usize,
    ideas_per_passage: [usize; 4],
    proposed: usize,
    kept: usize,
    rejected: BTreeMap<&'static str, usize>,
    errors: usize,
    llm_secs: f64,
}

pub fn run(cfg: &Config) -> Result<(), String> {
    let ep = Endpoint::parse(&cfg.server)?;
    chat(&ep, "You are terse.", "Reply with exactly: OK", 16, Duration::from_secs(120))
        .map_err(|e| format!("llama-server is not answering at {}: {e}", cfg.server))?;
    fs::create_dir_all(&cfg.out).map_err(|e| e.to_string())?;
    let claims_path = cfg.out.join("claims.jsonl");
    let passages_path = cfg.out.join("passages.jsonl");
    let progress_path = cfg.out.join("progress.jsonl");
    let done = load_done(&progress_path);

    let books = list_books(cfg)?;
    let mut st = Stats::default();
    let mut per_book: Vec<(String, usize, usize, usize)> = Vec::new();

    // plan the whole run first, so the ETA is meaningful
    let mut plan: Vec<(String, Vec<crate::chunk::Passage>)> = Vec::new();
    for p in &books {
        let name = p.file_stem().unwrap().to_string_lossy().to_string();
        let bytes = fs::read(p).map_err(|e| e.to_string())?;
        let text = String::from_utf8_lossy(&bytes);
        let all = passages(&text, cfg.words);
        st.passages_total += all.len();
        let good: Vec<_> = all
            .into_iter()
            .filter(|ps| match reject_reason(&ps.text) {
                Some(r) => {
                    *st.skipped.entry(r).or_default() += 1;
                    false
                }
                None => true,
            })
            .collect();
        let pick: HashSet<usize> = even_sample(good.len(), cfg.per_book).into_iter().collect();
        let chosen: Vec<_> = good.into_iter().enumerate().filter(|(i, _)| pick.contains(i)).map(|(_, p)| p).collect();
        st.sampled += chosen.len();
        plan.push((name, chosen));
    }
    let todo: usize = plan
        .iter()
        .map(|(b, ps)| ps.iter().filter(|p| !done.contains(&format!("{b}#{}", p.index))).count())
        .sum();
    eprintln!("{} books, {} passages, {} after filters+sampling, {} still to do", plan.len(), st.passages_total, st.sampled, todo);

    let started = Instant::now();
    let mut processed = 0usize;
    for (book, chosen) in &plan {
        let (mut b_prop, mut b_kept) = (0usize, 0usize);
        for ps in chosen {
            let key = format!("{book}#{}", ps.index);
            if done.contains(&key) {
                st.resumed += 1;
                continue;
            }
            processed += 1;
            st.sent += 1;
            let t0 = Instant::now();
            let reply = match chat(&ep, EXTRACT_SYSTEM, &extract_prompt(&ps.text), cfg.max_tokens, cfg.timeout) {
                Ok(r) => r,
                Err(e) => {
                    st.errors += 1;
                    eprintln!("  error on {key}: {e}");
                    append(&progress_path, &obj(vec![("key", s(&key)), ("status", s("error")), ("error", s(&e))]));
                    if st.errors >= 5 && st.errors * 2 > st.sent {
                        return Err("too many server errors, stopping (fix the server and run again: finished passages are kept)".into());
                    }
                    continue;
                }
            };
            st.llm_secs += t0.elapsed().as_secs_f64();
            let claims = parse_claims(&reply);
            st.ideas_per_passage[claims.len().min(3)] += 1;

            let mut kept_here = 0usize;
            if !claims.is_empty() {
                append(
                    &passages_path,
                    &obj(vec![("key", s(&key)), ("book", s(book)), ("passage", n(ps.index as f64)), ("word", n(ps.start_word as f64)), ("text", s(&ps.text))]),
                );
            }
            for (i, c) in claims.iter().enumerate() {
                let (reason, g) = if cfg.verify { check(c, &ps.text) } else { (None, grounding(&c.quote, &ps.text)) };
                st.proposed += 1;
                b_prop += 1;
                match reason {
                    None => {
                        st.kept += 1;
                        b_kept += 1;
                        kept_here += 1;
                    }
                    Some(r) => *st.rejected.entry(r).or_default() += 1,
                }
                append(
                    &claims_path,
                    &obj(vec![
                        ("id", s(&format!("{key}#{i}"))),
                        ("book", s(book)),
                        ("passage", n(ps.index as f64)),
                        ("claim", s(&c.claim)),
                        ("quote", s(&c.quote)),
                        ("grounding", n((g * 100.0).round() / 100.0)),
                        ("verified", Json::Bool(reason.is_none())),
                        ("reject", reason.map(s).unwrap_or(Json::Null)),
                    ]),
                );
            }
            append(
                &progress_path,
                &obj(vec![("key", s(&key)), ("status", s(if claims.is_empty() { "none" } else { "ok" })), ("raw", s(&reply))]),
            );

            let per = started.elapsed().as_secs_f64() / processed as f64;
            let left = todo.saturating_sub(processed) as f64 * per;
            eprintln!(
                "[{processed}/{todo}] {book} #{:<5} ideas {} kept {}  | ~{:.0} min left",
                ps.index,
                claims.len(),
                kept_here,
                left / 60.0
            );
        }
        per_book.push((book.clone(), chosen.len(), b_prop, b_kept));
    }

    write_summary(cfg, &st, &per_book)?;
    Ok(())
}

fn write_summary(cfg: &Config, st: &Stats, per_book: &[(String, usize, usize, usize)]) -> Result<(), String> {
    let mut r = String::new();
    r.push_str("# ANIMUS v5 — extraction summary\n\n");
    r.push_str(&format!(
        "- Server: `{}`  ·  passage size ≈ {} words  ·  check: {}\n",
        cfg.server,
        cfg.words,
        if cfg.verify { format!("exact quote, ≥{:.0}% of word trigrams found in the passage", GROUNDING_MIN * 100.0) } else { "OFF".into() }
    ));
    r.push_str(&format!("- Passages in books: {}  ·  after filters and sampling: {}\n", st.passages_total, st.sampled));
    if !st.skipped.is_empty() {
        let sk: Vec<String> = st.skipped.iter().map(|(k, v)| format!("{k}: {v}")).collect();
        r.push_str(&format!("- Filtered out before the model: {}\n", sk.join(", ")));
    }
    r.push_str(&format!("- Sent to the model this run: {}  (resumed: {}, errors: {})\n", st.sent, st.resumed, st.errors));
    let h = st.ideas_per_passage;
    r.push_str(&format!("- Ideas per passage: 0 → {}  ·  1 → {}  ·  2 → {}  ·  3 → {}\n", h[0], h[1], h[2], h[3]));
    r.push_str(&format!("- Ideas proposed: {}  ·  kept: {}", st.proposed, st.kept));
    if cfg.verify && st.proposed > 0 {
        r.push_str(&format!("  ·  **rejected: {:.0}%**", 100.0 * (st.proposed - st.kept) as f64 / st.proposed as f64));
        let rj: Vec<String> = st.rejected.iter().map(|(k, v)| format!("{k}: {v}")).collect();
        if !rj.is_empty() {
            r.push_str(&format!(" ({})", rj.join(", ")));
        }
    }
    r.push('\n');
    if st.sent > 0 {
        r.push_str(&format!("- Model time: {:.1} s per passage\n", st.llm_secs / st.sent as f64));
    }
    r.push_str("\n| Book | Passages read | Ideas proposed | Ideas kept |\n|---|---|---|---|\n");
    for (b, p, pr, k) in per_book {
        r.push_str(&format!("| {b} | {p} | {pr} | {k} |\n"));
    }
    r.push_str("\nFiles: `claims.jsonl` (every idea with its quote, kept and rejected), `passages.jsonl` (the evidence), `progress.jsonl` (raw model replies, for audit).\n");
    let path = cfg.out.join("extract_summary.md");
    fs::write(&path, &r).map_err(|e| e.to_string())?;
    eprintln!("\n{r}\nsummary written to {}", path.display());
    Ok(())
}

pub fn ping(server: &str) -> Result<(), String> {
    let ep = Endpoint::parse(server)?;
    let t0 = Instant::now();
    let r = chat(&ep, "You are terse.", "Reply with exactly: OK", 16, Duration::from_secs(120))?;
    println!("server answered {:?} in {:.1}s", r, t0.elapsed().as_secs_f64());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const PASSAGE: &str = "The evolution of cooperation requires that individuals have a sufficiently large chance to meet again, so that they have a stake in their future interaction. When the future is not important enough, defection pays.";

    #[test]
    fn claims_parse_with_quotes() {
        let r = "Ideas:\n- Cooperation needs a real chance of meeting again. || individuals have a sufficiently large chance to meet again\n- Cooperation needs a real chance of meeting again. || duplicate\n2. Defection pays when the future matters little to the players. || \"When the future is not important enough, defection pays.\"\n- Short.\nNONE";
        let c = parse_claims(r);
        assert_eq!(c.len(), 2);
        assert_eq!(c[1].quote, "When the future is not important enough, defection pays.");
        assert!(parse_claims("NONE").is_empty());
    }

    #[test]
    fn exact_quote_is_grounded() {
        let p = Proposed { claim: "x x x x x".into(), quote: "so that they have a stake in their future interaction".into() };
        assert_eq!(check(&p, PASSAGE).0, None);
        // case and punctuation do not matter
        let p = Proposed { claim: "x".into(), quote: "WHEN THE FUTURE is not important enough -- defection pays!".into() };
        assert_eq!(check(&p, PASSAGE).0, None);
    }

    #[test]
    fn invented_or_paraphrased_quote_is_rejected() {
        let p = Proposed { claim: "x".into(), quote: "Trust between nations grows only through repeated trade agreements.".into() };
        assert_eq!(check(&p, PASSAGE).0, Some("quote not in passage"));
        let p = Proposed { claim: "x".into(), quote: "people need a big enough probability of meeting each other again".into() };
        assert_eq!(check(&p, PASSAGE).0, Some("quote not in passage"));
        let p = Proposed { claim: "x".into(), quote: "".into() };
        assert_eq!(check(&p, PASSAGE).0, Some("no quote given"));
        let p = Proposed { claim: "x".into(), quote: "defection pays".into() };
        assert_eq!(check(&p, PASSAGE).0, Some("quote too short"));
    }

    #[test]
    fn spanish_quote_works() {
        let pas = "No hay que hacer nada al azar ni de manera distinta a como lo exige la técnica de la vida.";
        assert!(grounding("No hay que hacer nada al azar", pas) > 0.99);
    }

    #[test]
    fn parses_template_echoes_from_gemma() {
        let r = "<idea> Cooperation can emerge and be stable under certain conditions. || when the conditions are present for the emergence of cooperation\n<Mutual restraint is preferred over random alternation of serious hostilities. || Moreover, both sides would prefer mutual restraint.>\n<idea> Utility for one person is also useful for others. || lo que es útil a un hombre, lo es también a otros hombres.</idea\n<idea> || claim missing here";
        let c = parse_claims(r);
        assert_eq!(c.len(), 3, "{c:?}");
        assert_eq!(c[1].claim, "Mutual restraint is preferred over random alternation of serious hostilities.");
        assert_eq!(c[2].quote, "lo que es útil a un hombre, lo es también a otros hombres.");
    }

    #[test]
    fn hyphenated_pdf_words_still_ground() {
        let pas = "people are often in situations where effective strat- egies continue to be used and ineffective strategies are dropped.";
        assert!(grounding("effective strategies continue to be used and ineffective strategies are dropped", pas) > 0.99);
    }

    #[test]
    fn strips_think() {
        assert_eq!(strip_thinking("<think>hmm</think>\n- idea"), "- idea");
    }
}
