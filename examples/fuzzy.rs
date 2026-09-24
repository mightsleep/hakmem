//! Approximate search the way a spell checker or a log grep wants it:
//! the words of a list closest to a misspelling, and where a phrase
//! occurs in a text with a few typos. Bit-parallel Myers, one word of
//! state per 64 pattern bytes.
//!
//! `cargo run --release --example fuzzy`

use hakmem::myers::{distance, search};

fn main() {
    let words = [
        "necessary",
        "separate",
        "definitely",
        "occurrence",
        "accommodate",
        "embarrass",
        "rhythm",
        "millennium",
        "conscience",
        "liaison",
    ];
    let typo = "neccesary";

    // Closest words first; `distance` picks a word wide enough for the
    // pattern, and says `None` past 512 bytes.
    let mut ranked: Vec<(u32, &str)> = words
        .iter()
        .filter_map(|w| distance(typo.as_bytes(), w.as_bytes()).map(|d| (d, *w)))
        .collect();
    ranked.sort_unstable();
    println!("{typo}: {:?}", &ranked[..3]);
    assert_eq!(ranked[0], (2, "necessary"));

    // A phrase in a text, up to two edits away. The pattern fits a u64,
    // so the word is `u64`; each occurrence comes with where it starts.
    let text = b"the quick brown fox jumsp over the lazy dgo, and the quikc brown fox too";
    let phrase = b"quick brown fox";
    let found: Vec<_> = search::<u64>(phrase, text, 2)
        .expect("pattern fits a u64")
        .occurrences()
        .collect();
    for o in &found {
        println!(
            "  {:?} at {:?}, {} edits",
            String::from_utf8_lossy(&text[o.range()]),
            o.range(),
            o.distance()
        );
    }
    assert_eq!(found.len(), 2);
}
