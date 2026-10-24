//! Exact hypergeometric tail and Benjamini–Hochberg.

fn ln_fact(n: u64) -> f64 {
    if n < 256 {
        (2..=n).map(|i| (i as f64).ln()).sum()
    } else {
        let x = n as f64;
        x * x.ln() - x + 0.5 * (2.0 * std::f64::consts::PI * x).ln() + 1.0 / (12.0 * x) - 1.0 / (360.0 * x * x * x)
    }
}

fn ln_choose(n: u64, k: u64) -> f64 {
    if k > n {
        f64::NEG_INFINITY
    } else {
        ln_fact(n) - ln_fact(k) - ln_fact(n - k)
    }
}

/// P(X ≥ k) for X ~ Hypergeometric(population N, K successes, n draws):
/// the chance that two entities found in K and n of N documents share at least k
/// documents if they were placed independently.
pub fn hypergeom_tail(n_pop: u64, k_succ: u64, n_draw: u64, k: u64) -> f64 {
    let hi = k_succ.min(n_draw);
    if k == 0 {
        return 1.0;
    }
    if k > hi {
        return 0.0;
    }
    let denom = ln_choose(n_pop, n_draw);
    let mut s = 0.0;
    for x in k..=hi {
        if n_draw - x > n_pop - k_succ {
            continue;
        }
        s += (ln_choose(k_succ, x) + ln_choose(n_pop - k_succ, n_draw - x) - denom).exp();
    }
    s.min(1.0)
}

pub fn bh(p: &[f64]) -> Vec<f64> {
    let m = p.len() as f64;
    let mut idx: Vec<usize> = (0..p.len()).collect();
    idx.sort_by(|&a, &b| p[a].partial_cmp(&p[b]).unwrap());
    let mut q = vec![1.0; p.len()];
    let mut run = 1.0f64;
    for (rank, &i) in idx.iter().enumerate().rev() {
        run = run.min(p[i] * m / (rank as f64 + 1.0)).min(1.0);
        q[i] = run;
    }
    q
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn tail_small_exact() {
        // N=10, K=4, n=3: P(X>=2) = [C(4,2)C(6,1)+C(4,3)C(6,0)]/C(10,3) = (36+4)/120
        assert!((hypergeom_tail(10, 4, 3, 2) - 40.0 / 120.0).abs() < 1e-12);
        assert_eq!(hypergeom_tail(10, 4, 3, 4), 0.0);
        assert_eq!(hypergeom_tail(10, 4, 3, 0), 1.0);
    }
    #[test]
    fn tail_large_reasonable() {
        // expected overlap 300*300/1000 = 90; P(X>=90) ~ 0.5, P(X>=130) tiny
        let p = hypergeom_tail(1000, 300, 300, 90);
        assert!(p > 0.4 && p < 0.6, "{p}");
        assert!(hypergeom_tail(1000, 300, 300, 130) < 1e-6);
    }
    #[test]
    fn bh_monotone() {
        let q = bh(&[0.01, 0.04, 0.03, 0.5]);
        assert!((q[0] - 0.04).abs() < 1e-12 && (q[3] - 0.5).abs() < 1e-12);
    }
}
