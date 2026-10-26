use unicode_segmentation::UnicodeSegmentation;

fn eq_fold(a: &str, b: &str) -> bool {
    if a == b {
        return true;
    }
    a.chars()
        .flat_map(char::to_lowercase)
        .map(|c| if c == 'ς' { 'σ' } else { c })
        .eq(b
            .chars()
            .flat_map(char::to_lowercase)
            .map(|c| if c == 'ς' { 'σ' } else { c }))
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum FuzzyBoundary {
    #[default]
    Word,
    Identifier,
}
impl FuzzyBoundary {
    fn contains(self, grapheme: &str) -> bool {
        matches!(grapheme, "_" | ".") || (self == Self::Word && matches!(grapheme, " " | "-"))
    }
}

pub fn fuzzy(label: &str, word: &str) -> Option<(u32, Vec<usize>)> {
    fuzzy_with_boundary(label, word, FuzzyBoundary::default())
}

pub fn fuzzy_with_boundary(
    label: &str,
    word: &str,
    boundary: FuzzyBoundary,
) -> Option<(u32, Vec<usize>)> {
    if word.is_empty() {
        return Some((0, Vec::new()));
    }
    let lg: Vec<&str> = label.graphemes(true).collect();
    let wg: Vec<&str> = word.graphemes(true).collect();
    if wg.len() > lg.len() {
        return subsequence(&lg, &wg);
    }

    let last_start = lg.len().saturating_sub(wg.len());
    for start in 0..=last_start {
        let hit = wg.iter().enumerate().all(|(k, w)| {
            lg.get(start.saturating_add(k))
                .is_some_and(|l| eq_fold(l, w))
        });
        if hit {
            let idx: Vec<usize> = (start..start.saturating_add(wg.len())).collect();
            if start == 0 {
                return Some((0, idx));
            }
            let at_boundary = lg
                .get(start.saturating_sub(1))
                .is_some_and(|grapheme| boundary.contains(grapheme));
            return Some((if at_boundary { 10 } else { 30 }, idx));
        }
    }
    subsequence(&lg, &wg)
}

fn subsequence(lg: &[&str], wg: &[&str]) -> Option<(u32, Vec<usize>)> {
    let mut matched = Vec::with_capacity(wg.len());
    let mut li = 0usize;
    for w in wg {
        loop {
            let l = lg.get(li)?;
            if eq_fold(l, w) {
                matched.push(li);
                li = li.saturating_add(1);
                break;
            }
            li = li.saturating_add(1);
        }
    }
    let last = matched.last().copied().unwrap_or(0) as u32;
    Some((60u32.saturating_add(last), matched))
}
