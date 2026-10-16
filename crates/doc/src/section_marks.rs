//! The section marks a drawing has (A–A′, B–B′ …) and the letter the next one takes (Trowel
//! design 3.0 §5 ④, §6): the Section Line tool and `trowel.insertSectionMarker` give each new
//! mark the first letter no mark in the drawing uses yet.

use crate::{Document, NodeKind};

/// How the groups of section marks are named: `단면 기호 A–A′` (`trowel.insertSectionMarker`) and
/// `단면선 A–A′` (`trowel.sectionLine`).
const PREFIXES: [&str; 2] = ["단면 기호 ", "단면선 "];

/// The most marks a drawing names (A–Z, then A1–Z1 … Z99): past that the next one is `A` again.
const ROUNDS: u32 = 100;

/// The letters the drawing's section marks use, in document order, each once.
pub fn letters(doc: &Document) -> Vec<String> {
    let mut out: Vec<String> = vec![];
    doc.walk(|n| {
        if !matches!(n.kind, NodeKind::Group { .. }) {
            return;
        }
        let Some(name) = n.name.as_deref() else { return };
        for prefix in PREFIXES {
            if let Some((letter, _)) = name.strip_prefix(prefix).and_then(|rest| rest.split_once('–')) {
                let letter = letter.trim();
                if !letter.is_empty() && !out.iter().any(|l| l == letter) {
                    out.push(letter.to_string());
                }
            }
        }
    });
    out
}

/// The first of A–Z that no section mark in `doc` uses; after Z, A1–Z1 and so on.
pub fn next_letter(doc: &Document) -> String {
    let used = letters(doc);
    (0..ROUNDS)
        .flat_map(|round| ('A'..='Z').map(move |c| if round == 0 { c.to_string() } else { format!("{c}{round}") }))
        .find(|l| !used.contains(l))
        .unwrap_or_else(|| "A".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Node;

    fn mark(doc: &mut Document, name: &str) {
        let layer = doc.layers.first().map(|l| l.id);
        let id = doc.alloc_id();
        let mut g = Node::group(id, vec![]);
        g.name = Some(name.to_string());
        doc.insert(layer, usize::MAX, g).unwrap();
    }

    #[test]
    fn the_next_letter_skips_the_ones_in_use() {
        let mut doc = Document::new(400.0, 300.0);
        assert_eq!(next_letter(&doc), "A");
        mark(&mut doc, "단면 기호 A–A′");
        assert_eq!(next_letter(&doc), "B");
        mark(&mut doc, "단면선 C–C′");
        assert_eq!((letters(&doc), next_letter(&doc)), (vec!["A".to_string(), "C".to_string()], "B".to_string()));
        // Other names, a datum tick and the section layer, don't count.
        mark(&mut doc, "단면 해발 눈금 (93.0m)");
        mark(&mut doc, "단면선 (잘린 땅)");
        assert_eq!(letters(&doc).len(), 2);
        for c in ['B', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z'] {
            mark(&mut doc, &format!("단면 기호 {c}–{c}′"));
        }
        assert_eq!(next_letter(&doc), "A1");
    }
}
