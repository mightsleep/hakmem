//! Myers bit-parallel edit distance against the textbook DP.

use hakmem::myers::distance_in;
use proptest::prelude::*;

fn levenshtein_dp(a: &[u8], b: &[u8]) -> u32 {
    let mut prev: Vec<u32> = (0..=u32::try_from(b.len()).unwrap()).collect();
    for (i, &ca) in a.iter().enumerate() {
        let mut cur = vec![u32::try_from(i).unwrap() + 1; b.len() + 1];
        for (j, &cb) in b.iter().enumerate() {
            let sub = prev[j] + u32::from(ca != cb);
            cur[j + 1] = sub.min(prev[j + 1] + 1).min(cur[j] + 1);
        }
        prev = cur;
    }
    prev[b.len()]
}

proptest! {
    #[test]
    fn u64_matches_dp(a in prop::collection::vec(0u8..4, 0..=64), b in prop::collection::vec(0u8..4, 0..=80)) {
        prop_assert_eq!(distance_in::<u64>(&a, &b), Some(levenshtein_dp(&a, &b)));
    }
    #[test]
    fn u128_matches_dp(a in prop::collection::vec(any::<u8>(), 0..=128), b in prop::collection::vec(any::<u8>(), 0..=100)) {
        prop_assert_eq!(distance_in::<u128>(&a, &b), Some(levenshtein_dp(&a, &b)));
    }
    #[test]
    fn u8_full_width_matches_dp(a in prop::collection::vec(0u8..3, 8..=8), b in prop::collection::vec(0u8..3, 0..=20)) {
        prop_assert_eq!(distance_in::<u8>(&a, &b), Some(levenshtein_dp(&a, &b)));
    }
}

#[test]
fn rejects_pattern_wider_than_word() {
    assert_eq!(distance_in::<u8>(&[0; 9], b"x"), None);
    assert_eq!(distance_in::<u8>(&[0; 8], b"x"), Some(8));
}

/// Semi-global DP: `D[0][j] = 0`, distances of the pattern against the
/// best-starting substring ending at each `j`.
fn semi_global_dp(p: &[u8], t: &[u8]) -> Vec<u32> {
    let mut prev: Vec<u32> = vec![0; t.len() + 1];
    for (i, &cp) in p.iter().enumerate() {
        let mut cur = vec![u32::try_from(i).unwrap() + 1; t.len() + 1];
        for (j, &ct) in t.iter().enumerate() {
            let sub = prev[j] + u32::from(cp != ct);
            cur[j + 1] = sub.min(prev[j + 1] + 1).min(cur[j] + 1);
        }
        prev = cur;
    }
    prev
}

proptest! {
    #[test]
    fn search_matches_semi_global_dp(p in prop::collection::vec(0u8..4, 1..=64), t in prop::collection::vec(0u8..4, 0..=80), k in 0u32..4) {
        let dp = semi_global_dp(&p, &t);
        let want: Vec<(usize, u32)> = dp.iter().enumerate().skip(1).filter(|&(_, &d)| d <= k).map(|(j, &d)| (j, d)).collect();
        let got: Vec<(usize, u32)> = hakmem::myers::search::<u64>(&p, &t, k).unwrap().collect();
        prop_assert_eq!(got, want);
        let best = dp.iter().copied().min().unwrap();
        prop_assert_eq!(hakmem::myers::substring_distance::<u64>(&p, &t), Some(best));
    }
}

proptest! {
    #[test]
    fn wide4_matches_dp(a in prop::collection::vec(any::<u8>(), 0..=256), b in prop::collection::vec(any::<u8>(), 0..=300)) {
        prop_assert_eq!(distance_in::<hakmem::Wide<4>>(&a, &b), Some(levenshtein_dp(&a, &b)));
    }
    #[test]
    fn wide4_search_matches_dp(p in prop::collection::vec(0u8..4, 1..=200), t in prop::collection::vec(0u8..4, 0..=120), k in 0u32..4) {
        let dp = semi_global_dp(&p, &t);
        let want: Vec<(usize, u32)> = dp.iter().enumerate().skip(1).filter(|&(_, &d)| d <= k).map(|(j, &d)| (j, d)).collect();
        let got: Vec<(usize, u32)> = hakmem::myers::search::<hakmem::Wide<4>>(&p, &t, k).unwrap().collect();
        prop_assert_eq!(got, want);
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(2048))]
    /// Occurrences against the DP: each is `distance` edits from its
    /// text, that is the least distance of any substring ending there,
    /// and there is one per run of adjacent ends, at the run's first
    /// least.
    #[test]
    fn occurrences_match_dp(
        pattern in prop::collection::vec(0u8..3, 1..10),
        text in prop::collection::vec(0u8..3, 0..48),
        k in 0u32..4,
    ) {
        let ends: Vec<(usize, u32)> = hakmem::myers::search::<u64>(&pattern, &text, k).unwrap().collect();
        let mut want: Vec<(usize, (usize, u32))> = Vec::new();
        for &(end, d) in &ends {
            match want.last_mut() {
                Some((last_end, best)) if *last_end + 1 == end => {
                    *last_end = end;
                    if d < best.1 {
                        *best = (end, d);
                    }
                }
                _ => want.push((end, (end, d))),
            }
        }
        let found: Vec<_> = hakmem::myers::search::<u64>(&pattern, &text, k).unwrap().occurrences().collect();
        prop_assert_eq!(found.len(), want.len());
        for (o, &(_, (end, d))) in found.iter().zip(&want) {
            prop_assert_eq!((o.end(), o.distance()), (end, d));
            prop_assert_eq!(levenshtein_dp(&pattern, &text[o.range()]), d);
            let least = (0..=end).map(|s| levenshtein_dp(&pattern, &text[s..end])).min().unwrap();
            prop_assert_eq!(least, d);
            // Of the starts that reach `d`, the length nearest the pattern's.
            let nearest = (0..=end)
                .filter(|&s| levenshtein_dp(&pattern, &text[s..end]) == d)
                .map(|s| (end - s).abs_diff(pattern.len()))
                .min()
                .unwrap();
            prop_assert_eq!((end - o.start()).abs_diff(pattern.len()), nearest);
        }
    }
}
