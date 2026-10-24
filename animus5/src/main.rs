//! ANIMUS v5 — a knowledge graph that has to prove each connection is not chance.
//!
//! Step 1 `extract`: read passages with a local model (llama-server) and keep the
//! general ideas each passage states, each proven by an exact quote.
//! Step 2 `link`: find ideas from different books that say the same thing, using a
//! multilingual embedding model, and count how many such links word-shuffled decoys
//! produce under the very same procedure.

mod chunk;
mod extract;
mod http;
mod json;
mod link;

use std::env;
use std::path::PathBuf;
use std::time::Duration;

const HELP: &str = "animus5 — ANIMUS v5: a knowledge graph that has to prove each connection is not chance

USAGE:
  animus5 ping    [--server URL]
  animus5 extract <txt_dir> [options]     step 1: ideas, each proven by an exact quote
  animus5 link    [options]               step 2: same idea in different books, vs. decoys

link options:
  --out DIR          folder with claims.jsonl from extract (default: animus5_out)
  --server URL       embedding server, llama-server --embeddings (default http://127.0.0.1:8081)
  --percentile P     similarity threshold as a percentile of cross-book pairs (default 99)
  --decoys K         word-shuffled decoy runs (default 3)
  --decoy-mode M     words: shuffle all words (default) · content: keep not/no/there/is… in place,
                     shuffle only content words (controls for grammatical form, e.g. negation)
  --top N            links to show in the report (default 25)

extract options:
  --out DIR          output folder (default: animus5_out)
  --books a,b,c      only books whose file name contains one of these (case-insensitive)
  --per-book N       read N passages per book, evenly spread (default 30; 0 = all)
  --words N          passage size in words (default 180)
  --server URL       llama-server (default http://127.0.0.1:8080)
  --no-verify        keep ideas even if their quote is not found in the passage (not recommended)
  --max-tokens N     max tokens per extraction reply (default 400)
  --timeout S        seconds per model call (default 300)

Runs are resumable: run the same command again and finished passages are skipped.";

fn die(msg: &str) -> ! {
    eprintln!("animus5: {msg}");
    std::process::exit(2);
}

fn main() {
    let mut args = env::args().skip(1);
    let cmd = args.next().unwrap_or_default();
    let rest: Vec<String> = args.collect();
    let mut server = "http://127.0.0.1:8080".to_string();

    match cmd.as_str() {
        "ping" => {
            let mut it = rest.into_iter();
            while let Some(a) = it.next() {
                if a == "--server" {
                    server = it.next().unwrap_or_else(|| die("--server needs a value"));
                }
            }
            if let Err(e) = extract::ping(&server) {
                die(&e);
            }
        }
        "extract" => {
            let mut cfg = extract::Config {
                corpus: PathBuf::new(),
                out: PathBuf::from("animus5_out"),
                books: Vec::new(),
                per_book: 30,
                words: 180,
                server,
                verify: true,
                max_tokens: 400,
                timeout: Duration::from_secs(300),
            };
            let mut corpus = None;
            let mut it = rest.into_iter();
            while let Some(a) = it.next() {
                let mut val = |name: &str| it.next().unwrap_or_else(|| die(&format!("{name} needs a value")));
                match a.as_str() {
                    "--out" => cfg.out = val("--out").into(),
                    "--books" => {
                        cfg.books = val("--books").split(',').map(|b| b.trim().to_lowercase()).filter(|b| !b.is_empty()).collect()
                    }
                    "--per-book" => cfg.per_book = val("--per-book").parse().unwrap_or_else(|_| die("bad --per-book")),
                    "--words" => cfg.words = val("--words").parse().unwrap_or_else(|_| die("bad --words")),
                    "--server" => cfg.server = val("--server"),
                    "--no-verify" => cfg.verify = false,
                    "--max-tokens" => cfg.max_tokens = val("--max-tokens").parse().unwrap_or_else(|_| die("bad --max-tokens")),
                    "--timeout" => cfg.timeout = Duration::from_secs(val("--timeout").parse().unwrap_or_else(|_| die("bad --timeout"))),
                    s if s.starts_with("--") => die(&format!("unknown option {s}")),
                    s => corpus = Some(PathBuf::from(s)),
                }
            }
            cfg.corpus = corpus.unwrap_or_else(|| die("missing <txt_dir>"));
            if let Err(e) = extract::run(&cfg) {
                die(&e);
            }
        }
        "link" => {
            let mut cfg = link::Config {
                out: PathBuf::from("animus5_out"),
                server: "http://127.0.0.1:8081".to_string(),
                percentile: 99.0,
                decoys: 3,
                decoy_mode: "words".to_string(),
                seed: 20261023,
                top: 25,
            };
            let mut it = rest.into_iter();
            while let Some(a) = it.next() {
                let mut val = |name: &str| it.next().unwrap_or_else(|| die(&format!("{name} needs a value")));
                match a.as_str() {
                    "--out" => cfg.out = val("--out").into(),
                    "--server" => cfg.server = val("--server"),
                    "--percentile" => cfg.percentile = val("--percentile").parse().unwrap_or_else(|_| die("bad --percentile")),
                    "--decoys" => cfg.decoys = val("--decoys").parse::<usize>().unwrap_or_else(|_| die("bad --decoys")).max(1),
                    "--decoy-mode" => {
                        cfg.decoy_mode = val("--decoy-mode");
                        if cfg.decoy_mode != "words" && cfg.decoy_mode != "content" {
                            die("--decoy-mode must be 'words' or 'content'");
                        }
                    }
                    "--top" => cfg.top = val("--top").parse().unwrap_or_else(|_| die("bad --top")),
                    "--seed" => cfg.seed = val("--seed").parse().unwrap_or_else(|_| die("bad --seed")),
                    s => die(&format!("unknown option {s}")),
                }
            }
            if let Err(e) = link::run(&cfg) {
                die(&e);
            }
        }
        _ => println!("{HELP}"),
    }
}
