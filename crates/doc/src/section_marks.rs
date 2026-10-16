//! The section marks a drawing has (A–A′, B–B′ …) and the letter the next one takes (Trowel
//! design 3.0 §5 ④, §6): the Section Line tool and `trowel.insertSectionMarker` give each new
//! mark the first letter no mark in the drawing uses yet.

use crate::{Document, Node, NodeKind};

/// How the groups of section marks are named: `단면 기호 A–A′` (`trowel.insertSectionMarker`) and
/// `단면선 A–A′` (`trowel.sectionLine`).
const PREFIXES: [&str; 2] = ["단면 기호 ", "단면선 "];

/// How the other marks the archaeology tools and the drawing set's elements put in are named: datum
/// ticks, spot heights, layer boundaries, numbers and descriptions, scale bars, north arrows,
/// slope hachures.
const MARK_PREFIXES: [&str; 9] = ["단면 해발 눈금", "표고 ", "토층선 ", "층위 번호", "토층 번호", "토층설명", "축척 막대", "방위표", "비탈면 빗금"];

/// Whether `n` is a mark the tools put in (A–A′ marks, level lines, datum ticks …), not a line
/// drawn by hand: the Level Lines and Slope Hachure tools don't take one for the line clicked.
pub fn is_mark(n: &Node) -> bool {
    if n.is_layer() {
        return false;
    }
    if n.elevation.is_some() {
        return true;
    }
    let Some(name) = n.name.as_deref() else { return false };
    PREFIXES.iter().any(|p| name.strip_prefix(p).is_some_and(|rest| rest.contains('–'))) || MARK_PREFIXES.iter().any(|p| name.starts_with(p))
}

/// Whether `n` is a soil layer description block (`stratigraphy.boundary`'s 「토층설명 7p」, an
/// inline table's 「토층설명 7p (…)」): a double-click on one of its lines types in it at once
/// rather than isolating the group (design 3.0 §8 step 8).
pub fn is_note_block(n: &Node) -> bool {
    matches!(n.kind, NodeKind::Group { .. }) && n.name.as_deref().is_some_and(|m| m.starts_with(NOTE_BLOCK))
}

/// The name the description blocks start with.
pub const NOTE_BLOCK: &str = "토층설명 7p";

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

    #[test]
    fn marks_are_told_from_drawn_lines() {
        let named = |name: Option<&str>| {
            let mut n = Node::group(crate::NodeId(1), vec![]);
            n.name = name.map(str::to_string);
            n
        };
        for name in ["단면 기호 A–A′", "단면선 B–B′", "단면 해발 눈금 (32.5m)", "표고 32.5m", "토층선 Ⅰ", "층위 번호 Ⅱ", "토층설명 7p", "비탈면 빗금"]
        {
            assert!(is_mark(&named(Some(name))), "{name}");
        }
        // A drawn line, a group of them, and the section layer's name on a group are not.
        for name in [None, Some("지표선"), Some("단면선 (잘린 땅)"), Some("단면선")] {
            assert!(!is_mark(&named(name)), "{name:?}");
        }
    }
}
