//! Same-meaning duplicates through an embedding server (llama-server --embeddings,
//! or anything that speaks the OpenAI /v1/embeddings API). Std only.

use crate::http::Endpoint;
use crate::json::{self, obj, s, Json};
use std::time::Duration;

/// One unit-length vector per text, in order.
pub fn embed_texts(url: &str, texts: &[String]) -> Result<Vec<Vec<f32>>, String> {
    let ep = Endpoint::parse(url)?;
    // accept "http://host:port" or a full ".../v1/embeddings" URL
    let path = if url.trim_end_matches('/').ends_with("/embeddings") { "" } else { "/v1/embeddings" };
    let mut out: Vec<Vec<f32>> = Vec::with_capacity(texts.len());
    for (bi, batch) in texts.chunks(32).enumerate() {
        if bi % 20 == 0 {
            eprintln!("embedding {}/{} names…", bi * 32, texts.len());
        }
        let body = obj(vec![("input", Json::Arr(batch.iter().map(|t| s(t)).collect()))]);
        let resp = ep
            .post_json(path, &body.to_string(), Duration::from_secs(300))
            .map_err(|e| format!("{e}\n(start the server with: llama-server -m bge-m3-Q8_0.gguf --embeddings --port 8081)"))?;
        let v = json::parse(&resp).map_err(|e| format!("bad JSON from embedding server: {e}"))?;
        let data = match v.get("data") {
            Some(Json::Arr(d)) => d.clone(),
            _ => return Err(format!("unexpected embedding reply: {}", resp.chars().take(200).collect::<String>())),
        };
        if data.len() != batch.len() {
            return Err(format!("asked for {} embeddings, got {}", batch.len(), data.len()));
        }
        let mut got: Vec<Option<Vec<f32>>> = vec![None; batch.len()];
        for (k, item) in data.iter().enumerate() {
            let idx = item.get("index").and_then(|x| x.as_f64()).map(|x| x as usize).unwrap_or(k);
            let arr = match item.get("embedding") {
                Some(Json::Arr(a)) => a,
                _ => return Err("embedding missing in reply".into()),
            };
            if matches!(arr.first(), Some(Json::Arr(_))) {
                return Err("the server returned one vector per token; restart it with --pooling cls".into());
            }
            let mut e: Vec<f32> = arr.iter().map(|x| x.as_f64().unwrap_or(0.0) as f32).collect();
            let norm = e.iter().map(|x| x * x).sum::<f32>().sqrt().max(1e-12);
            e.iter_mut().for_each(|x| *x /= norm);
            if idx < got.len() {
                got[idx] = Some(e);
            }
        }
        for g in got {
            out.push(g.ok_or("embedding reply skipped an index")?);
        }
    }
    Ok(out)
}

/// All pairs with cosine ≥ `min` for which `allowed(i, j)` holds, best first.
pub fn similar_pairs(vecs: &[Vec<f32>], min: f32, allowed: impl Fn(usize, usize) -> bool) -> Vec<(usize, usize, f32)> {
    let mut out = Vec::new();
    for i in 0..vecs.len() {
        for j in (i + 1)..vecs.len() {
            let c: f32 = vecs[i].iter().zip(&vecs[j]).map(|(a, b)| a * b).sum();
            if c >= min && allowed(i, j) {
                out.push((i, j, c));
            }
        }
    }
    out.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap());
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn pairs_above_threshold() {
        let v = vec![vec![1.0, 0.0], vec![0.99, 0.141], vec![0.0, 1.0]];
        let p = similar_pairs(&v, 0.95, |_, _| true);
        assert_eq!(p.len(), 1);
        assert_eq!((p[0].0, p[0].1), (0, 1));
        assert!(similar_pairs(&v, 0.95, |_, _| false).is_empty());
    }
}
