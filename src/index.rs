//! The index of a whole book (`index`): its words by number, its passages as
//! numbers, and for each word the passages that hold it, as a search engine
//! keeps them. The passages' text is not held here; see `volumina`.
//!
//! What it holds, per book:
//!
//! - the book's words of matter, each once, sorted (a word's number is its
//!   place in that list);
//! - each passage's words in the order they stand, as numbers;
//! - each passage's heading lines, as numbers;
//! - for each word, the passages that hold it (built when the index is made or
//!   read back, never stored).
//!
//! It can be written to disk and read back. What is read back is checked
//! throughout: a number out of range, an order that runs backwards or a file
//! cut short gives none, and the book is prepared anew.

use std::convert::TryInto;

/// Parts heading lines within a passage's run of headings.
const LIMES: u32 = u32::MAX;

/// A book's words and passages by number.
#[derive(Clone, Debug, Default)]
pub struct Index {
    /// The book's words, each once, sorted.
    verba: Vec<String>,
    /// Every passage's words in order, one run after another, and where each
    /// passage's run starts (one more start than passages, for the end).
    ordo: Vec<u32>,
    ordo_at: Vec<u32>,
    /// Every passage's heading lines, each line's words then `LIMES`, and
    /// where each passage's run starts.
    capita: Vec<u32>,
    capita_at: Vec<u32>,
    /// For each word, the passages that hold it, each once, in order.
    sedes: Vec<Vec<u32>>,
}

impl Index {
    /// Makes the index from each passage's words in order and its heading
    /// lines' words.
    pub fn build(ordines: &[Vec<String>], capita: &[Vec<Vec<String>>]) -> Index {
        let mut verba: Vec<String> = ordines
            .iter()
            .flatten()
            .chain(capita.iter().flatten().flatten())
            .cloned()
            .collect();
        verba.sort();
        verba.dedup();
        let id = |w: &String| verba.binary_search(w).map_or(0, |k| k as u32);
        let mut ix = Index {
            ordo_at: vec![0],
            capita_at: vec![0],
            ..Index::default()
        };
        for seq in ordines {
            ix.ordo.extend(seq.iter().map(id));
            ix.ordo_at.push(ix.ordo.len() as u32);
        }
        for heads in capita {
            for h in heads {
                ix.capita.extend(h.iter().map(id));
                ix.capita.push(LIMES);
            }
            ix.capita_at.push(ix.capita.len() as u32);
        }
        ix.verba = verba;
        ix.sedes = sedes(&ix);
        ix
    }

    /// How many passages it holds.
    pub fn len(&self) -> usize {
        self.ordo_at.len().saturating_sub(1)
    }

    /// Whether it holds no passage.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// A word's number, if the book holds it.
    pub fn id(&self, w: &str) -> Option<u32> {
        self.verba
            .binary_search_by(|v| v.as_str().cmp(w))
            .ok()
            .map(|k| k as u32)
    }

    /// A passage's words in order, as numbers.
    pub fn seq(&self, e: usize) -> &[u32] {
        &self.ordo[self.ordo_at[e] as usize..self.ordo_at[e + 1] as usize]
    }

    /// A passage's heading lines, each as numbers.
    pub fn heads(&self, e: usize) -> impl Iterator<Item = &[u32]> {
        let run = &self.capita[self.capita_at[e] as usize..self.capita_at[e + 1] as usize];
        run.split(|&w| w == LIMES).filter(|h| !h.is_empty())
    }

    /// The passages that hold a word, each once, in order.
    pub fn sedes(&self, id: u32) -> &[u32] {
        self.sedes.get(id as usize).map_or(&[], Vec::as_slice)
    }

    /// The index as bytes, for keeping on disk (the passages that hold each
    /// word are not kept: they are made again when it is read).
    pub fn bytes(&self, out: &mut Vec<u8>) {
        put(out, self.verba.len() as u32);
        for w in &self.verba {
            put(out, w.len() as u32);
            out.extend_from_slice(w.as_bytes());
        }
        for run in [&self.ordo_at, &self.ordo, &self.capita_at, &self.capita] {
            put(out, run.len() as u32);
            for &n in run.iter() {
                put(out, n);
            }
        }
    }

