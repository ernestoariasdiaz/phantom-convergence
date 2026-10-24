//! Split a book into passages of roughly paragraph size, snapping to sentence ends,
//! and filter out passages that are code, tables, indexes or OCR noise.

pub struct Passage {
    pub index: usize,
    pub start_word: usize,
    pub text: String,
}

/// Group sentences until a passage reaches `target` words (hard cap 2*target).
pub fn passages(text: &str, target: usize) -> Vec<Passage> {
    let words: Vec<&str> = text.split_whitespace().collect();
    let mut out = Vec::new();
    let mut cur: Vec<&str> = Vec::new();
    let mut start = 0usize;
    for (i, w) in words.iter().enumerate() {
        if cur.is_empty() {
            start = i;
        }
        cur.push(w);
        let ends_sentence = w.ends_with('.') || w.ends_with('?') || w.ends_with('!') || w.ends_with('»') || w.ends_with('"');
        if (cur.len() >= target && ends_sentence) || cur.len() >= target * 2 {
            out.push(Passage { index: out.len(), start_word: start, text: cur.join(" ") });
            cur.clear();
        }
    }
    if cur.len() >= target / 2 {
        out.push(Passage { index: out.len(), start_word: start, text: cur.join(" ") });
    }
    out
}

/// Why a passage is not worth sending to the model (None = keep).
pub fn reject_reason(text: &str) -> Option<&'static str> {
    let total = text.chars().count().max(1) as f64;
    let alpha = text.chars().filter(|c| c.is_alphabetic()).count() as f64;
    let digits = text.chars().filter(|c| c.is_ascii_digit()).count() as f64;
    let code = text.chars().filter(|c| "{};=<>[]()#*/\\|_".contains(*c)).count() as f64;
    let words: Vec<&str> = text.split_whitespace().collect();
    if words.len() < 40 {
        return Some("too short");
    }
    if alpha / total < 0.70 {
        return Some("not prose (few letters)");
    }
    if digits / total > 0.08 {
        return Some("numbers/table/index");
    }
    if code / total > 0.025 {
        return Some("code or markup");
    }
    let years = words
        .iter()
        .filter(|w| {
            let d: String = w.chars().filter(|c| c.is_ascii_digit()).collect();
            d.len() == 4 && (d.starts_with("19") || d.starts_with("20")) && w.chars().filter(|c| c.is_alphabetic()).count() <= 1
        })
        .count();
    if years >= 4 {
        return Some("bibliography/references");
    }
    let single = words.iter().filter(|w| w.chars().filter(|c| c.is_alphabetic()).count() == 1).count() as f64;
    if single / words.len() as f64 > 0.25 {
        return Some("OCR noise");
    }
    None
}

/// Evenly spaced sample of `k` items out of `n` (all if k >= n or k == 0).
pub fn even_sample(n: usize, k: usize) -> Vec<usize> {
    if k == 0 || k >= n {
        return (0..n).collect();
    }
    (0..k).map(|i| (i * n + n / 2) / k).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn chunks_on_sentence_end() {
        let t = "One two three. Four five six. Seven eight nine ten. Eleven.";
        let p = passages(t, 4);
        assert_eq!(p[0].text, "One two three. Four five six.");
    }
    #[test]
    fn rejects_code() {
        let code = "fn main() { let x = vec![1, 2, 3]; for i in x { println!(\"{}\", i); } } ".repeat(5);
        assert!(reject_reason(&code).is_some());
        let prose = "Cooperation can emerge among egoists when the interaction is repeated and the future casts a long shadow over the present, because each player knows that a betrayal today will be answered tomorrow, and that the gains from mutual trust accumulate over many rounds of the game.";
        assert_eq!(reject_reason(prose), None);
    }
    #[test]
    fn rejects_bibliography() {
        let b = "Kim, S. 2016. Stream Windowing Performance Analysis. concord.io, July 6, 2016. Smith, J. (1978). The Genetics of Altruism. New York: Academic Press. Taylor, M. 1976. Anarchy and Cooperation. London: Wiley. Hamilton, W. D. 1967. Extraordinary sex ratios. Science 156: 477 and further notes on the subject of these works";
        assert!(reject_reason(b).is_some(), "{:?}", reject_reason(b));
        let refs = "Axelrod, Robert. 1984. The Evolution of Cooperation. New York: Basic Books. Dawkins, Richard. 1976. The Selfish Gene. Oxford: Oxford University Press. Hamilton, William. 1964. The genetical evolution of social behaviour. Journal of Theoretical Biology. Trivers, Robert. 1971. The evolution of reciprocal altruism. Quarterly Review of Biology, which remains a classic reference for anyone studying these questions in depth today.";
        assert_eq!(reject_reason(refs), Some("bibliography/references"));
    }

    #[test]
    fn sample_spreads() {
        assert_eq!(even_sample(10, 2), vec![2, 7]);
        assert_eq!(even_sample(3, 5), vec![0, 1, 2]);
    }
}
