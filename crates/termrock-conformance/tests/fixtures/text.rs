//! The text corpus (`COMPONENT_ARCHITECTURE.md` §21 item 29): ≥ 200 strings
//! covering ASCII, CJK wide, combining marks, ZWJ emoji, RTL, control
//! characters and widths `0..=120`, shared by the width, `RowUi` and
//! truncation tests.

/// Hand-written seeds covering every category.
pub(crate) const SEEDS: &[&str] = &[
    "",
    "a",
    "hello",
    "hello world",
    "ｶﾞ",
    "あ",
    "a\u{FF9E}",
    "日本語",
    "漢字テキスト",
    "\u{1F468}\u{200D}\u{1F469}\u{200D}\u{1F467}\u{200D}\u{1F466}",
    "e\u{301}",
    "\u{00E9}",
    "abc\u{FF9F}x",
    "\r\n",
    "\u{7}",
    "a\u{7}b",
    "x\r\ny",
    "שלום",
    "مرحبا",
    "עברית and english",
    "tab\there",
    "🎉 party",
    "👍🏽",
    "🇯🇵",
    "naïve café",
    "Zürich",
    "İstanbul",
    "snake_case_identifier",
    "kebab-case-identifier",
    "very_long_identifier_name_that_keeps_going",
    "a b c d e f g h i j k l m n o p q r s t u v w x y z",
    "0123456789",
    "…",
    "‹N N›",
    "▎›✓•+−!▲▸▾▴∇▪→↓‹›…×●○◆◇",
    "⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏",
    "│┃─━╭╮╰╯",
    "[✓] [ ] (●) (○)",
    "mixed 日本 and ascii",
    "combining a\u{300}\u{301}\u{302}",
    "zero\u{200B}width",
    "\u{FEFF}bom",
];

/// The full corpus: the seeds plus generated strings of every width from
/// 0 to 120 in ASCII, CJK and mixed alphabets.
pub(crate) fn corpus() -> Vec<String> {
    let mut out: Vec<String> = SEEDS.iter().map(|s| (*s).to_owned()).collect();
    for w in 0..=120usize {
        out.push("a".repeat(w));
        out.push("漢".repeat(w / 2));
        let mut mixed = String::new();
        for i in 0..w {
            mixed.push_str(match i % 4 {
                0 => "a",
                1 => "字",
                2 => "e\u{301}",
                _ => "é",
            });
        }
        out.push(mixed);
    }
    out
}