    /// Reads an index back from `bytes` at `at`, moving `at` past it, if it is
    /// whole and every number in it stands in range.
    pub fn read(bytes: &[u8], at: &mut usize) -> Option<Index> {
        let n = take(bytes, at)? as usize;
        let mut verba = Vec::with_capacity(n.min(bytes.len()));
        for _ in 0..n {
            let len = take(bytes, at)? as usize;
            let w = std::str::from_utf8(bytes.get(*at..at.checked_add(len)?)?).ok()?;
            *at += len;
            verba.push(w.to_string());
        }
        if verba.windows(2).any(|p| p[0] >= p[1]) {
            return None;
        }
        let mut runs = Vec::new();
        for _ in 0..4 {
            let len = take(bytes, at)? as usize;
            if len > bytes.len() / 4 {
                return None;
            }
            let mut run = Vec::with_capacity(len);
            for _ in 0..len {
                run.push(take(bytes, at)?);
            }
            runs.push(run);
        }
        let capita = runs.pop()?;
        let capita_at = runs.pop()?;
        let ordo = runs.pop()?;
        let ordo_at = runs.pop()?;
        let words = verba.len() as u32;
        let starts = |at: &[u32], run: &[u32]| {
            at.first() == Some(&0)
                && at.last().copied() == Some(run.len() as u32)
                && at.windows(2).all(|p| p[0] <= p[1])
        };
        let whole = starts(&ordo_at, &ordo)
            && starts(&capita_at, &capita)
            && ordo_at.len() == capita_at.len()
            && ordo.iter().all(|&w| w < words)
            && capita.iter().all(|&w| w < words || w == LIMES);
        if !whole {
            return None;
        }
        let mut ix = Index {
            verba,
            ordo,
            ordo_at,
            capita,
            capita_at,
            sedes: Vec::new(),
        };
        ix.sedes = sedes(&ix);
        Some(ix)
    }
}

/// For each word, the passages that hold it.
fn sedes(ix: &Index) -> Vec<Vec<u32>> {
    let mut sedes: Vec<Vec<u32>> = vec![Vec::new(); ix.verba.len()];
    for e in 0..ix.len() {
        for &w in ix.seq(e) {
            let s = &mut sedes[w as usize];
            if s.last() != Some(&(e as u32)) {
                s.push(e as u32);
            }
        }
    }
    sedes
}

/// Writes a number of four bytes, low byte first.
pub fn put(out: &mut Vec<u8>, n: u32) {
    out.extend_from_slice(&n.to_le_bytes());
}

/// Writes a number of eight bytes, low byte first.
pub fn put64(out: &mut Vec<u8>, n: u64) {
    out.extend_from_slice(&n.to_le_bytes());
}

/// Reads a number of four bytes at `at`, moving `at` past it.
pub fn take(bytes: &[u8], at: &mut usize) -> Option<u32> {
    let b = bytes.get(*at..at.checked_add(4)?)?;
    *at += 4;
    Some(u32::from_le_bytes(b.try_into().ok()?))
}

/// Reads a number of eight bytes at `at`, moving `at` past it.
pub fn take64(bytes: &[u8], at: &mut usize) -> Option<u64> {
    let b = bytes.get(*at..at.checked_add(8)?)?;
    *at += 8;
    Some(u64::from_le_bytes(b.try_into().ok()?))
}

/// A short fingerprint of some bytes (FNV-1a, 64 bits): enough to tell that
/// what an index was made from has changed, not a seal against forgery.
pub fn signum(parts: &[&[u8]]) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for part in parts {
        for &b in part.iter().chain(&[0xff]) {
            h ^= u64::from(b);
            h = h.wrapping_mul(0x0100_0000_01b3);
        }
    }
    h
}
