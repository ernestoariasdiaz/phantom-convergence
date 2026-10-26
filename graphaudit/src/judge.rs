//! Ask a local instruction model whether two nodes are the same entity, reading their
//! descriptions. Each pair is asked twice, in both orders; only two "SAME" answers count.
//! Std only; talks to llama-server's /v1/chat/completions.

use crate::http::Endpoint;
use crate::json::{self, n, obj, s, Json};
use std::time::Duration;

pub struct Node<'a> {
    pub name: &'a str,
    pub kind: &'a str,
    pub desc: &'a str,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Verdict {
    Same,
    Different,
    Disagree,
}

impl Verdict {
    pub fn as_str(&self) -> &'static str {
        match self {
            Verdict::Same => "same",
            Verdict::Different => "different",
            Verdict::Disagree => "uncertain",
        }
    }
}

const SYSTEM: &str = "You check a knowledge graph for duplicate nodes. Two nodes are the SAME only if they \
refer to one and the same real-world entity: the same person, organisation, place, event, object or concept, \
perhaps under another name, title or spelling. Related entities are DIFFERENT: relatives, spouses, allies, \
a predecessor and successor, a part and its whole, a general idea and a specific kind of it, two members of the \
same family or category, two events in a series. If the descriptions do not show they are the same, answer DIFFERENT.";

fn prompt(a: &Node, b: &Node) -> String {
    let show = |x: &Node| {
        format!(
            "Name: {}\nType: {}\nDescription: {}",
            x.name,
            if x.kind.is_empty() { "unknown" } else { x.kind },
            if x.desc.is_empty() { "(none)" } else { x.desc }
        )
    };
    format!(
        "Node A\n{}\n\nNode B\n{}\n\nAre A and B the same entity? Answer with one word, SAME or DIFFERENT, then ' | ' and a reason of at most 15 words.",
        show(a),
        show(b)
    )
}

fn ask(ep: &Endpoint, user: &str) -> Result<(bool, String), String> {
    let body = obj(vec![
        (
            "messages",
            Json::Arr(vec![
                obj(vec![("role", s("system")), ("content", s(SYSTEM))]),
                obj(vec![("role", s("user")), ("content", s(user))]),
            ]),
        ),
        ("temperature", n(0.0)),
        ("max_tokens", n(60.0)),
        ("stream", Json::Bool(false)),
        ("chat_template_kwargs", obj(vec![("enable_thinking", Json::Bool(false))])),
    ]);
    let resp = ep.post_json("/v1/chat/completions", &body.to_string(), Duration::from_secs(300))?;
    let v = json::parse(&resp).map_err(|e| format!("bad JSON from server: {e}"))?;
    let content = v
        .get("choices")
        .and_then(|c| c.idx(0))
        .and_then(|c| c.get("message"))
        .and_then(|m| m.get("content"))
        .and_then(|c| c.as_str())
        .unwrap_or("")
        .trim()
        .to_string();
    if content.is_empty() {
        return Err("the model returned an empty answer; if it is a thinking model, restart llama-server with --reasoning-budget 0".into());
    }
    let first = content.trim_start_matches(|c: char| !c.is_alphabetic()).to_uppercase();
    let same = first.starts_with("SAME");
    let reason = content.split_once('|').map(|(_, r)| r.trim().to_string()).unwrap_or_default();
    Ok((same, reason))
}

/// Judge one pair in both orders.
pub fn judge(ep: &Endpoint, a: &Node, b: &Node) -> Result<(Verdict, String), String> {
    let (s1, r1) = ask(ep, &prompt(a, b))?;
    let (s2, r2) = ask(ep, &prompt(b, a))?;
    let v = match (s1, s2) {
        (true, true) => Verdict::Same,
        (false, false) => Verdict::Different,
        _ => Verdict::Disagree,
    };
    Ok((v, if r1.is_empty() { r2 } else { r1 }))
}

pub fn endpoint(url: &str) -> Result<Endpoint, String> {
    Endpoint::parse(url.trim_end_matches("/v1/chat/completions"))
}
