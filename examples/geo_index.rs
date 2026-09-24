//! A spatial index the way a column store keeps one: points sorted by
//! their key on a curve, cut into row groups that remember their least
//! and greatest key, and a rectangle query that skips the groups it
//! cannot touch and seeks inside the rest.
//!
//! `cargo run --release --example geo_index`

// Degrees to cells is a rounding by design, and so are the random
// coordinates; the casts say what they mean.
#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss,
    clippy::suboptimal_flops
)]

use hakmem::prelude::*;

/// Longitude and latitude on a grid of 2^32 × 2^32 cells.
fn cell(lon: f64, lat: f64) -> (u64, u64) {
    let scale = |v: f64, lo: f64, hi: f64| ((v - lo) / (hi - lo) * f64::from(u32::MAX)) as u64;
    (scale(lon, -180.0, 180.0), scale(lat, -90.0, 90.0))
}

struct Index<C> {
    /// Keys, sorted, with the point each came from.
    rows: Vec<(C, (f64, f64))>,
    /// Least and greatest key of each group of `GROUP` rows.
    groups: Vec<(u64, u64)>,
}

const GROUP: usize = 1024;

impl<C: Curve2<Word = u64>> Index<C> {
    fn build(points: &[(f64, f64)]) -> Self {
        let mut rows: Vec<(C, (f64, f64))> = points
            .iter()
            .map(|&(lon, lat)| {
                let (x, y) = cell(lon, lat);
                (C::encode(x, y), (lon, lat))
            })
            .collect();
        rows.sort_unstable_by_key(|r| r.0);
        let groups = rows
            .chunks(GROUP)
            .map(|g| (g[0].0.key(), g[g.len() - 1].0.key()))
            .collect();
        Self { rows, groups }
    }

    /// Points in the box, and how many groups the query had to read.
    fn query(
        &self,
        (lon0, lat0): (f64, f64),
        (lon1, lat1): (f64, f64),
    ) -> (Vec<(f64, f64)>, usize) {
        let ((x0, y0), (x1, y1)) = (cell(lon0, lat0), cell(lon1, lat1));
        let mut ranges = [(0, 0); 32];
        let ranges = C::cover(x0..=x1, y0..=y1, &mut ranges);
        let (mut found, mut read) = (Vec::new(), 0);
        for (g, &(lo, hi)) in self.groups.iter().enumerate() {
            if !C::intersects(lo..=hi, x0..=x1, y0..=y1) {
                continue;
            }
            read += 1;
            let rows = &self.rows[g * GROUP..((g + 1) * GROUP).min(self.rows.len())];
            // A seek per range; the ranges hold more than the box when
            // the budget is short, so each point still gets its test.
            for &(a, b) in ranges {
                let from = rows.partition_point(|r| r.0.key() < a);
                let to = rows.partition_point(|r| r.0.key() <= b);
                found.extend(rows[from..to].iter().map(|r| r.1).filter(|&(lon, lat)| {
                    (lon0..=lon1).contains(&lon) && (lat0..=lat1).contains(&lat)
                }));
            }
        }
        (found, read)
    }
}

fn main() {
    // A million points around a few hundred towns.
    let mut state = 0x2545_F491_4F6C_DD1Du64;
    let mut next = move || {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        (state >> 11) as f64 / (1u64 << 53) as f64
    };
    let towns: Vec<(f64, f64)> = (0..300)
        .map(|_| (next() * 360.0 - 180.0, next() * 140.0 - 70.0))
        .collect();
    let points: Vec<(f64, f64)> = (0..1_000_000)
        .map(|i| {
            let (lon, lat) = towns[i % towns.len()];
            (lon + (next() - 0.5) * 2.0, lat + (next() - 0.5) * 2.0)
        })
        .collect();

    // A box around the first town and a little of the country around it.
    let (lon, lat) = towns[0];
    let (lo, hi) = ((lon - 1.5, lat - 1.0), (lon + 1.0, lat + 1.5));
    let truth = points
        .iter()
        .filter(|&&(lon, lat)| (lo.0..=hi.0).contains(&lon) && (lo.1..=hi.1).contains(&lat))
        .count();

    for (name, (found, read), groups) in [
        (
            "Z-order",
            Index::<Morton2<u64>>::build(&points).query(lo, hi),
            points.len().div_ceil(GROUP),
        ),
        (
            "Hilbert",
            Index::<Hilbert2<u64>>::build(&points).query(lo, hi),
            points.len().div_ceil(GROUP),
        ),
    ] {
        assert_eq!(found.len(), truth);
        println!(
            "{name}: {} points, {read} of {groups} groups read",
            found.len()
        );
    }
}
