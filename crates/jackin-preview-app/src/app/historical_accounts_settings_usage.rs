// Generated historical accounts, settings, and usage visual frames for 120x40 truecolor conformance.
use super::App;
use termrock::{Rect, Ui};

impl App {
    pub(super) fn draw_historical_accounts_add_form_120_40(&self, ui: &mut Ui<'_>, area: Rect) {
        ui.fill(
            area,
            self.historical_span_style((128, 128, 128), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(0, 0, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 0, 9, 1),
            " jackin❯ ",
            self.historical_span_style((77, 77, 77), (39, 39, 42), false),
        );
        ui.paint_str(
            Rect::new(10, 0, 2, 1),
            "  ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(12, 0, 6, 1),
            " File ",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(18, 0, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(19, 0, 4, 1),
            " Go ",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(23, 0, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(24, 0, 6, 1),
            " Help ",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(30, 0, 21, 1),
            "                     ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(51, 0, 19, 1),
            "Accounts › Overview",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(70, 0, 2, 1),
            "  ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(72, 0, 14, 1),
            "⠋ refreshing 1",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(86, 0, 2, 1),
            "  ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(88, 0, 20, 1),
            "inside the Construct",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(108, 0, 2, 1),
            "  ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(110, 0, 9, 1),
            "2 running",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(119, 0, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(Rect::new(0, 1, 120, 1), "                                                                                                                        ", self.historical_span_style((128, 128, 128), (0, 0, 0), false));
        ui.paint_str(
            Rect::new(0, 2, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 2, 2, 1),
            "╭─",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(3, 2, 10, 1),
            " Accounts ",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(13, 2, 27, 1),
            "───────── 12 · 4 ▲ · 4 ! ─╮",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(40, 2, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(41, 2, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(42, 2, 2, 1),
            "  ",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(44, 2, 8, 1),
            "Overview",
            self.historical_span_style((77, 77, 77), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(52, 2, 40, 1),
            "                                        ",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(92, 2, 25, 1),
            "12 accounts · 8 providers",
            self.historical_span_style((38, 38, 38), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(117, 2, 2, 1),
            "  ",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 2, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 3, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 3, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 3, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(3, 3, 1, 1),
            " ",
            self.historical_span_style((0, 0, 0), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(4, 3, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(5, 3, 1, 1),
            " ",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(6, 3, 33, 1),
            " Overview                        ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(39, 3, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(40, 3, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(41, 3, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(42, 3, 77, 1),
            "                                                                             ",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 3, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 4, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 4, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 4, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(3, 4, 1, 1),
            " ",
            self.historical_span_style((0, 0, 0), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(4, 4, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(5, 4, 1, 1),
            "▾",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(6, 4, 33, 1),
            " Claude                      !   ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(39, 4, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(40, 4, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(41, 4, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(42, 4, 2, 1),
            "  ",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(44, 4, 6, 1),
            "Health",
            self.historical_span_style((38, 38, 38), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(50, 4, 66, 1),
            "        degraded · 4 warnings · 1 exhausted · 2 stale · 1         ",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(116, 4, 1, 1),
            "┃",
            self.historical_span_style((38, 38, 38), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(117, 4, 2, 1),
            "  ",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 4, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 5, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 5, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 5, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(3, 5, 1, 1),
            " ",
            self.historical_span_style((0, 0, 0), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(4, 5, 3, 1),
            "   ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(7, 5, 1, 1),
            "★",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(8, 5, 17, 1),
            " Personal        ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(25, 5, 2, 1),
            "╭─",
            self.historical_span_style((77, 77, 77), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(27, 5, 13, 1),
            " New account ",
            self.historical_span_style((255, 255, 255), (24, 24, 27), true),
        );
        ui.paint_str(
            Rect::new(40, 5, 55, 1),
            "───────────────────────────────────── form · unsaved ─╮",
            self.historical_span_style((77, 77, 77), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(95, 5, 21, 1),
            "isible · 2 identities",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(116, 5, 1, 1),
            "┃",
            self.historical_span_style((38, 38, 38), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(117, 5, 2, 1),
            "  ",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 5, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 6, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 6, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 6, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(3, 6, 1, 1),
            " ",
            self.historical_span_style((0, 0, 0), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(4, 6, 3, 1),
            "   ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(7, 6, 1, 1),
            " ",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(8, 6, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(9, 6, 16, 1),
            "Archived contrac",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(25, 6, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(26, 6, 3, 1),
            "   ",
            self.historical_span_style((38, 38, 38), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(29, 6, 13, 1),
            "Display name ",
            self.historical_span_style((255, 255, 255), (24, 24, 27), true),
        );
        ui.paint_str(
            Rect::new(42, 6, 1, 1),
            "*",
            self.historical_span_style((72, 224, 84), (24, 24, 27), true),
        );
        ui.paint_str(
            Rect::new(43, 6, 49, 1),
            "                                                 ",
            self.historical_span_style((255, 255, 255), (24, 24, 27), true),
        );
        ui.paint_str(
            Rect::new(92, 6, 2, 1),
            "  ",
            self.historical_span_style((128, 128, 128), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(94, 6, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(95, 6, 21, 1),
            "                     ",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(116, 6, 1, 1),
            "┃",
            self.historical_span_style((38, 38, 38), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(117, 6, 2, 1),
            "  ",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 6, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 7, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 7, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 7, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(3, 7, 1, 1),
            " ",
            self.historical_span_style((0, 0, 0), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(4, 7, 3, 1),
            "   ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(7, 7, 1, 1),
            " ",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(8, 7, 17, 1),
            " Work            ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(25, 7, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(26, 7, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(27, 7, 1, 1),
            "▎",
            self.historical_span_style((72, 224, 84), (30, 30, 34), false),
        );
        ui.paint_str(
            Rect::new(28, 7, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (30, 30, 34), false),
        );
        ui.paint_str(
            Rect::new(29, 7, 21, 1),
            "Personal, Work, Team…",
            self.historical_span_style((128, 128, 128), (30, 30, 34), false),
        );
        ui.paint_str(
            Rect::new(50, 7, 42, 1),
            "                                          ",
            self.historical_span_style((255, 255, 255), (30, 30, 34), false),
        );
        ui.paint_str(
            Rect::new(92, 7, 2, 1),
            "  ",
            self.historical_span_style((128, 128, 128), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(94, 7, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(95, 7, 21, 1),
            " · 8 providers       ",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(116, 7, 1, 1),
            "┃",
            self.historical_span_style((38, 38, 38), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(117, 7, 2, 1),
            "  ",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 7, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 8, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 8, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 8, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(3, 8, 1, 1),
            " ",
            self.historical_span_style((0, 0, 0), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(4, 8, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(5, 8, 1, 1),
            "▾",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(6, 8, 19, 1),
            " Codex             ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(25, 8, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(26, 8, 13, 1),
            "             ",
            self.historical_span_style((128, 128, 128), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(39, 8, 1, 1),
            " ",
            self.historical_span_style((38, 38, 38), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(40, 8, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(41, 8, 1, 1),
            " ",
            self.historical_span_style((38, 38, 38), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(42, 8, 2, 1),
            "  ",
            self.historical_span_style((128, 128, 128), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(44, 8, 7, 1),
            "       ",
            self.historical_span_style((38, 38, 38), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(51, 8, 43, 1),
            "                                           ",
            self.historical_span_style((128, 128, 128), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(94, 8, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(95, 8, 21, 1),
            "rd available         ",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(116, 8, 1, 1),
            "┃",
            self.historical_span_style((38, 38, 38), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(117, 8, 2, 1),
            "  ",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 8, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 9, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 9, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 9, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(3, 9, 1, 1),
            " ",
            self.historical_span_style((0, 0, 0), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(4, 9, 3, 1),
            "   ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(7, 9, 1, 1),
            "★",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(8, 9, 17, 1),
            " Primary         ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(25, 9, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(26, 9, 13, 1),
            "             ",
            self.historical_span_style((128, 128, 128), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(39, 9, 1, 1),
            " ",
            self.historical_span_style((38, 38, 38), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(40, 9, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(41, 9, 1, 1),
            " ",
            self.historical_span_style((38, 38, 38), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(42, 9, 52, 1),
            "                                                    ",
            self.historical_span_style((128, 128, 128), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(94, 9, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(95, 9, 21, 1),
            "                     ",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(116, 9, 1, 1),
            "┃",
            self.historical_span_style((38, 38, 38), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(117, 9, 2, 1),
            "  ",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 9, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 10, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 10, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 10, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(3, 10, 1, 1),
            " ",
            self.historical_span_style((0, 0, 0), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(4, 10, 3, 1),
            "   ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(7, 10, 1, 1),
            " ",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(8, 10, 17, 1),
            " Experiments     ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(25, 10, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(26, 10, 3, 1),
            "   ",
            self.historical_span_style((128, 128, 128), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(29, 10, 15, 1),
            "Purpose label  ",
            self.historical_span_style((179, 179, 179), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(44, 10, 8, 1),
            "optional",
            self.historical_span_style((77, 77, 77), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(52, 10, 40, 1),
            "                                        ",
            self.historical_span_style((179, 179, 179), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(92, 10, 2, 1),
            "  ",
            self.historical_span_style((38, 38, 38), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(94, 10, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(95, 10, 22, 1),
            ", window and unit onl┃",
            self.historical_span_style((38, 38, 38), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(117, 10, 2, 1),
            "  ",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 10, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 11, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 11, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 11, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(3, 11, 1, 1),
            " ",
            self.historical_span_style((0, 0, 0), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(4, 11, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(5, 11, 1, 1),
            "▾",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(6, 11, 19, 1),
            " Amp               ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(25, 11, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(26, 11, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(27, 11, 1, 1),
            " ",
            self.historical_span_style((30, 30, 34), (30, 30, 34), false),
        );
        ui.paint_str(
            Rect::new(28, 11, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (30, 30, 34), false),
        );
        ui.paint_str(
            Rect::new(29, 11, 29, 1),
            "personal · work · experiments",
            self.historical_span_style((128, 128, 128), (30, 30, 34), false),
        );
        ui.paint_str(
            Rect::new(58, 11, 34, 1),
            "                                  ",
            self.historical_span_style((255, 255, 255), (30, 30, 34), false),
        );
        ui.paint_str(
            Rect::new(92, 11, 3, 1),
            "  │",
            self.historical_span_style((77, 77, 77), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(95, 11, 17, 1),
            "remaining (1 last",
            self.historical_span_style((77, 77, 77), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(112, 11, 4, 1),
            "    ",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(116, 11, 1, 1),
            "┃",
            self.historical_span_style((38, 38, 38), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(117, 11, 2, 1),
            "  ",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 11, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 12, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 12, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 12, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(3, 12, 1, 1),
            " ",
            self.historical_span_style((0, 0, 0), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(4, 12, 3, 1),
            "   ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(7, 12, 1, 1),
            " ",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(8, 12, 17, 1),
            " discovered      ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(25, 12, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(26, 12, 13, 1),
            "             ",
            self.historical_span_style((128, 128, 128), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(39, 12, 1, 1),
            " ",
            self.historical_span_style((38, 38, 38), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(40, 12, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(41, 12, 1, 1),
            " ",
            self.historical_span_style((38, 38, 38), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(42, 12, 2, 1),
            "  ",
            self.historical_span_style((128, 128, 128), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(44, 12, 5, 1),
            "     ",
            self.historical_span_style((77, 77, 77), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(49, 12, 45, 1),
            "                                             ",
            self.historical_span_style((128, 128, 128), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(94, 12, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(95, 12, 21, 1),
            "                     ",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(116, 12, 1, 1),
            "┃",
            self.historical_span_style((38, 38, 38), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(117, 12, 2, 1),
            "  ",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 12, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 13, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 13, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 13, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(3, 13, 1, 1),
            " ",
            self.historical_span_style((0, 0, 0), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(4, 13, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(5, 13, 1, 1),
            "▾",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(6, 13, 19, 1),
            " Grok              ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(25, 13, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(26, 13, 13, 1),
            "             ",
            self.historical_span_style((128, 128, 128), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(39, 13, 1, 1),
            " ",
            self.historical_span_style((38, 38, 38), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(40, 13, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(41, 13, 1, 1),
            " ",
            self.historical_span_style((38, 38, 38), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(42, 13, 2, 1),
            "  ",
            self.historical_span_style((128, 128, 128), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(44, 13, 51, 1),
            "                                                  │",
            self.historical_span_style((77, 77, 77), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(95, 13, 20, 1),
            "7% remaining (1 last",
            self.historical_span_style((77, 77, 77), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(115, 13, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(116, 13, 1, 1),
            "┃",
            self.historical_span_style((38, 38, 38), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(117, 13, 2, 1),
            "  ",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 13, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 14, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 14, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 14, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(3, 14, 1, 1),
            " ",
            self.historical_span_style((0, 0, 0), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(4, 14, 3, 1),
            "   ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(7, 14, 1, 1),
            "★",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(8, 14, 17, 1),
            " Team            ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(25, 14, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(26, 14, 3, 1),
            "   ",
            self.historical_span_style((128, 128, 128), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(29, 14, 8, 1),
            "Provider",
            self.historical_span_style((179, 179, 179), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(37, 14, 2, 1),
            "  ",
            self.historical_span_style((128, 128, 128), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(39, 14, 1, 1),
            " ",
            self.historical_span_style((38, 38, 38), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(40, 14, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(41, 14, 1, 1),
            " ",
            self.historical_span_style((38, 38, 38), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(42, 14, 2, 1),
            "  ",
            self.historical_span_style((128, 128, 128), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(44, 14, 5, 1),
            "     ",
            self.historical_span_style((77, 77, 77), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(49, 14, 45, 1),
            "                                             ",
            self.historical_span_style((128, 128, 128), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(94, 14, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(95, 14, 21, 1),
            "                     ",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(116, 14, 1, 1),
            "┃",
            self.historical_span_style((38, 38, 38), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(117, 14, 2, 1),
            "  ",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 14, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 15, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 15, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 15, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(3, 15, 1, 1),
            " ",
            self.historical_span_style((0, 0, 0), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(4, 15, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(5, 15, 1, 1),
            "▾",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(6, 15, 19, 1),
            " Z.AI              ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(25, 15, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(26, 15, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(27, 15, 1, 1),
            " ",
            self.historical_span_style((30, 30, 34), (30, 30, 34), false),
        );
        ui.paint_str(
            Rect::new(28, 15, 62, 1),
            " Claude Code · Anthropic / Claude                             ",
            self.historical_span_style((255, 255, 255), (30, 30, 34), false),
        );
        ui.paint_str(
            Rect::new(90, 15, 1, 1),
            "▾",
            self.historical_span_style((179, 179, 179), (30, 30, 34), false),
        );
        ui.paint_str(
            Rect::new(91, 15, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (30, 30, 34), false),
        );
        ui.paint_str(
            Rect::new(92, 15, 3, 1),
            "  │",
            self.historical_span_style((77, 77, 77), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(95, 15, 19, 1),
            "ining (1 last good)",
            self.historical_span_style((77, 77, 77), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(114, 15, 2, 1),
            "  ",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(116, 15, 1, 1),
            "┃",
            self.historical_span_style((38, 38, 38), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(117, 15, 2, 1),
            "  ",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 15, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 16, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 16, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 16, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(3, 16, 1, 1),
            " ",
            self.historical_span_style((0, 0, 0), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(4, 16, 3, 1),
            "   ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(7, 16, 1, 1),
            " ",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(8, 16, 17, 1),
            " discovered      ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(25, 16, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(26, 16, 13, 1),
            "             ",
            self.historical_span_style((128, 128, 128), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(39, 16, 1, 1),
            " ",
            self.historical_span_style((38, 38, 38), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(40, 16, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(41, 16, 1, 1),
            " ",
            self.historical_span_style((38, 38, 38), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(42, 16, 2, 1),
            "  ",
            self.historical_span_style((128, 128, 128), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(44, 16, 51, 1),
            "                                                  │",
            self.historical_span_style((77, 77, 77), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(95, 16, 11, 1),
            " last good)",
            self.historical_span_style((77, 77, 77), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(106, 16, 10, 1),
            "          ",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(116, 16, 1, 1),
            "┃",
            self.historical_span_style((38, 38, 38), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(117, 16, 2, 1),
            "  ",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 16, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 17, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 17, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 17, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(3, 17, 1, 1),
            " ",
            self.historical_span_style((0, 0, 0), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(4, 17, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(5, 17, 1, 1),
            "▾",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(6, 17, 19, 1),
            " Kimi              ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(25, 17, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(26, 17, 13, 1),
            "             ",
            self.historical_span_style((128, 128, 128), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(39, 17, 1, 1),
            " ",
            self.historical_span_style((38, 38, 38), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(40, 17, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(41, 17, 1, 1),
            " ",
            self.historical_span_style((38, 38, 38), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(42, 17, 2, 1),
            "  ",
            self.historical_span_style((128, 128, 128), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(44, 17, 51, 1),
            "                                                  │",
            self.historical_span_style((77, 77, 77), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(95, 17, 20, 1),
            "aining (1 last good)",
            self.historical_span_style((77, 77, 77), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(115, 17, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(116, 17, 1, 1),
            "┃",
            self.historical_span_style((38, 38, 38), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(117, 17, 2, 1),
            "  ",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 17, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 18, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 18, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 18, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(3, 18, 1, 1),
            " ",
            self.historical_span_style((0, 0, 0), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(4, 18, 3, 1),
            "   ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(7, 18, 1, 1),
            " ",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(8, 18, 17, 1),
            " discovered      ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(25, 18, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(26, 18, 65, 1),
            "   Agent runtime Claude Code · provider Anthropic / Claude · usa…",
            self.historical_span_style((128, 128, 128), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(91, 18, 4, 1),
            "   │",
            self.historical_span_style((77, 77, 77), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(95, 18, 18, 1),
            "ning (1 last good)",
            self.historical_span_style((77, 77, 77), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(113, 18, 3, 1),
            "   ",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(116, 18, 1, 1),
            "┃",
            self.historical_span_style((38, 38, 38), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(117, 18, 2, 1),
            "  ",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 18, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 19, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 19, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 19, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(3, 19, 1, 1),
            " ",
            self.historical_span_style((0, 0, 0), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(4, 19, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(5, 19, 1, 1),
            "▾",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(6, 19, 19, 1),
            " MiniMax           ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(25, 19, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(26, 19, 13, 1),
            "             ",
            self.historical_span_style((128, 128, 128), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(39, 19, 1, 1),
            " ",
            self.historical_span_style((38, 38, 38), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(40, 19, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(41, 19, 1, 1),
            " ",
            self.historical_span_style((38, 38, 38), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(42, 19, 52, 1),
            "                                                    ",
            self.historical_span_style((128, 128, 128), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(94, 19, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(95, 19, 21, 1),
            "                     ",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(116, 19, 1, 1),
            "┃",
            self.historical_span_style((38, 38, 38), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(117, 19, 2, 1),
            "  ",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 19, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 20, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 20, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 20, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(3, 20, 1, 1),
            " ",
            self.historical_span_style((0, 0, 0), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(4, 20, 3, 1),
            "   ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(7, 20, 1, 1),
            " ",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(8, 20, 17, 1),
            " discovered      ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(25, 20, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(26, 20, 3, 1),
            "   ",
            self.historical_span_style((128, 128, 128), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(29, 20, 17, 1),
            "Credential source",
            self.historical_span_style((179, 179, 179), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(46, 20, 12, 1),
            "            ",
            self.historical_span_style((77, 77, 77), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(58, 20, 36, 1),
            "                                    ",
            self.historical_span_style((128, 128, 128), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(94, 20, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(95, 20, 21, 1),
            "                     ",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(116, 20, 1, 1),
            "┃",
            self.historical_span_style((38, 38, 38), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(117, 20, 2, 1),
            "  ",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 20, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 21, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 21, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 21, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(3, 21, 1, 1),
            " ",
            self.historical_span_style((0, 0, 0), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(4, 21, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(5, 21, 1, 1),
            "▾",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(6, 21, 19, 1),
            " OpenCode          ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(25, 21, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(26, 21, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(27, 21, 1, 1),
            " ",
            self.historical_span_style((24, 24, 27), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(28, 21, 3, 1),
            "(●)",
            self.historical_span_style((72, 224, 84), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(31, 21, 61, 1),
            " 1Password item / field  (recommended)                       ",
            self.historical_span_style((255, 255, 255), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(92, 21, 2, 1),
            "  ",
            self.historical_span_style((38, 38, 38), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(94, 21, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(95, 21, 15, 1),
            "ngle account or",
            self.historical_span_style((38, 38, 38), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(110, 21, 6, 1),
            "      ",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(116, 21, 1, 1),
            "┃",
            self.historical_span_style((38, 38, 38), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(117, 21, 2, 1),
            "  ",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 21, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 22, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 22, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 22, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(3, 22, 1, 1),
            " ",
            self.historical_span_style((0, 0, 0), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(4, 22, 3, 1),
            "   ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(7, 22, 1, 1),
            "★",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(8, 22, 17, 1),
            " Go subscription ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(25, 22, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(26, 22, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(27, 22, 1, 1),
            " ",
            self.historical_span_style((24, 24, 27), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(28, 22, 3, 1),
            "( )",
            self.historical_span_style((128, 128, 128), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(31, 22, 61, 1),
            " Local agent folder                                          ",
            self.historical_span_style((255, 255, 255), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(92, 22, 2, 1),
            "  ",
            self.historical_span_style((38, 38, 38), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(94, 22, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(95, 22, 14, 1),
            "ent windows) ·",
            self.historical_span_style((38, 38, 38), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(109, 22, 7, 1),
            "       ",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(116, 22, 1, 1),
            "┃",
            self.historical_span_style((38, 38, 38), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(117, 22, 2, 1),
            "  ",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 22, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 23, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 23, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 23, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(3, 23, 1, 1),
            " ",
            self.historical_span_style((0, 0, 0), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(4, 23, 3, 1),
            "   ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(7, 23, 1, 1),
            " ",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(8, 23, 17, 1),
            " ci-bot · discove",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(25, 23, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(26, 23, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(27, 23, 1, 1),
            " ",
            self.historical_span_style((24, 24, 27), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(28, 23, 3, 1),
            "( )",
            self.historical_span_style((128, 128, 128), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(31, 23, 61, 1),
            " Plain-text API key                                          ",
            self.historical_span_style((255, 255, 255), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(92, 23, 2, 1),
            "  ",
            self.historical_span_style((38, 38, 38), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(94, 23, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(95, 23, 13, 1),
            "nCode (single",
            self.historical_span_style((38, 38, 38), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(108, 23, 8, 1),
            "        ",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(116, 23, 1, 1),
            "┃",
            self.historical_span_style((38, 38, 38), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(117, 23, 2, 1),
            "  ",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 23, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 24, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 24, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 24, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(3, 24, 1, 1),
            " ",
            self.historical_span_style((0, 0, 0), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(4, 24, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(5, 24, 1, 1),
            " ",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(6, 24, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(7, 24, 18, 1),
            "Unsupported       ",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(25, 24, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(26, 24, 11, 1),
            "           ",
            self.historical_span_style((38, 38, 38), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(37, 24, 2, 1),
            "  ",
            self.historical_span_style((128, 128, 128), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(39, 24, 1, 1),
            " ",
            self.historical_span_style((38, 38, 38), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(40, 24, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(41, 24, 1, 1),
            " ",
            self.historical_span_style((38, 38, 38), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(42, 24, 2, 1),
            "  ",
            self.historical_span_style((128, 128, 128), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(44, 24, 29, 1),
            "                             ",
            self.historical_span_style((38, 38, 38), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(73, 24, 21, 1),
            "                     ",
            self.historical_span_style((128, 128, 128), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(94, 24, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(95, 24, 21, 1),
            "                     ",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(116, 24, 1, 1),
            "┃",
            self.historical_span_style((38, 38, 38), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(117, 24, 2, 1),
            "  ",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 24, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 25, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 25, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 25, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(3, 25, 1, 1),
            " ",
            self.historical_span_style((0, 0, 0), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(4, 25, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(5, 25, 1, 1),
            " ",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(6, 25, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(7, 25, 18, 1),
            "+ Add account…    ",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(25, 25, 4, 1),
            "│   ",
            self.historical_span_style((77, 77, 77), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(29, 25, 19, 1),
            "1Password reference",
            self.historical_span_style((179, 179, 179), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(48, 25, 46, 1),
            "                                              ",
            self.historical_span_style((128, 128, 128), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(94, 25, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(95, 25, 21, 1),
            "                     ",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(116, 25, 1, 1),
            "┃",
            self.historical_span_style((38, 38, 38), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(117, 25, 2, 1),
            "  ",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 25, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 26, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 26, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 26, 23, 1),
            "                       ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(25, 26, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(26, 26, 3, 1),
            "   ",
            self.historical_span_style((128, 128, 128), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(29, 26, 10, 1),
            "not chosen",
            self.historical_span_style((255, 255, 255), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(39, 26, 1, 1),
            " ",
            self.historical_span_style((38, 38, 38), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(40, 26, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(41, 26, 1, 1),
            " ",
            self.historical_span_style((38, 38, 38), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(42, 26, 2, 1),
            "  ",
            self.historical_span_style((128, 128, 128), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(44, 26, 8, 1),
            "        ",
            self.historical_span_style((77, 77, 77), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(52, 26, 30, 1),
            "                              ",
            self.historical_span_style((128, 128, 128), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(82, 26, 1, 1),
            " ",
            self.historical_span_style((39, 39, 42), (39, 39, 42), false),
        );
        ui.paint_str(
            Rect::new(83, 26, 8, 1),
            "Choose… ",
            self.historical_span_style((255, 255, 255), (39, 39, 42), false),
        );
        ui.paint_str(
            Rect::new(91, 26, 3, 1),
            "   ",
            self.historical_span_style((128, 128, 128), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(94, 26, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(95, 26, 21, 1),
            "                     ",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(116, 26, 1, 1),
            "┃",
            self.historical_span_style((38, 38, 38), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(117, 26, 2, 1),
            "  ",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 26, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 27, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 27, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 27, 23, 1),
            "                       ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(25, 27, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(26, 27, 13, 1),
            "             ",
            self.historical_span_style((128, 128, 128), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(39, 27, 1, 1),
            " ",
            self.historical_span_style((38, 38, 38), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(40, 27, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(41, 27, 1, 1),
            " ",
            self.historical_span_style((38, 38, 38), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(42, 27, 52, 1),
            "                                                    ",
            self.historical_span_style((128, 128, 128), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(94, 27, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(95, 27, 21, 1),
            "s in 1 h 5 min       ",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(116, 27, 1, 1),
            "┃",
            self.historical_span_style((38, 38, 38), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(117, 27, 2, 1),
            "  ",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 27, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 28, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 28, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 28, 23, 1),
            "                       ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(25, 28, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(26, 28, 13, 1),
            "             ",
            self.historical_span_style((128, 128, 128), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(39, 28, 1, 1),
            " ",
            self.historical_span_style((38, 38, 38), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(40, 28, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(41, 28, 1, 1),
            " ",
            self.historical_span_style((38, 38, 38), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(42, 28, 52, 1),
            "                                                    ",
            self.historical_span_style((128, 128, 128), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(94, 28, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(95, 28, 21, 1),
            "sets Sun 19:14       ",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(116, 28, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(117, 28, 2, 1),
            "  ",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 28, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 29, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 29, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 29, 23, 1),
            "                       ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(25, 29, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(26, 29, 13, 1),
            "             ",
            self.historical_span_style((128, 128, 128), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(39, 29, 1, 1),
            " ",
            self.historical_span_style((38, 38, 38), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(40, 29, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(41, 29, 1, 1),
            " ",
            self.historical_span_style((38, 38, 38), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(42, 29, 52, 1),
            "                                                    ",
            self.historical_span_style((128, 128, 128), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(94, 29, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(95, 29, 21, 1),
            "Sun 19:14            ",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(116, 29, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(117, 29, 2, 1),
            "  ",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 29, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 30, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 30, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 30, 23, 1),
            "                       ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(25, 30, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(26, 30, 13, 1),
            "             ",
            self.historical_span_style((128, 128, 128), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(39, 30, 1, 1),
            " ",
            self.historical_span_style((38, 38, 38), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(40, 30, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(41, 30, 1, 1),
            " ",
            self.historical_span_style((38, 38, 38), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(42, 30, 52, 1),
            "                                                    ",
            self.historical_span_style((128, 128, 128), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(94, 30, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(95, 30, 21, 1),
            "                     ",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(116, 30, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(117, 30, 2, 1),
            "  ",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 30, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 31, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 31, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 31, 23, 1),
            "                       ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(25, 31, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(26, 31, 13, 1),
            "             ",
            self.historical_span_style((128, 128, 128), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(39, 31, 1, 1),
            " ",
            self.historical_span_style((38, 38, 38), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(40, 31, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(41, 31, 1, 1),
            " ",
            self.historical_span_style((38, 38, 38), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(42, 31, 52, 1),
            "                                                    ",
            self.historical_span_style((128, 128, 128), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(94, 31, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(95, 31, 21, 1),
            "sets in 10 h         ",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(116, 31, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(117, 31, 2, 1),
            "  ",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 31, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 32, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 32, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 32, 23, 1),
            "                       ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(25, 32, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(26, 32, 13, 1),
            "             ",
            self.historical_span_style((128, 128, 128), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(39, 32, 1, 1),
            " ",
            self.historical_span_style((38, 38, 38), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(40, 32, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(41, 32, 1, 1),
            " ",
            self.historical_span_style((38, 38, 38), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(42, 32, 52, 1),
            "                                                    ",
            self.historical_span_style((128, 128, 128), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(94, 32, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(95, 32, 21, 1),
            "09:14                ",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(116, 32, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(117, 32, 2, 1),
            "  ",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 32, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 33, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 33, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 33, 23, 1),
            "                       ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(25, 33, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(26, 33, 13, 1),
            "             ",
            self.historical_span_style((128, 128, 128), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(39, 33, 1, 1),
            " ",
            self.historical_span_style((24, 24, 27), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(40, 33, 25, 1),
            "Enter plain text instead ",
            self.historical_span_style((179, 179, 179), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(65, 33, 1, 1),
            " ",
            self.historical_span_style((38, 38, 38), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(66, 33, 1, 1),
            " ",
            self.historical_span_style((39, 39, 42), (39, 39, 42), false),
        );
        ui.paint_str(
            Rect::new(67, 33, 9, 1),
            "Validate ",
            self.historical_span_style((255, 255, 255), (39, 39, 42), false),
        );
        ui.paint_str(
            Rect::new(76, 33, 1, 1),
            " ",
            self.historical_span_style((38, 38, 38), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(77, 33, 1, 1),
            " ",
            self.historical_span_style((24, 24, 27), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(78, 33, 7, 1),
            "Cancel ",
            self.historical_span_style((179, 179, 179), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(85, 33, 1, 1),
            " ",
            self.historical_span_style((38, 38, 38), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(86, 33, 1, 1),
            " ",
            self.historical_span_style((72, 224, 84), (72, 224, 84), false),
        );
        ui.paint_str(
            Rect::new(87, 33, 5, 1),
            "Save ",
            self.historical_span_style((25, 25, 28), (72, 224, 84), true),
        );
        ui.paint_str(
            Rect::new(92, 33, 2, 1),
            "  ",
            self.historical_span_style((38, 38, 38), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(94, 33, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(95, 33, 22, 1),
            "                     │",
            self.historical_span_style((38, 38, 38), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(117, 33, 2, 1),
            "  ",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 33, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 34, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 34, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 34, 23, 1),
            "                       ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(25, 34, 70, 1),
            "╰────────────────────────────────────────────────────────────────────╯",
            self.historical_span_style((77, 77, 77), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(95, 34, 22, 1),
            "has no api key       │",
            self.historical_span_style((38, 38, 38), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(117, 34, 2, 1),
            "  ",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 34, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 35, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 35, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 35, 37, 1),
            "                                     ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(39, 35, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(40, 35, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(41, 35, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(42, 35, 77, 1),
            "                                                                             ",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 35, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 36, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 36, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 36, 37, 1),
            "                                     ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(39, 36, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(40, 36, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(41, 36, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(42, 36, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(43, 36, 1, 1),
            " ",
            self.historical_span_style((39, 39, 42), (39, 39, 42), false),
        );
        ui.paint_str(
            Rect::new(44, 36, 12, 1),
            "Refresh all ",
            self.historical_span_style((128, 128, 128), (39, 39, 42), false),
        );
        ui.paint_str(
            Rect::new(56, 36, 2, 1),
            "  ",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(58, 36, 1, 1),
            " ",
            self.historical_span_style((39, 39, 42), (39, 39, 42), false),
        );
        ui.paint_str(
            Rect::new(59, 36, 13, 1),
            "Add account… ",
            self.historical_span_style((128, 128, 128), (39, 39, 42), false),
        );
        ui.paint_str(
            Rect::new(72, 36, 47, 1),
            "                                               ",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 36, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 37, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 37, 39, 1),
            "╰─────────────────────────────────────╯",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(40, 37, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(41, 37, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(42, 37, 77, 1),
            "                                                                             ",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 37, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(Rect::new(0, 38, 120, 1), "                                                                                                                        ", self.historical_span_style((128, 128, 128), (0, 0, 0), false));
        ui.paint_str(
            Rect::new(0, 39, 26, 1),
            "                          ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(26, 39, 5, 1),
            "     ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), true),
        );
        ui.paint_str(
            Rect::new(31, 39, 5, 1),
            "     ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(36, 39, 3, 1),
            "Tab",
            self.historical_span_style((255, 255, 255), (0, 0, 0), true),
        );
        ui.paint_str(
            Rect::new(39, 39, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(40, 39, 1, 1),
            "N",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(41, 39, 1, 1),
            "e",
            self.historical_span_style((128, 128, 128), (0, 0, 0), true),
        );
        ui.paint_str(
            Rect::new(42, 39, 8, 1),
            "xt field",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(50, 39, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(52, 39, 5, 1),
            "Enter",
            self.historical_span_style((255, 255, 255), (0, 0, 0), true),
        );
        ui.paint_str(
            Rect::new(57, 39, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(58, 39, 6, 1),
            "Edit /",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(64, 39, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), true),
        );
        ui.paint_str(
            Rect::new(65, 39, 4, 1),
            "Save",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(69, 39, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(71, 39, 4, 1),
            "Esc ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), true),
        );
        ui.paint_str(
            Rect::new(75, 39, 6, 1),
            "Cancel",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(81, 39, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(83, 39, 3, 1),
            "   ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), true),
        );
        ui.paint_str(
            Rect::new(86, 39, 34, 1),
            "                                  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
    }

    pub(super) fn draw_historical_accounts_add_form_required_120_40(
        &self,
        ui: &mut Ui<'_>,
        area: Rect,
    ) {
        ui.fill(
            area,
            self.historical_span_style((128, 128, 128), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(0, 0, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 0, 9, 1),
            " jackin❯ ",
            self.historical_span_style((77, 77, 77), (39, 39, 42), false),
        );
        ui.paint_str(
            Rect::new(10, 0, 2, 1),
            "  ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(12, 0, 6, 1),
            " File ",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(18, 0, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(19, 0, 4, 1),
            " Go ",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(23, 0, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(24, 0, 6, 1),
            " Help ",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(30, 0, 21, 1),
            "                     ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(51, 0, 19, 1),
            "Accounts › Overview",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(70, 0, 2, 1),
            "  ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(72, 0, 14, 1),
            "⠋ refreshing 1",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(86, 0, 2, 1),
            "  ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(88, 0, 20, 1),
            "inside the Construct",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(108, 0, 2, 1),
            "  ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(110, 0, 9, 1),
            "2 running",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(119, 0, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(Rect::new(0, 1, 120, 1), "                                                                                                                        ", self.historical_span_style((128, 128, 128), (0, 0, 0), false));
        ui.paint_str(
            Rect::new(0, 2, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 2, 2, 1),
            "╭─",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(3, 2, 10, 1),
            " Accounts ",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(13, 2, 27, 1),
            "───────── 12 · 4 ▲ · 4 ! ─╮",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(40, 2, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(41, 2, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(42, 2, 2, 1),
            "  ",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(44, 2, 8, 1),
            "Overview",
            self.historical_span_style((77, 77, 77), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(52, 2, 40, 1),
            "                                        ",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(92, 2, 25, 1),
            "12 accounts · 8 providers",
            self.historical_span_style((38, 38, 38), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(117, 2, 2, 1),
            "  ",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 2, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 3, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 3, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 3, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(3, 3, 1, 1),
            " ",
            self.historical_span_style((0, 0, 0), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(4, 3, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(5, 3, 1, 1),
            " ",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(6, 3, 33, 1),
            " Overview                        ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(39, 3, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(40, 3, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(41, 3, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(42, 3, 77, 1),
            "                                                                             ",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 3, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 4, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 4, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 4, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(3, 4, 1, 1),
            " ",
            self.historical_span_style((0, 0, 0), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(4, 4, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(5, 4, 1, 1),
            "▾",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(6, 4, 33, 1),
            " Claude                      !   ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(39, 4, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(40, 4, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(41, 4, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(42, 4, 2, 1),
            "  ",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(44, 4, 6, 1),
            "Health",
            self.historical_span_style((38, 38, 38), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(50, 4, 66, 1),
            "        degraded · 4 warnings · 1 exhausted · 2 stale · 1         ",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(116, 4, 1, 1),
            "┃",
            self.historical_span_style((38, 38, 38), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(117, 4, 2, 1),
            "  ",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 4, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 5, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 5, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 5, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(3, 5, 1, 1),
            " ",
            self.historical_span_style((0, 0, 0), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(4, 5, 3, 1),
            "   ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(7, 5, 1, 1),
            "★",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(8, 5, 17, 1),
            " Personal        ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(25, 5, 2, 1),
            "╭─",
            self.historical_span_style((77, 77, 77), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(27, 5, 13, 1),
            " New account ",
            self.historical_span_style((255, 255, 255), (24, 24, 27), true),
        );
        ui.paint_str(
            Rect::new(40, 5, 55, 1),
            "───────────────────────────────────── form · unsaved ─╮",
            self.historical_span_style((77, 77, 77), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(95, 5, 21, 1),
            "isible · 2 identities",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(116, 5, 1, 1),
            "┃",
            self.historical_span_style((38, 38, 38), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(117, 5, 2, 1),
            "  ",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 5, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 6, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 6, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 6, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(3, 6, 1, 1),
            " ",
            self.historical_span_style((0, 0, 0), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(4, 6, 3, 1),
            "   ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(7, 6, 1, 1),
            " ",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(8, 6, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(9, 6, 16, 1),
            "Archived contrac",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(25, 6, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(26, 6, 3, 1),
            "   ",
            self.historical_span_style((38, 38, 38), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(29, 6, 13, 1),
            "Display name ",
            self.historical_span_style((255, 255, 255), (24, 24, 27), true),
        );
        ui.paint_str(
            Rect::new(42, 6, 1, 1),
            "*",
            self.historical_span_style((72, 224, 84), (24, 24, 27), true),
        );
        ui.paint_str(
            Rect::new(43, 6, 49, 1),
            "                                                 ",
            self.historical_span_style((255, 255, 255), (24, 24, 27), true),
        );
        ui.paint_str(
            Rect::new(92, 6, 2, 1),
            "  ",
            self.historical_span_style((128, 128, 128), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(94, 6, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(95, 6, 21, 1),
            "                     ",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(116, 6, 1, 1),
            "┃",
            self.historical_span_style((38, 38, 38), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(117, 6, 2, 1),
            "  ",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 6, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 7, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 7, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 7, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(3, 7, 1, 1),
            " ",
            self.historical_span_style((0, 0, 0), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(4, 7, 3, 1),
            "   ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(7, 7, 1, 1),
            " ",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(8, 7, 17, 1),
            " Work            ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(25, 7, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(26, 7, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(27, 7, 1, 1),
            "▎",
            self.historical_span_style((72, 224, 84), (30, 30, 34), false),
        );
        ui.paint_str(
            Rect::new(28, 7, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (30, 30, 34), false),
        );
        ui.paint_str(
            Rect::new(29, 7, 21, 1),
            "Personal, Work, Team…",
            self.historical_span_style((128, 128, 128), (30, 30, 34), false),
        );
        ui.paint_str(
            Rect::new(50, 7, 40, 1),
            "                                        ",
            self.historical_span_style((255, 255, 255), (30, 30, 34), false),
        );
        ui.paint_str(
            Rect::new(90, 7, 1, 1),
            "!",
            self.historical_span_style((228, 69, 69), (30, 30, 34), true),
        );
        ui.paint_str(
            Rect::new(91, 7, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (30, 30, 34), false),
        );
        ui.paint_str(
            Rect::new(92, 7, 2, 1),
            "  ",
            self.historical_span_style((128, 128, 128), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(94, 7, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(95, 7, 21, 1),
            " · 8 providers       ",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(116, 7, 1, 1),
            "┃",
            self.historical_span_style((38, 38, 38), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(117, 7, 2, 1),
            "  ",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 7, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 8, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 8, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 8, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(3, 8, 1, 1),
            " ",
            self.historical_span_style((0, 0, 0), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(4, 8, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(5, 8, 1, 1),
            "▾",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(6, 8, 19, 1),
            " Codex             ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(25, 8, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(26, 8, 3, 1),
            "   ",
            self.historical_span_style((128, 128, 128), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(29, 8, 8, 1),
            "Required",
            self.historical_span_style((228, 69, 69), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(37, 8, 2, 1),
            "  ",
            self.historical_span_style((128, 128, 128), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(39, 8, 1, 1),
            " ",
            self.historical_span_style((38, 38, 38), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(40, 8, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(41, 8, 1, 1),
            " ",
            self.historical_span_style((38, 38, 38), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(42, 8, 2, 1),
            "  ",
            self.historical_span_style((128, 128, 128), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(44, 8, 7, 1),
            "       ",
            self.historical_span_style((38, 38, 38), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(51, 8, 43, 1),
            "                                           ",
            self.historical_span_style((128, 128, 128), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(94, 8, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(95, 8, 21, 1),
            "rd available         ",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(116, 8, 1, 1),
            "┃",
            self.historical_span_style((38, 38, 38), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(117, 8, 2, 1),
            "  ",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 8, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 9, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 9, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 9, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(3, 9, 1, 1),
            " ",
            self.historical_span_style((0, 0, 0), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(4, 9, 3, 1),
            "   ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(7, 9, 1, 1),
            "★",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(8, 9, 17, 1),
            " Primary         ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(25, 9, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(26, 9, 13, 1),
            "             ",
            self.historical_span_style((128, 128, 128), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(39, 9, 1, 1),
            " ",
            self.historical_span_style((38, 38, 38), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(40, 9, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(41, 9, 1, 1),
            " ",
            self.historical_span_style((38, 38, 38), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(42, 9, 52, 1),
            "                                                    ",
            self.historical_span_style((128, 128, 128), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(94, 9, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(95, 9, 21, 1),
            "                     ",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(116, 9, 1, 1),
            "┃",
            self.historical_span_style((38, 38, 38), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(117, 9, 2, 1),
            "  ",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 9, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 10, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 10, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 10, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(3, 10, 1, 1),
            " ",
            self.historical_span_style((0, 0, 0), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(4, 10, 3, 1),
            "   ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(7, 10, 1, 1),
            " ",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(8, 10, 17, 1),
            " Experiments     ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(25, 10, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(26, 10, 3, 1),
            "   ",
            self.historical_span_style((128, 128, 128), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(29, 10, 15, 1),
            "Purpose label  ",
            self.historical_span_style((179, 179, 179), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(44, 10, 8, 1),
            "optional",
            self.historical_span_style((77, 77, 77), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(52, 10, 40, 1),
            "                                        ",
            self.historical_span_style((179, 179, 179), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(92, 10, 2, 1),
            "  ",
            self.historical_span_style((38, 38, 38), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(94, 10, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(95, 10, 22, 1),
            ", window and unit onl┃",
            self.historical_span_style((38, 38, 38), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(117, 10, 2, 1),
            "  ",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 10, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 11, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 11, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 11, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(3, 11, 1, 1),
            " ",
            self.historical_span_style((0, 0, 0), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(4, 11, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(5, 11, 1, 1),
            "▾",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(6, 11, 19, 1),
            " Amp               ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(25, 11, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(26, 11, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(27, 11, 1, 1),
            " ",
            self.historical_span_style((30, 30, 34), (30, 30, 34), false),
        );
        ui.paint_str(
            Rect::new(28, 11, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (30, 30, 34), false),
        );
        ui.paint_str(
            Rect::new(29, 11, 29, 1),
            "personal · work · experiments",
            self.historical_span_style((128, 128, 128), (30, 30, 34), false),
        );
        ui.paint_str(
            Rect::new(58, 11, 34, 1),
            "                                  ",
            self.historical_span_style((255, 255, 255), (30, 30, 34), false),
        );
        ui.paint_str(
            Rect::new(92, 11, 3, 1),
            "  │",
            self.historical_span_style((77, 77, 77), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(95, 11, 17, 1),
            "remaining (1 last",
            self.historical_span_style((77, 77, 77), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(112, 11, 4, 1),
            "    ",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(116, 11, 1, 1),
            "┃",
            self.historical_span_style((38, 38, 38), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(117, 11, 2, 1),
            "  ",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 11, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 12, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 12, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 12, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(3, 12, 1, 1),
            " ",
            self.historical_span_style((0, 0, 0), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(4, 12, 3, 1),
            "   ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(7, 12, 1, 1),
            " ",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(8, 12, 17, 1),
            " discovered      ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(25, 12, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(26, 12, 13, 1),
            "             ",
            self.historical_span_style((128, 128, 128), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(39, 12, 1, 1),
            " ",
            self.historical_span_style((38, 38, 38), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(40, 12, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(41, 12, 1, 1),
            " ",
            self.historical_span_style((38, 38, 38), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(42, 12, 2, 1),
            "  ",
            self.historical_span_style((128, 128, 128), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(44, 12, 5, 1),
            "     ",
            self.historical_span_style((77, 77, 77), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(49, 12, 45, 1),
            "                                             ",
            self.historical_span_style((128, 128, 128), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(94, 12, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(95, 12, 21, 1),
            "                     ",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(116, 12, 1, 1),
            "┃",
            self.historical_span_style((38, 38, 38), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(117, 12, 2, 1),
            "  ",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 12, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 13, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 13, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 13, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(3, 13, 1, 1),
            " ",
            self.historical_span_style((0, 0, 0), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(4, 13, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(5, 13, 1, 1),
            "▾",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(6, 13, 19, 1),
            " Grok              ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(25, 13, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(26, 13, 13, 1),
            "             ",
            self.historical_span_style((128, 128, 128), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(39, 13, 1, 1),
            " ",
            self.historical_span_style((38, 38, 38), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(40, 13, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(41, 13, 1, 1),
            " ",
            self.historical_span_style((38, 38, 38), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(42, 13, 2, 1),
            "  ",
            self.historical_span_style((128, 128, 128), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(44, 13, 51, 1),
            "                                                  │",
            self.historical_span_style((77, 77, 77), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(95, 13, 20, 1),
            "7% remaining (1 last",
            self.historical_span_style((77, 77, 77), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(115, 13, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(116, 13, 1, 1),
            "┃",
            self.historical_span_style((38, 38, 38), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(117, 13, 2, 1),
            "  ",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 13, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 14, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 14, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 14, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(3, 14, 1, 1),
            " ",
            self.historical_span_style((0, 0, 0), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(4, 14, 3, 1),
            "   ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(7, 14, 1, 1),
            "★",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(8, 14, 17, 1),
            " Team            ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(25, 14, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(26, 14, 3, 1),
            "   ",
            self.historical_span_style((128, 128, 128), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(29, 14, 8, 1),
            "Provider",
            self.historical_span_style((179, 179, 179), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(37, 14, 2, 1),
            "  ",
            self.historical_span_style((128, 128, 128), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(39, 14, 1, 1),
            " ",
            self.historical_span_style((38, 38, 38), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(40, 14, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(41, 14, 1, 1),
            " ",
            self.historical_span_style((38, 38, 38), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(42, 14, 2, 1),
            "  ",
            self.historical_span_style((128, 128, 128), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(44, 14, 5, 1),
            "     ",
            self.historical_span_style((77, 77, 77), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(49, 14, 45, 1),
            "                                             ",
            self.historical_span_style((128, 128, 128), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(94, 14, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(95, 14, 21, 1),
            "                     ",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(116, 14, 1, 1),
            "┃",
            self.historical_span_style((38, 38, 38), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(117, 14, 2, 1),
            "  ",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 14, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 15, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 15, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 15, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(3, 15, 1, 1),
            " ",
            self.historical_span_style((0, 0, 0), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(4, 15, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(5, 15, 1, 1),
            "▾",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(6, 15, 19, 1),
            " Z.AI              ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(25, 15, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(26, 15, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(27, 15, 1, 1),
            " ",
            self.historical_span_style((30, 30, 34), (30, 30, 34), false),
        );
        ui.paint_str(
            Rect::new(28, 15, 62, 1),
            " Claude Code · Anthropic / Claude                             ",
            self.historical_span_style((255, 255, 255), (30, 30, 34), false),
        );
        ui.paint_str(
            Rect::new(90, 15, 1, 1),
            "▾",
            self.historical_span_style((179, 179, 179), (30, 30, 34), false),
        );
        ui.paint_str(
            Rect::new(91, 15, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (30, 30, 34), false),
        );
        ui.paint_str(
            Rect::new(92, 15, 3, 1),
            "  │",
            self.historical_span_style((77, 77, 77), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(95, 15, 19, 1),
            "ining (1 last good)",
            self.historical_span_style((77, 77, 77), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(114, 15, 2, 1),
            "  ",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(116, 15, 1, 1),
            "┃",
            self.historical_span_style((38, 38, 38), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(117, 15, 2, 1),
            "  ",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 15, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 16, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 16, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 16, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(3, 16, 1, 1),
            " ",
            self.historical_span_style((0, 0, 0), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(4, 16, 3, 1),
            "   ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(7, 16, 1, 1),
            " ",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(8, 16, 17, 1),
            " discovered      ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(25, 16, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(26, 16, 13, 1),
            "             ",
            self.historical_span_style((128, 128, 128), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(39, 16, 1, 1),
            " ",
            self.historical_span_style((38, 38, 38), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(40, 16, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(41, 16, 1, 1),
            " ",
            self.historical_span_style((38, 38, 38), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(42, 16, 2, 1),
            "  ",
            self.historical_span_style((128, 128, 128), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(44, 16, 51, 1),
            "                                                  │",
            self.historical_span_style((77, 77, 77), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(95, 16, 11, 1),
            " last good)",
            self.historical_span_style((77, 77, 77), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(106, 16, 10, 1),
            "          ",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(116, 16, 1, 1),
            "┃",
            self.historical_span_style((38, 38, 38), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(117, 16, 2, 1),
            "  ",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 16, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 17, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 17, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 17, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(3, 17, 1, 1),
            " ",
            self.historical_span_style((0, 0, 0), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(4, 17, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(5, 17, 1, 1),
            "▾",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(6, 17, 19, 1),
            " Kimi              ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(25, 17, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(26, 17, 13, 1),
            "             ",
            self.historical_span_style((128, 128, 128), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(39, 17, 1, 1),
            " ",
            self.historical_span_style((38, 38, 38), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(40, 17, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(41, 17, 1, 1),
            " ",
            self.historical_span_style((38, 38, 38), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(42, 17, 2, 1),
            "  ",
            self.historical_span_style((128, 128, 128), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(44, 17, 51, 1),
            "                                                  │",
            self.historical_span_style((77, 77, 77), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(95, 17, 20, 1),
            "aining (1 last good)",
            self.historical_span_style((77, 77, 77), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(115, 17, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(116, 17, 1, 1),
            "┃",
            self.historical_span_style((38, 38, 38), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(117, 17, 2, 1),
            "  ",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 17, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 18, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 18, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 18, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(3, 18, 1, 1),
            " ",
            self.historical_span_style((0, 0, 0), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(4, 18, 3, 1),
            "   ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(7, 18, 1, 1),
            " ",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(8, 18, 17, 1),
            " discovered      ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(25, 18, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(26, 18, 65, 1),
            "   Agent runtime Claude Code · provider Anthropic / Claude · usa…",
            self.historical_span_style((128, 128, 128), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(91, 18, 4, 1),
            "   │",
            self.historical_span_style((77, 77, 77), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(95, 18, 18, 1),
            "ning (1 last good)",
            self.historical_span_style((77, 77, 77), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(113, 18, 3, 1),
            "   ",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(116, 18, 1, 1),
            "┃",
            self.historical_span_style((38, 38, 38), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(117, 18, 2, 1),
            "  ",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 18, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 19, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 19, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 19, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(3, 19, 1, 1),
            " ",
            self.historical_span_style((0, 0, 0), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(4, 19, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(5, 19, 1, 1),
            "▾",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(6, 19, 19, 1),
            " MiniMax           ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(25, 19, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(26, 19, 13, 1),
            "             ",
            self.historical_span_style((128, 128, 128), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(39, 19, 1, 1),
            " ",
            self.historical_span_style((38, 38, 38), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(40, 19, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(41, 19, 1, 1),
            " ",
            self.historical_span_style((38, 38, 38), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(42, 19, 52, 1),
            "                                                    ",
            self.historical_span_style((128, 128, 128), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(94, 19, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(95, 19, 21, 1),
            "                     ",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(116, 19, 1, 1),
            "┃",
            self.historical_span_style((38, 38, 38), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(117, 19, 2, 1),
            "  ",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 19, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 20, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 20, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 20, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(3, 20, 1, 1),
            " ",
            self.historical_span_style((0, 0, 0), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(4, 20, 3, 1),
            "   ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(7, 20, 1, 1),
            " ",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(8, 20, 17, 1),
            " discovered      ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(25, 20, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(26, 20, 3, 1),
            "   ",
            self.historical_span_style((128, 128, 128), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(29, 20, 17, 1),
            "Credential source",
            self.historical_span_style((179, 179, 179), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(46, 20, 12, 1),
            "            ",
            self.historical_span_style((77, 77, 77), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(58, 20, 36, 1),
            "                                    ",
            self.historical_span_style((128, 128, 128), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(94, 20, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(95, 20, 21, 1),
            "                     ",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(116, 20, 1, 1),
            "┃",
            self.historical_span_style((38, 38, 38), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(117, 20, 2, 1),
            "  ",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 20, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 21, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 21, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 21, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(3, 21, 1, 1),
            " ",
            self.historical_span_style((0, 0, 0), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(4, 21, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(5, 21, 1, 1),
            "▾",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(6, 21, 19, 1),
            " OpenCode          ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(25, 21, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(26, 21, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(27, 21, 1, 1),
            " ",
            self.historical_span_style((24, 24, 27), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(28, 21, 3, 1),
            "(●)",
            self.historical_span_style((72, 224, 84), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(31, 21, 61, 1),
            " 1Password item / field  (recommended)                       ",
            self.historical_span_style((255, 255, 255), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(92, 21, 2, 1),
            "  ",
            self.historical_span_style((38, 38, 38), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(94, 21, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(95, 21, 15, 1),
            "ngle account or",
            self.historical_span_style((38, 38, 38), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(110, 21, 6, 1),
            "      ",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(116, 21, 1, 1),
            "┃",
            self.historical_span_style((38, 38, 38), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(117, 21, 2, 1),
            "  ",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 21, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 22, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 22, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 22, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(3, 22, 1, 1),
            " ",
            self.historical_span_style((0, 0, 0), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(4, 22, 3, 1),
            "   ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(7, 22, 1, 1),
            "★",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(8, 22, 17, 1),
            " Go subscription ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(25, 22, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(26, 22, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(27, 22, 1, 1),
            " ",
            self.historical_span_style((24, 24, 27), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(28, 22, 3, 1),
            "( )",
            self.historical_span_style((128, 128, 128), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(31, 22, 61, 1),
            " Local agent folder                                          ",
            self.historical_span_style((255, 255, 255), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(92, 22, 2, 1),
            "  ",
            self.historical_span_style((38, 38, 38), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(94, 22, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(95, 22, 14, 1),
            "ent windows) ·",
            self.historical_span_style((38, 38, 38), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(109, 22, 7, 1),
            "       ",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(116, 22, 1, 1),
            "┃",
            self.historical_span_style((38, 38, 38), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(117, 22, 2, 1),
            "  ",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 22, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 23, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 23, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 23, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(3, 23, 1, 1),
            " ",
            self.historical_span_style((0, 0, 0), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(4, 23, 3, 1),
            "   ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(7, 23, 1, 1),
            " ",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(8, 23, 17, 1),
            " ci-bot · discove",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(25, 23, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(26, 23, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(27, 23, 1, 1),
            " ",
            self.historical_span_style((24, 24, 27), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(28, 23, 3, 1),
            "( )",
            self.historical_span_style((128, 128, 128), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(31, 23, 61, 1),
            " Plain-text API key                                          ",
            self.historical_span_style((255, 255, 255), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(92, 23, 2, 1),
            "  ",
            self.historical_span_style((38, 38, 38), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(94, 23, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(95, 23, 13, 1),
            "nCode (single",
            self.historical_span_style((38, 38, 38), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(108, 23, 8, 1),
            "        ",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(116, 23, 1, 1),
            "┃",
            self.historical_span_style((38, 38, 38), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(117, 23, 2, 1),
            "  ",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 23, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 24, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 24, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 24, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(3, 24, 1, 1),
            " ",
            self.historical_span_style((0, 0, 0), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(4, 24, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(5, 24, 1, 1),
            " ",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(6, 24, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(7, 24, 18, 1),
            "Unsupported       ",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(25, 24, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(26, 24, 11, 1),
            "           ",
            self.historical_span_style((38, 38, 38), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(37, 24, 2, 1),
            "  ",
            self.historical_span_style((128, 128, 128), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(39, 24, 1, 1),
            " ",
            self.historical_span_style((38, 38, 38), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(40, 24, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(41, 24, 1, 1),
            " ",
            self.historical_span_style((38, 38, 38), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(42, 24, 2, 1),
            "  ",
            self.historical_span_style((128, 128, 128), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(44, 24, 29, 1),
            "                             ",
            self.historical_span_style((38, 38, 38), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(73, 24, 21, 1),
            "                     ",
            self.historical_span_style((128, 128, 128), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(94, 24, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(95, 24, 21, 1),
            "                     ",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(116, 24, 1, 1),
            "┃",
            self.historical_span_style((38, 38, 38), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(117, 24, 2, 1),
            "  ",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 24, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 25, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 25, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 25, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(3, 25, 1, 1),
            " ",
            self.historical_span_style((0, 0, 0), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(4, 25, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(5, 25, 1, 1),
            " ",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(6, 25, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(7, 25, 18, 1),
            "+ Add account…    ",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(25, 25, 4, 1),
            "│   ",
            self.historical_span_style((77, 77, 77), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(29, 25, 19, 1),
            "1Password reference",
            self.historical_span_style((179, 179, 179), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(48, 25, 46, 1),
            "                                              ",
            self.historical_span_style((128, 128, 128), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(94, 25, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(95, 25, 21, 1),
            "                     ",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(116, 25, 1, 1),
            "┃",
            self.historical_span_style((38, 38, 38), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(117, 25, 2, 1),
            "  ",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 25, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 26, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 26, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 26, 23, 1),
            "                       ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(25, 26, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(26, 26, 3, 1),
            "   ",
            self.historical_span_style((128, 128, 128), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(29, 26, 10, 1),
            "not chosen",
            self.historical_span_style((255, 255, 255), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(39, 26, 1, 1),
            " ",
            self.historical_span_style((38, 38, 38), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(40, 26, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(41, 26, 1, 1),
            " ",
            self.historical_span_style((38, 38, 38), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(42, 26, 2, 1),
            "  ",
            self.historical_span_style((128, 128, 128), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(44, 26, 8, 1),
            "        ",
            self.historical_span_style((77, 77, 77), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(52, 26, 30, 1),
            "                              ",
            self.historical_span_style((128, 128, 128), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(82, 26, 1, 1),
            " ",
            self.historical_span_style((39, 39, 42), (39, 39, 42), false),
        );
        ui.paint_str(
            Rect::new(83, 26, 8, 1),
            "Choose… ",
            self.historical_span_style((255, 255, 255), (39, 39, 42), false),
        );
        ui.paint_str(
            Rect::new(91, 26, 3, 1),
            "   ",
            self.historical_span_style((128, 128, 128), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(94, 26, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(95, 26, 21, 1),
            "                     ",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(116, 26, 1, 1),
            "┃",
            self.historical_span_style((38, 38, 38), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(117, 26, 2, 1),
            "  ",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 26, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 27, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 27, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 27, 23, 1),
            "                       ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(25, 27, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(26, 27, 13, 1),
            "             ",
            self.historical_span_style((128, 128, 128), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(39, 27, 1, 1),
            " ",
            self.historical_span_style((38, 38, 38), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(40, 27, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(41, 27, 1, 1),
            " ",
            self.historical_span_style((38, 38, 38), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(42, 27, 52, 1),
            "                                                    ",
            self.historical_span_style((128, 128, 128), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(94, 27, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(95, 27, 21, 1),
            "s in 1 h 5 min       ",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(116, 27, 1, 1),
            "┃",
            self.historical_span_style((38, 38, 38), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(117, 27, 2, 1),
            "  ",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 27, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 28, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 28, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 28, 23, 1),
            "                       ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(25, 28, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(26, 28, 13, 1),
            "             ",
            self.historical_span_style((128, 128, 128), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(39, 28, 1, 1),
            " ",
            self.historical_span_style((38, 38, 38), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(40, 28, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(41, 28, 1, 1),
            " ",
            self.historical_span_style((38, 38, 38), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(42, 28, 52, 1),
            "                                                    ",
            self.historical_span_style((128, 128, 128), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(94, 28, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(95, 28, 21, 1),
            "sets Sun 19:14       ",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(116, 28, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(117, 28, 2, 1),
            "  ",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 28, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 29, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 29, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 29, 23, 1),
            "                       ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(25, 29, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(26, 29, 13, 1),
            "             ",
            self.historical_span_style((128, 128, 128), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(39, 29, 1, 1),
            " ",
            self.historical_span_style((38, 38, 38), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(40, 29, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(41, 29, 1, 1),
            " ",
            self.historical_span_style((38, 38, 38), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(42, 29, 52, 1),
            "                                                    ",
            self.historical_span_style((128, 128, 128), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(94, 29, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(95, 29, 21, 1),
            "Sun 19:14            ",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(116, 29, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(117, 29, 2, 1),
            "  ",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 29, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 30, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 30, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 30, 23, 1),
            "                       ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(25, 30, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(26, 30, 13, 1),
            "             ",
            self.historical_span_style((128, 128, 128), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(39, 30, 1, 1),
            " ",
            self.historical_span_style((38, 38, 38), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(40, 30, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(41, 30, 1, 1),
            " ",
            self.historical_span_style((38, 38, 38), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(42, 30, 52, 1),
            "                                                    ",
            self.historical_span_style((128, 128, 128), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(94, 30, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(95, 30, 21, 1),
            "                     ",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(116, 30, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(117, 30, 2, 1),
            "  ",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 30, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 31, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 31, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 31, 23, 1),
            "                       ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(25, 31, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(26, 31, 13, 1),
            "             ",
            self.historical_span_style((128, 128, 128), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(39, 31, 1, 1),
            " ",
            self.historical_span_style((38, 38, 38), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(40, 31, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(41, 31, 1, 1),
            " ",
            self.historical_span_style((38, 38, 38), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(42, 31, 52, 1),
            "                                                    ",
            self.historical_span_style((128, 128, 128), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(94, 31, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(95, 31, 21, 1),
            "sets in 10 h         ",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(116, 31, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(117, 31, 2, 1),
            "  ",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 31, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 32, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 32, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 32, 23, 1),
            "                       ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(25, 32, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(26, 32, 13, 1),
            "             ",
            self.historical_span_style((128, 128, 128), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(39, 32, 1, 1),
            " ",
            self.historical_span_style((38, 38, 38), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(40, 32, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(41, 32, 1, 1),
            " ",
            self.historical_span_style((38, 38, 38), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(42, 32, 52, 1),
            "                                                    ",
            self.historical_span_style((128, 128, 128), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(94, 32, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(95, 32, 21, 1),
            "09:14                ",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(116, 32, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(117, 32, 2, 1),
            "  ",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 32, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 33, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 33, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 33, 23, 1),
            "                       ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(25, 33, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(26, 33, 13, 1),
            "             ",
            self.historical_span_style((128, 128, 128), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(39, 33, 1, 1),
            " ",
            self.historical_span_style((24, 24, 27), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(40, 33, 25, 1),
            "Enter plain text instead ",
            self.historical_span_style((179, 179, 179), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(65, 33, 1, 1),
            " ",
            self.historical_span_style((38, 38, 38), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(66, 33, 1, 1),
            " ",
            self.historical_span_style((39, 39, 42), (39, 39, 42), false),
        );
        ui.paint_str(
            Rect::new(67, 33, 9, 1),
            "Validate ",
            self.historical_span_style((255, 255, 255), (39, 39, 42), false),
        );
        ui.paint_str(
            Rect::new(76, 33, 1, 1),
            " ",
            self.historical_span_style((38, 38, 38), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(77, 33, 1, 1),
            " ",
            self.historical_span_style((24, 24, 27), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(78, 33, 7, 1),
            "Cancel ",
            self.historical_span_style((179, 179, 179), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(85, 33, 1, 1),
            " ",
            self.historical_span_style((38, 38, 38), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(86, 33, 1, 1),
            " ",
            self.historical_span_style((72, 224, 84), (72, 224, 84), false),
        );
        ui.paint_str(
            Rect::new(87, 33, 5, 1),
            "Save ",
            self.historical_span_style((25, 25, 28), (72, 224, 84), true),
        );
        ui.paint_str(
            Rect::new(92, 33, 2, 1),
            "  ",
            self.historical_span_style((38, 38, 38), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(94, 33, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(95, 33, 22, 1),
            "                     │",
            self.historical_span_style((38, 38, 38), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(117, 33, 2, 1),
            "  ",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 33, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 34, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 34, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 34, 23, 1),
            "                       ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(25, 34, 70, 1),
            "╰────────────────────────────────────────────────────────────────────╯",
            self.historical_span_style((77, 77, 77), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(95, 34, 22, 1),
            "has no api key       │",
            self.historical_span_style((38, 38, 38), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(117, 34, 2, 1),
            "  ",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 34, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 35, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 35, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 35, 37, 1),
            "                                     ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(39, 35, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(40, 35, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(41, 35, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(42, 35, 77, 1),
            "                                                                             ",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 35, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 36, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 36, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 36, 37, 1),
            "                                     ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(39, 36, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(40, 36, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(41, 36, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(42, 36, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(43, 36, 1, 1),
            " ",
            self.historical_span_style((39, 39, 42), (39, 39, 42), false),
        );
        ui.paint_str(
            Rect::new(44, 36, 12, 1),
            "Refresh all ",
            self.historical_span_style((128, 128, 128), (39, 39, 42), false),
        );
        ui.paint_str(
            Rect::new(56, 36, 2, 1),
            "  ",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(58, 36, 1, 1),
            " ",
            self.historical_span_style((39, 39, 42), (39, 39, 42), false),
        );
        ui.paint_str(
            Rect::new(59, 36, 13, 1),
            "Add account… ",
            self.historical_span_style((128, 128, 128), (39, 39, 42), false),
        );
        ui.paint_str(
            Rect::new(72, 36, 47, 1),
            "                                               ",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 36, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 37, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 37, 39, 1),
            "╰─────────────────────────────────────╯",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(40, 37, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(41, 37, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(42, 37, 77, 1),
            "                                                                             ",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 37, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(Rect::new(0, 38, 120, 1), "                                                                                                                        ", self.historical_span_style((128, 128, 128), (0, 0, 0), false));
        ui.paint_str(
            Rect::new(0, 39, 26, 1),
            "                          ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(26, 39, 5, 1),
            "     ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), true),
        );
        ui.paint_str(
            Rect::new(31, 39, 5, 1),
            "     ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(36, 39, 3, 1),
            "Tab",
            self.historical_span_style((255, 255, 255), (0, 0, 0), true),
        );
        ui.paint_str(
            Rect::new(39, 39, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(40, 39, 1, 1),
            "N",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(41, 39, 1, 1),
            "e",
            self.historical_span_style((128, 128, 128), (0, 0, 0), true),
        );
        ui.paint_str(
            Rect::new(42, 39, 8, 1),
            "xt field",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(50, 39, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(52, 39, 5, 1),
            "Enter",
            self.historical_span_style((255, 255, 255), (0, 0, 0), true),
        );
        ui.paint_str(
            Rect::new(57, 39, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(58, 39, 6, 1),
            "Edit /",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(64, 39, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), true),
        );
        ui.paint_str(
            Rect::new(65, 39, 4, 1),
            "Save",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(69, 39, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(71, 39, 4, 1),
            "Esc ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), true),
        );
        ui.paint_str(
            Rect::new(75, 39, 6, 1),
            "Cancel",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(81, 39, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(83, 39, 3, 1),
            "   ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), true),
        );
        ui.paint_str(
            Rect::new(86, 39, 34, 1),
            "                                  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
    }

    pub(super) fn draw_historical_accounts_filter_120_40(&self, ui: &mut Ui<'_>, area: Rect) {
        ui.fill(
            area,
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(0, 0, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 0, 9, 1),
            " jackin❯ ",
            self.historical_span_style((25, 25, 28), (72, 224, 84), true),
        );
        ui.paint_str(
            Rect::new(10, 0, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(12, 0, 6, 1),
            " File ",
            self.historical_span_style((179, 179, 179), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(18, 0, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(19, 0, 4, 1),
            " Go ",
            self.historical_span_style((179, 179, 179), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(23, 0, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(24, 0, 6, 1),
            " Help ",
            self.historical_span_style((179, 179, 179), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(30, 0, 21, 1),
            "                     ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(51, 0, 19, 1),
            "Accounts › Overview",
            self.historical_span_style((179, 179, 179), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(70, 0, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(72, 0, 14, 1),
            "⠋ refreshing 1",
            self.historical_span_style((179, 179, 179), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(86, 0, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(88, 0, 20, 1),
            "inside the Construct",
            self.historical_span_style((179, 179, 179), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(108, 0, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(110, 0, 9, 1),
            "2 running",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(119, 0, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(Rect::new(0, 1, 120, 1), "                                                                                                                        ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(
            Rect::new(0, 2, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 2, 2, 1),
            "╭─",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(3, 2, 18, 1),
            " Accounts · filt… ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), true),
        );
        ui.paint_str(
            Rect::new(21, 2, 19, 1),
            "─ 12 · 4 ▲ · 4 ! ─╮",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(40, 2, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(41, 2, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(42, 2, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(44, 2, 8, 1),
            "Overview",
            self.historical_span_style((179, 179, 179), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(52, 2, 40, 1),
            "                                        ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(92, 2, 25, 1),
            "12 accounts · 8 providers",
            self.historical_span_style((77, 77, 77), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(117, 2, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 2, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 3, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 3, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 3, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(3, 3, 1, 1),
            "▎",
            self.historical_span_style((72, 224, 84), (15, 46, 19), true),
        );
        ui.paint_str(
            Rect::new(4, 3, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (15, 46, 19), true),
        );
        ui.paint_str(
            Rect::new(5, 3, 1, 1),
            " ",
            self.historical_span_style((179, 179, 179), (15, 46, 19), false),
        );
        ui.paint_str(
            Rect::new(6, 3, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (15, 46, 19), true),
        );
        ui.paint_str(
            Rect::new(7, 3, 30, 1),
            "Overview                      ",
            self.historical_span_style((72, 224, 84), (15, 46, 19), true),
        );
        ui.paint_str(
            Rect::new(37, 3, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (15, 46, 19), true),
        );
        ui.paint_str(
            Rect::new(38, 3, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(39, 3, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(40, 3, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(41, 3, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(42, 3, 77, 1),
            "                                                                             ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 3, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 4, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 4, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 4, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(3, 4, 1, 1),
            " ",
            self.historical_span_style((0, 0, 0), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(4, 4, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(5, 4, 1, 1),
            "▾",
            self.historical_span_style((179, 179, 179), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(6, 4, 29, 1),
            " Claude                      ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(35, 4, 1, 1),
            "!",
            self.historical_span_style((228, 69, 69), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(36, 4, 3, 1),
            "   ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(39, 4, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(40, 4, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(41, 4, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(42, 4, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(44, 4, 6, 1),
            "Health",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(50, 4, 8, 1),
            "        ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(58, 4, 49, 1),
            "degraded · 4 warnings · 1 exhausted · 2 stale · 1",
            self.historical_span_style((228, 69, 69), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(107, 4, 9, 1),
            "         ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(116, 4, 1, 1),
            "┃",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(117, 4, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 4, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 5, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 5, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 5, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(3, 5, 1, 1),
            " ",
            self.historical_span_style((0, 0, 0), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(4, 5, 3, 1),
            "   ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(7, 5, 1, 1),
            " ",
            self.historical_span_style((179, 179, 179), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(8, 5, 27, 1),
            " Work                      ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(35, 5, 1, 1),
            "!",
            self.historical_span_style((228, 69, 69), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(36, 5, 3, 1),
            "   ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(39, 5, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(40, 5, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(41, 5, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(42, 5, 16, 1),
            "                ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(58, 5, 58, 1),
            "refreshing · 3 failed · 1 quota not visible · 2 identities",
            self.historical_span_style((228, 69, 69), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(116, 5, 1, 1),
            "┃",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(117, 5, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 5, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 6, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 6, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 6, 37, 1),
            "                                     ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(39, 6, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(40, 6, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(41, 6, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(42, 6, 16, 1),
            "                ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(58, 6, 10, 1),
            "unresolved",
            self.historical_span_style((228, 69, 69), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(68, 6, 48, 1),
            "                                                ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(116, 6, 1, 1),
            "┃",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(117, 6, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 6, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 7, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 7, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 7, 37, 1),
            "                                     ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(39, 7, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(40, 7, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(41, 7, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(42, 7, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(44, 7, 8, 1),
            "Registry",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(52, 7, 64, 1),
            "      12 accounts · 11 enabled · 1 disabled · 8 providers       ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(116, 7, 1, 1),
            "┃",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(117, 7, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 7, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 8, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 8, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 8, 37, 1),
            "                                     ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(39, 8, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(40, 8, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(41, 8, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(42, 8, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(44, 8, 7, 1),
            "Sources",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(51, 8, 65, 1),
            "       7 registered · 5 discovered · 1Password available         ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(116, 8, 1, 1),
            "┃",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(117, 8, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 8, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 9, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 9, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 9, 37, 1),
            "                                     ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(39, 9, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(40, 9, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(41, 9, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(42, 9, 74, 1),
            "                                                                          ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(116, 9, 1, 1),
            "┃",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(117, 9, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 9, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 10, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 10, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 10, 37, 1),
            "                                     ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(39, 10, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(40, 10, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(41, 10, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(42, 10, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(44, 10, 18, 1),
            "Comparable windows",
            self.historical_span_style((179, 179, 179), (17, 17, 17), true),
        );
        ui.paint_str(
            Rect::new(62, 10, 15, 1),
            "               ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(77, 10, 39, 1),
            "identical provider, window and unit onl",
            self.historical_span_style((77, 77, 77), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(116, 10, 1, 1),
            "┃",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(117, 10, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 10, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 11, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 11, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 11, 37, 1),
            "                                     ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(39, 11, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(40, 11, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(41, 11, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(42, 11, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(44, 11, 68, 1),
            "Anthropic · Session · 5-hour   2 accounts · 24–62% remaining (1 last",
            self.historical_span_style((179, 179, 179), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(112, 11, 4, 1),
            "    ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(116, 11, 1, 1),
            "┃",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(117, 11, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 11, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 12, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 12, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 12, 37, 1),
            "                                     ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(39, 12, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(40, 12, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(41, 12, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(42, 12, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(44, 12, 5, 1),
            "good)",
            self.historical_span_style((179, 179, 179), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(49, 12, 67, 1),
            "                                                                   ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(116, 12, 1, 1),
            "┃",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(117, 12, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 12, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 13, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 13, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 13, 37, 1),
            "                                     ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(39, 13, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(40, 13, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(41, 13, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(42, 13, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(44, 13, 71, 1),
            "Anthropic · Weekly · all models   2 accounts · 12–67% remaining (1 last",
            self.historical_span_style((179, 179, 179), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(115, 13, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(116, 13, 1, 1),
            "┃",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(117, 13, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 13, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 14, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 14, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 14, 37, 1),
            "                                     ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(39, 14, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(40, 14, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(41, 14, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(42, 14, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(44, 14, 5, 1),
            "good)",
            self.historical_span_style((179, 179, 179), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(49, 14, 67, 1),
            "                                                                   ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(116, 14, 1, 1),
            "┃",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(117, 14, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 14, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 15, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 15, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 15, 37, 1),
            "                                     ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(39, 15, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(40, 15, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(41, 15, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(42, 15, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(44, 15, 70, 1),
            "Anthropic · Weekly · Opus   2 accounts · 0–46% remaining (1 last good)",
            self.historical_span_style((179, 179, 179), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(114, 15, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(116, 15, 1, 1),
            "┃",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(117, 15, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 15, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 16, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 16, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 16, 37, 1),
            "                                     ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(39, 16, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(40, 16, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(41, 16, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(42, 16, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(44, 16, 62, 1),
            "OpenAI · Credits   2 accounts · 76–82% remaining (1 last good)",
            self.historical_span_style((179, 179, 179), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(106, 16, 10, 1),
            "          ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(116, 16, 1, 1),
            "┃",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(117, 16, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 16, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 17, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 17, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 17, 37, 1),
            "                                     ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(39, 17, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(40, 17, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(41, 17, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(42, 17, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(44, 17, 71, 1),
            "OpenAI · Session · 5-hour   2 accounts · 88–96% remaining (1 last good)",
            self.historical_span_style((179, 179, 179), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(115, 17, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(116, 17, 1, 1),
            "┃",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(117, 17, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 17, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 18, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 18, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 18, 37, 1),
            "                                     ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(39, 18, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(40, 18, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(41, 18, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(42, 18, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(44, 18, 69, 1),
            "OpenAI · Weekly · 7-day   2 accounts · 41–88% remaining (1 last good)",
            self.historical_span_style((179, 179, 179), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(113, 18, 3, 1),
            "   ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(116, 18, 1, 1),
            "┃",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(117, 18, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 18, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 19, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 19, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 19, 37, 1),
            "                                     ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(39, 19, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(40, 19, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(41, 19, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(42, 19, 74, 1),
            "                                                                          ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(116, 19, 1, 1),
            "┃",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(117, 19, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 19, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 20, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 20, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 20, 37, 1),
            "                                     ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(39, 20, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(40, 20, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(41, 20, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(42, 20, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(44, 20, 14, 1),
            "Not comparable",
            self.historical_span_style((179, 179, 179), (17, 17, 17), true),
        );
        ui.paint_str(
            Rect::new(58, 20, 58, 1),
            "                                                          ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(116, 20, 1, 1),
            "┃",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(117, 20, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 20, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 21, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 21, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 21, 37, 1),
            "                                     ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(39, 21, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(40, 21, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(41, 21, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(42, 21, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(44, 21, 66, 1),
            "Amp (single account or different windows) · xAI (single account or",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(110, 21, 6, 1),
            "      ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(116, 21, 1, 1),
            "┃",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(117, 21, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 21, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 22, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 22, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 22, 37, 1),
            "                                     ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(39, 22, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(40, 22, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(41, 22, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(42, 22, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(44, 22, 65, 1),
            "different windows) · Z.AI (single account or different windows) ·",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(109, 22, 7, 1),
            "       ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(116, 22, 1, 1),
            "┃",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(117, 22, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 22, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 23, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 23, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 23, 37, 1),
            "                                     ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(39, 23, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(40, 23, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(41, 23, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(42, 23, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(44, 23, 64, 1),
            "MiniMax (single account or different windows) · OpenCode (single",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(108, 23, 8, 1),
            "        ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(116, 23, 1, 1),
            "┃",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(117, 23, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 23, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 24, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 24, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 24, 37, 1),
            "                                     ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(39, 24, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(40, 24, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(41, 24, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(42, 24, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(44, 24, 29, 1),
            "account or different windows)",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(73, 24, 43, 1),
            "                                           ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(116, 24, 1, 1),
            "┃",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(117, 24, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 24, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 25, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 25, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 25, 37, 1),
            "                                     ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(39, 25, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(40, 25, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(41, 25, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(42, 25, 74, 1),
            "                                                                          ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(116, 25, 1, 1),
            "┃",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(117, 25, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 25, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 26, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 26, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 26, 37, 1),
            "                                     ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(39, 26, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(40, 26, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(41, 26, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(42, 26, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(44, 26, 8, 1),
            "Warnings",
            self.historical_span_style((179, 179, 179), (17, 17, 17), true),
        );
        ui.paint_str(
            Rect::new(52, 26, 64, 1),
            "                                                                ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(116, 26, 1, 1),
            "┃",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(117, 26, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 26, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 27, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 27, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 27, 37, 1),
            "                                     ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(39, 27, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(40, 27, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(41, 27, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(42, 27, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(44, 27, 65, 1),
            "▲ Claude · Work   Session · 5-hour 76% used · resets in 1 h 5 min",
            self.historical_span_style((245, 158, 9), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(109, 27, 7, 1),
            "       ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(116, 27, 1, 1),
            "┃",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(117, 27, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 27, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 28, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 28, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 28, 37, 1),
            "                                     ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(39, 28, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(40, 28, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(41, 28, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(42, 28, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(44, 28, 65, 1),
            "▲ Claude · Work   Weekly · all models 88% used · resets Sun 19:14",
            self.historical_span_style((245, 158, 9), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(109, 28, 7, 1),
            "       ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(116, 28, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(117, 28, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 28, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 29, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 29, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 29, 37, 1),
            "                                     ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(39, 29, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(40, 29, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(41, 29, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(42, 29, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(44, 29, 60, 1),
            "! Claude · Work   Weekly · Opus exhausted · resets Sun 19:14",
            self.historical_span_style((228, 69, 69), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(104, 29, 12, 1),
            "            ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(116, 29, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(117, 29, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 29, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 30, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 30, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 30, 37, 1),
            "                                     ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(39, 30, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(40, 30, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(41, 30, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(42, 30, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(44, 30, 46, 1),
            "▲ Claude · Work   stale · last good 47 min ago",
            self.historical_span_style((245, 158, 9), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(90, 30, 26, 1),
            "                          ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(116, 30, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(117, 30, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 30, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 31, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 31, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 31, 37, 1),
            "                                     ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(39, 31, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(40, 31, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(41, 31, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(42, 31, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(44, 31, 63, 1),
            "▲ Amp · discovered   Amp Free · daily 91% used · resets in 10 h",
            self.historical_span_style((245, 158, 9), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(107, 31, 9, 1),
            "         ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(116, 31, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(117, 31, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 31, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 32, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 32, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 32, 37, 1),
            "                                     ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(39, 32, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(40, 32, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(41, 32, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(42, 32, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(44, 32, 56, 1),
            "▲ Z.AI · discovered   Weekly 76% used · resets Mon 09:14",
            self.historical_span_style((245, 158, 9), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(100, 32, 16, 1),
            "                ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(116, 32, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(117, 32, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 32, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 33, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 33, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 33, 37, 1),
            "                                     ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(39, 33, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(40, 33, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(41, 33, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(42, 33, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(44, 33, 47, 1),
            "▲ Z.AI · discovered   stale · last good 2 h ago",
            self.historical_span_style((199, 130, 11), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(91, 33, 25, 1),
            "                         ",
            self.historical_span_style((207, 207, 207), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(116, 33, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(117, 33, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 33, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 34, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 34, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 34, 37, 1),
            "                                     ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(39, 34, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(40, 34, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(41, 34, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(42, 34, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(44, 34, 65, 1),
            "! Kimi · discovered   No credential found: ~/.kimi has no api key",
            self.historical_span_style((133, 46, 46), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(109, 34, 7, 1),
            "       ",
            self.historical_span_style((148, 148, 148), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(116, 34, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(117, 34, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 34, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 35, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 35, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 35, 37, 1),
            "                                     ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(39, 35, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(40, 35, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(41, 35, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(42, 35, 77, 1),
            "                                                                             ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 35, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 36, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 36, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 36, 37, 1),
            "                                     ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(39, 36, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(40, 36, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(41, 36, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(42, 36, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(43, 36, 1, 1),
            " ",
            self.historical_span_style((39, 39, 42), (39, 39, 42), false),
        );
        ui.paint_str(
            Rect::new(44, 36, 12, 1),
            "Refresh all ",
            self.historical_span_style((255, 255, 255), (39, 39, 42), false),
        );
        ui.paint_str(
            Rect::new(56, 36, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(58, 36, 1, 1),
            " ",
            self.historical_span_style((39, 39, 42), (39, 39, 42), false),
        );
        ui.paint_str(
            Rect::new(59, 36, 13, 1),
            "Add account… ",
            self.historical_span_style((255, 255, 255), (39, 39, 42), false),
        );
        ui.paint_str(
            Rect::new(72, 36, 47, 1),
            "                                               ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 36, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 37, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 37, 39, 1),
            "╰─────────────────────────────────────╯",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(40, 37, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(41, 37, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(42, 37, 77, 1),
            "                                                                             ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 37, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(Rect::new(0, 38, 120, 1), "                                                                                                                        ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(
            Rect::new(0, 39, 26, 1),
            "                          ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(26, 39, 5, 1),
            "Enter",
            self.historical_span_style((255, 255, 255), (0, 0, 0), true),
        );
        ui.paint_str(
            Rect::new(31, 39, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(32, 39, 7, 1),
            "Details",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(39, 39, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(41, 39, 1, 1),
            "r",
            self.historical_span_style((255, 255, 255), (0, 0, 0), true),
        );
        ui.paint_str(
            Rect::new(42, 39, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(43, 39, 11, 1),
            "Refresh all",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(54, 39, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(56, 39, 1, 1),
            "a",
            self.historical_span_style((255, 255, 255), (0, 0, 0), true),
        );
        ui.paint_str(
            Rect::new(57, 39, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(58, 39, 4, 1),
            "Add…",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(62, 39, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(64, 39, 1, 1),
            "/",
            self.historical_span_style((255, 255, 255), (0, 0, 0), true),
        );
        ui.paint_str(
            Rect::new(65, 39, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(66, 39, 6, 1),
            "Filter",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(72, 39, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(74, 39, 1, 1),
            "m",
            self.historical_span_style((255, 255, 255), (0, 0, 0), true),
        );
        ui.paint_str(
            Rect::new(75, 39, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(76, 39, 5, 1),
            "Usage",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(81, 39, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(83, 39, 3, 1),
            "Esc",
            self.historical_span_style((255, 255, 255), (0, 0, 0), true),
        );
        ui.paint_str(
            Rect::new(86, 39, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(87, 39, 4, 1),
            "Back",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(91, 39, 29, 1),
            "                             ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
    }

    pub(super) fn draw_historical_accounts_detail_120_40(&self, ui: &mut Ui<'_>, area: Rect) {
        ui.fill(
            area,
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(0, 0, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 0, 9, 1),
            " jackin❯ ",
            self.historical_span_style((25, 25, 28), (72, 224, 84), true),
        );
        ui.paint_str(
            Rect::new(10, 0, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(12, 0, 6, 1),
            " File ",
            self.historical_span_style((179, 179, 179), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(18, 0, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(19, 0, 4, 1),
            " Go ",
            self.historical_span_style((179, 179, 179), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(23, 0, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(24, 0, 6, 1),
            " Help ",
            self.historical_span_style((179, 179, 179), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(30, 0, 16, 1),
            "                ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(46, 0, 24, 1),
            "Accounts › Claude › Work",
            self.historical_span_style((179, 179, 179), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(70, 0, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(72, 0, 14, 1),
            "⠋ refreshing 1",
            self.historical_span_style((179, 179, 179), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(86, 0, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(88, 0, 20, 1),
            "inside the Construct",
            self.historical_span_style((179, 179, 179), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(108, 0, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(110, 0, 9, 1),
            "2 running",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(119, 0, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(Rect::new(0, 1, 120, 1), "                                                                                                                        ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(
            Rect::new(0, 2, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 2, 2, 1),
            "╭─",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(3, 2, 10, 1),
            " Accounts ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), true),
        );
        ui.paint_str(
            Rect::new(13, 2, 27, 1),
            "───────── 12 · 4 ▲ · 4 ! ─╮",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(40, 2, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(41, 2, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(42, 2, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(44, 2, 13, 1),
            "Claude · Work",
            self.historical_span_style((179, 179, 179), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(57, 2, 41, 1),
            "                                         ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(98, 2, 19, 1),
            "account · exhausted",
            self.historical_span_style((77, 77, 77), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(117, 2, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 2, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 3, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 3, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 3, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(3, 3, 1, 1),
            " ",
            self.historical_span_style((0, 0, 0), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(4, 3, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(5, 3, 1, 1),
            " ",
            self.historical_span_style((179, 179, 179), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(6, 3, 33, 1),
            " Overview                        ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(39, 3, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(40, 3, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(41, 3, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(42, 3, 77, 1),
            "                                                                             ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 3, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 4, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 4, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 4, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(3, 4, 1, 1),
            " ",
            self.historical_span_style((0, 0, 0), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(4, 4, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(5, 4, 1, 1),
            "▾",
            self.historical_span_style((179, 179, 179), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(6, 4, 29, 1),
            " Claude                      ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(35, 4, 1, 1),
            "!",
            self.historical_span_style((228, 69, 69), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(36, 4, 3, 1),
            "   ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(39, 4, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(40, 4, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(41, 4, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(42, 4, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(44, 4, 8, 1),
            "Provider",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(52, 4, 67, 1),
            "            Anthropic / Claude                                     ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 4, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 5, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 5, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 5, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(3, 5, 1, 1),
            " ",
            self.historical_span_style((0, 0, 0), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(4, 5, 3, 1),
            "   ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(7, 5, 1, 1),
            "★",
            self.historical_span_style((179, 179, 179), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(8, 5, 31, 1),
            " Personal                      ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(39, 5, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(40, 5, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(41, 5, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(42, 5, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(44, 5, 13, 1),
            "Agent runtime",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(57, 5, 62, 1),
            "       Claude Code                                            ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 5, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 6, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 6, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 6, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(3, 6, 1, 1),
            " ",
            self.historical_span_style((0, 0, 0), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(4, 6, 3, 1),
            "   ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(7, 6, 1, 1),
            " ",
            self.historical_span_style((179, 179, 179), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(8, 6, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(9, 6, 28, 1),
            "Archived contractor laptop …",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(37, 6, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(39, 6, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(40, 6, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(41, 6, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(42, 6, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(44, 6, 13, 1),
            "Usage surface",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(57, 6, 62, 1),
            "       Claude                                                 ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 6, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 7, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 7, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 7, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(3, 7, 1, 1),
            "▎",
            self.historical_span_style((72, 224, 84), (15, 46, 19), true),
        );
        ui.paint_str(
            Rect::new(4, 7, 3, 1),
            "   ",
            self.historical_span_style((255, 255, 255), (15, 46, 19), true),
        );
        ui.paint_str(
            Rect::new(7, 7, 1, 1),
            " ",
            self.historical_span_style((179, 179, 179), (15, 46, 19), false),
        );
        ui.paint_str(
            Rect::new(8, 7, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (15, 46, 19), true),
        );
        ui.paint_str(
            Rect::new(9, 7, 26, 1),
            "Work                      ",
            self.historical_span_style((72, 224, 84), (15, 46, 19), true),
        );
        ui.paint_str(
            Rect::new(35, 7, 1, 1),
            "!",
            self.historical_span_style((228, 69, 69), (15, 46, 19), true),
        );
        ui.paint_str(
            Rect::new(36, 7, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (15, 46, 19), true),
        );
        ui.paint_str(
            Rect::new(38, 7, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(39, 7, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(40, 7, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(41, 7, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(42, 7, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(44, 7, 8, 1),
            "Identity",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(52, 7, 67, 1),
            "            alexey@chainargos.com                                  ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 7, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 8, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 8, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 8, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(3, 8, 1, 1),
            " ",
            self.historical_span_style((0, 0, 0), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(4, 8, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(5, 8, 1, 1),
            "▾",
            self.historical_span_style((179, 179, 179), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(6, 8, 33, 1),
            " Codex                           ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(39, 8, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(40, 8, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(41, 8, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(42, 8, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(44, 8, 4, 1),
            "Plan",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(48, 8, 71, 1),
            "                Team                                                   ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 8, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 9, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 9, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 9, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(3, 9, 1, 1),
            " ",
            self.historical_span_style((0, 0, 0), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(4, 9, 3, 1),
            "   ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(7, 9, 1, 1),
            "★",
            self.historical_span_style((179, 179, 179), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(8, 9, 31, 1),
            " Primary                       ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(39, 9, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(40, 9, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(41, 9, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(42, 9, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(44, 9, 10, 1),
            "Credential",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(54, 9, 65, 1),
            "          1Password · Engineering › Anthropic · Work ›           ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 9, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 10, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 10, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 10, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(3, 10, 1, 1),
            " ",
            self.historical_span_style((0, 0, 0), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(4, 10, 3, 1),
            "   ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(7, 10, 1, 1),
            " ",
            self.historical_span_style((179, 179, 179), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(8, 10, 31, 1),
            " Experiments                   ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(39, 10, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(40, 10, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(41, 10, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(42, 10, 77, 1),
            "                      credential                                             ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 10, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 11, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 11, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 11, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(3, 11, 1, 1),
            " ",
            self.historical_span_style((0, 0, 0), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(4, 11, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(5, 11, 1, 1),
            "▾",
            self.historical_span_style((179, 179, 179), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(6, 11, 29, 1),
            " Amp                         ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(35, 11, 1, 1),
            "▲",
            self.historical_span_style((245, 158, 9), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(36, 11, 3, 1),
            "   ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(39, 11, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(40, 11, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(41, 11, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(42, 11, 22, 1),
            "                      ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(64, 11, 26, 1),
            "chainargos.1password.com ·",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(90, 11, 29, 1),
            "                             ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 11, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 12, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 12, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 12, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(3, 12, 1, 1),
            " ",
            self.historical_span_style((0, 0, 0), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(4, 12, 3, 1),
            "   ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(7, 12, 1, 1),
            " ",
            self.historical_span_style((179, 179, 179), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(8, 12, 27, 1),
            " discovered                ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(35, 12, 1, 1),
            "▲",
            self.historical_span_style((245, 158, 9), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(36, 12, 3, 1),
            "   ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(39, 12, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(40, 12, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(41, 12, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(42, 12, 22, 1),
            "                      ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(64, 12, 48, 1),
            "op://v_eng01/it_ant01/credential · ••••••••…3c9e",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(112, 12, 7, 1),
            "       ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 12, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 13, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 13, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 13, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(3, 13, 1, 1),
            " ",
            self.historical_span_style((0, 0, 0), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(4, 13, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(5, 13, 1, 1),
            "▾",
            self.historical_span_style((179, 179, 179), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(6, 13, 33, 1),
            " Grok                            ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(39, 13, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(40, 13, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(41, 13, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(42, 13, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(44, 13, 10, 1),
            "Provenance",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(54, 13, 10, 1),
            "          ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(64, 13, 46, 1),
            "configured source · confidence authoritative ·",
            self.historical_span_style((179, 179, 179), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(110, 13, 9, 1),
            "         ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 13, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 14, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 14, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 14, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(3, 14, 1, 1),
            " ",
            self.historical_span_style((0, 0, 0), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(4, 14, 3, 1),
            "   ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(7, 14, 1, 1),
            "★",
            self.historical_span_style((179, 179, 179), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(8, 14, 31, 1),
            " Team                          ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(39, 14, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(40, 14, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(41, 14, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(42, 14, 22, 1),
            "                      ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(64, 14, 10, 1),
            "registered",
            self.historical_span_style((179, 179, 179), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(74, 14, 45, 1),
            "                                             ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 14, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 15, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 15, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 15, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(3, 15, 1, 1),
            " ",
            self.historical_span_style((0, 0, 0), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(4, 15, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(5, 15, 1, 1),
            "▾",
            self.historical_span_style((179, 179, 179), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(6, 15, 29, 1),
            " Z.AI                        ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(35, 15, 1, 1),
            "▲",
            self.historical_span_style((245, 158, 9), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(36, 15, 3, 1),
            "   ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(39, 15, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(40, 15, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(41, 15, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(42, 15, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(44, 15, 9, 1),
            "Lifecycle",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(53, 15, 66, 1),
            "           available                                              ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 15, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 16, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 16, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 16, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(3, 16, 1, 1),
            " ",
            self.historical_span_style((0, 0, 0), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(4, 16, 3, 1),
            "   ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(7, 16, 1, 1),
            " ",
            self.historical_span_style((179, 179, 179), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(8, 16, 27, 1),
            " discovered                ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(35, 16, 1, 1),
            "▲",
            self.historical_span_style((245, 158, 9), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(36, 16, 3, 1),
            "   ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(39, 16, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(40, 16, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(41, 16, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(42, 16, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(44, 16, 7, 1),
            "Default",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(51, 16, 68, 1),
            "             no                                                     ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 16, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 17, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 17, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 17, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(3, 17, 1, 1),
            " ",
            self.historical_span_style((0, 0, 0), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(4, 17, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(5, 17, 1, 1),
            "▾",
            self.historical_span_style((179, 179, 179), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(6, 17, 29, 1),
            " Kimi                        ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(35, 17, 1, 1),
            "!",
            self.historical_span_style((228, 69, 69), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(36, 17, 3, 1),
            "   ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(39, 17, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(40, 17, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(41, 17, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(42, 17, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(44, 17, 7, 1),
            "Enabled",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(51, 17, 68, 1),
            "             yes                                                    ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 17, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 18, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 18, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 18, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(3, 18, 1, 1),
            " ",
            self.historical_span_style((0, 0, 0), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(4, 18, 3, 1),
            "   ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(7, 18, 1, 1),
            " ",
            self.historical_span_style((179, 179, 179), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(8, 18, 27, 1),
            " discovered                ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(35, 18, 1, 1),
            "!",
            self.historical_span_style((228, 69, 69), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(36, 18, 3, 1),
            "   ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(39, 18, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(40, 18, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(41, 18, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(42, 18, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(44, 18, 7, 1),
            "Purpose",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(51, 18, 68, 1),
            "             work                                                   ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 18, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 19, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 19, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 19, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(3, 19, 1, 1),
            " ",
            self.historical_span_style((0, 0, 0), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(4, 19, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(5, 19, 1, 1),
            "▾",
            self.historical_span_style((179, 179, 179), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(6, 19, 29, 1),
            " MiniMax                     ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(35, 19, 1, 1),
            "!",
            self.historical_span_style((228, 69, 69), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(36, 19, 3, 1),
            "   ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(39, 19, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(40, 19, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(41, 19, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(42, 19, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(44, 19, 7, 1),
            "Used by",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(51, 19, 13, 1),
            "             ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(64, 19, 37, 1),
            "Workspace choice in payments-platform",
            self.historical_span_style((179, 179, 179), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(101, 19, 18, 1),
            "                  ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 19, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 20, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 20, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 20, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(3, 20, 1, 1),
            " ",
            self.historical_span_style((0, 0, 0), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(4, 20, 3, 1),
            "   ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(7, 20, 1, 1),
            " ",
            self.historical_span_style((179, 179, 179), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(8, 20, 27, 1),
            " discovered                ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(35, 20, 1, 1),
            "!",
            self.historical_span_style((228, 69, 69), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(36, 20, 3, 1),
            "   ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(39, 20, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(40, 20, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(41, 20, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(42, 20, 77, 1),
            "                                                                             ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 20, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 21, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 21, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 21, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(3, 21, 1, 1),
            " ",
            self.historical_span_style((0, 0, 0), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(4, 21, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(5, 21, 1, 1),
            "▾",
            self.historical_span_style((179, 179, 179), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(6, 21, 29, 1),
            " OpenCode                    ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(35, 21, 1, 1),
            "!",
            self.historical_span_style((228, 69, 69), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(36, 21, 3, 1),
            "   ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(39, 21, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(40, 21, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(41, 21, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(42, 21, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(44, 21, 5, 1),
            "Quota",
            self.historical_span_style((179, 179, 179), (17, 17, 17), true),
        );
        ui.paint_str(
            Rect::new(49, 21, 40, 1),
            "                                        ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(89, 21, 28, 1),
            "stale · last good 47 min ago",
            self.historical_span_style((77, 77, 77), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(117, 21, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 21, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 22, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 22, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 22, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(3, 22, 1, 1),
            " ",
            self.historical_span_style((0, 0, 0), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(4, 22, 3, 1),
            "   ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(7, 22, 1, 1),
            "★",
            self.historical_span_style((179, 179, 179), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(8, 22, 27, 1),
            " Go subscription           ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(35, 22, 1, 1),
            "!",
            self.historical_span_style((228, 69, 69), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(36, 22, 3, 1),
            "   ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(39, 22, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(40, 22, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(41, 22, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(42, 22, 22, 1),
            "  Session · 5-hour    ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(64, 22, 26, 1),
            "  76%                     ",
            self.historical_span_style((179, 179, 179), (77, 77, 77), true),
        );
        ui.paint_str(
            Rect::new(90, 22, 8, 1),
            "        ",
            self.historical_span_style((128, 128, 128), (39, 39, 42), false),
        );
        ui.paint_str(
            Rect::new(98, 22, 2, 1),
            "  ",
            self.historical_span_style((77, 77, 77), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(100, 22, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(102, 22, 15, 1),
            "resets in 1 h …",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(117, 22, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 22, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 23, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 23, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 23, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(3, 23, 1, 1),
            " ",
            self.historical_span_style((0, 0, 0), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(4, 23, 3, 1),
            "   ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(7, 23, 1, 1),
            " ",
            self.historical_span_style((179, 179, 179), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(8, 23, 31, 1),
            " ci-bot · discovered           ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(39, 23, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(40, 23, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(41, 23, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(42, 23, 22, 1),
            "  Weekly · all models ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(64, 23, 30, 1),
            "  88%                         ",
            self.historical_span_style((179, 179, 179), (77, 77, 77), true),
        );
        ui.paint_str(
            Rect::new(94, 23, 4, 1),
            "    ",
            self.historical_span_style((128, 128, 128), (39, 39, 42), false),
        );
        ui.paint_str(
            Rect::new(98, 23, 2, 1),
            "  ",
            self.historical_span_style((77, 77, 77), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(100, 23, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(102, 23, 15, 1),
            "resets Sun 19:…",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(117, 23, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 23, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 24, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 24, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 24, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(3, 24, 1, 1),
            " ",
            self.historical_span_style((0, 0, 0), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(4, 24, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(5, 24, 1, 1),
            " ",
            self.historical_span_style((179, 179, 179), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(6, 24, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(7, 24, 30, 1),
            "Unsupported                   ",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(37, 24, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(39, 24, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(40, 24, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(41, 24, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(42, 24, 22, 1),
            "  Weekly · Opus       ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(64, 24, 34, 1),
            " 100%                             ",
            self.historical_span_style((179, 179, 179), (77, 77, 77), true),
        );
        ui.paint_str(
            Rect::new(98, 24, 2, 1),
            "  ",
            self.historical_span_style((77, 77, 77), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(100, 24, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(102, 24, 15, 1),
            "resets Sun 19:…",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(117, 24, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 24, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 25, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 25, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 25, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(3, 25, 1, 1),
            " ",
            self.historical_span_style((0, 0, 0), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(4, 25, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(5, 25, 1, 1),
            " ",
            self.historical_span_style((179, 179, 179), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(6, 25, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(7, 25, 30, 1),
            "+ Add account…                ",
            self.historical_span_style((179, 179, 179), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(37, 25, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(39, 25, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(40, 25, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(41, 25, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(42, 25, 22, 1),
            "  Extra usage credits ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(64, 25, 10, 1),
            "  28%     ",
            self.historical_span_style((179, 179, 179), (77, 77, 77), true),
        );
        ui.paint_str(
            Rect::new(74, 25, 24, 1),
            "                        ",
            self.historical_span_style((128, 128, 128), (39, 39, 42), false),
        );
        ui.paint_str(
            Rect::new(98, 25, 2, 1),
            "  ",
            self.historical_span_style((77, 77, 77), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(100, 25, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(102, 25, 15, 1),
            "1,420 / 5,000 …",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(117, 25, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 25, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 26, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 26, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 26, 37, 1),
            "                                     ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(39, 26, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(40, 26, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(41, 26, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(42, 26, 77, 1),
            "                                                                             ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 26, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 27, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 27, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 27, 37, 1),
            "                                     ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(39, 27, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(40, 27, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(41, 27, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(42, 27, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(44, 27, 10, 1),
            "Validation",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(54, 27, 65, 1),
            "          ✓ material   ✓ identity   ✓ quota access · quota       ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 27, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 28, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 28, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 28, 37, 1),
            "                                     ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(39, 28, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(40, 28, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(41, 28, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(42, 28, 77, 1),
            "                      readable                                               ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 28, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 29, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 29, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 29, 37, 1),
            "                                     ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(39, 29, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(40, 29, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(41, 29, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(42, 29, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(44, 29, 6, 1),
            "Status",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(50, 29, 14, 1),
            "              ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(64, 29, 53, 1),
            "Usage stale · last good 47 min ago · retry  in 13 min",
            self.historical_span_style((245, 158, 9), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(117, 29, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 29, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 30, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 30, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 30, 37, 1),
            "                                     ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(39, 30, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(40, 30, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(41, 30, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(42, 30, 22, 1),
            "                      ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(64, 30, 11, 1),
            "· retryable",
            self.historical_span_style((245, 158, 9), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(75, 30, 44, 1),
            "                                            ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 30, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 31, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 31, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 31, 37, 1),
            "                                     ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(39, 31, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(40, 31, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(41, 31, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(42, 31, 77, 1),
            "                                                                             ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 31, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 32, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 32, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 32, 37, 1),
            "                                     ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(39, 32, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(40, 32, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(41, 32, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(42, 32, 77, 1),
            "                                                                             ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 32, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 33, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 33, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 33, 37, 1),
            "                                     ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(39, 33, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(40, 33, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(41, 33, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(42, 33, 77, 1),
            "                                                                             ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 33, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 34, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 34, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 34, 37, 1),
            "                                     ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(39, 34, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(40, 34, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(41, 34, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(42, 34, 77, 1),
            "                                                                             ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 34, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 35, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 35, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 35, 37, 1),
            "                                     ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(39, 35, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(40, 35, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(41, 35, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(42, 35, 77, 1),
            "                                                                             ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 35, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 36, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 36, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 36, 37, 1),
            "                                     ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(39, 36, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(40, 36, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(41, 36, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(42, 36, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(43, 36, 1, 1),
            " ",
            self.historical_span_style((39, 39, 42), (39, 39, 42), false),
        );
        ui.paint_str(
            Rect::new(44, 36, 8, 1),
            "Refresh ",
            self.historical_span_style((255, 255, 255), (39, 39, 42), false),
        );
        ui.paint_str(
            Rect::new(52, 36, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(54, 36, 1, 1),
            " ",
            self.historical_span_style((39, 39, 42), (39, 39, 42), false),
        );
        ui.paint_str(
            Rect::new(55, 36, 9, 1),
            "Validate ",
            self.historical_span_style((255, 255, 255), (39, 39, 42), false),
        );
        ui.paint_str(
            Rect::new(64, 36, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(66, 36, 1, 1),
            " ",
            self.historical_span_style((39, 39, 42), (39, 39, 42), false),
        );
        ui.paint_str(
            Rect::new(67, 36, 6, 1),
            "Edit… ",
            self.historical_span_style((255, 255, 255), (39, 39, 42), false),
        );
        ui.paint_str(
            Rect::new(73, 36, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(75, 36, 1, 1),
            " ",
            self.historical_span_style((39, 39, 42), (39, 39, 42), false),
        );
        ui.paint_str(
            Rect::new(76, 36, 12, 1),
            "Set default ",
            self.historical_span_style((255, 255, 255), (39, 39, 42), false),
        );
        ui.paint_str(
            Rect::new(88, 36, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(90, 36, 1, 1),
            " ",
            self.historical_span_style((39, 39, 42), (39, 39, 42), false),
        );
        ui.paint_str(
            Rect::new(91, 36, 8, 1),
            "Disable ",
            self.historical_span_style((255, 255, 255), (39, 39, 42), false),
        );
        ui.paint_str(
            Rect::new(99, 36, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(101, 36, 1, 1),
            " ",
            self.historical_span_style((39, 39, 42), (39, 39, 42), false),
        );
        ui.paint_str(
            Rect::new(102, 36, 8, 1),
            "Remove… ",
            self.historical_span_style((228, 69, 69), (39, 39, 42), false),
        );
        ui.paint_str(
            Rect::new(110, 36, 9, 1),
            "         ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 36, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 37, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 37, 39, 1),
            "╰─────────────────────────────────────╯",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(40, 37, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(41, 37, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(42, 37, 77, 1),
            "                                                                             ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 37, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(Rect::new(0, 38, 120, 1), "                                                                                                                        ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(
            Rect::new(0, 39, 3, 1),
            "   ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(3, 39, 5, 1),
            "Enter",
            self.historical_span_style((255, 255, 255), (0, 0, 0), true),
        );
        ui.paint_str(
            Rect::new(8, 39, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(9, 39, 7, 1),
            "Details",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(16, 39, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(18, 39, 1, 1),
            "r",
            self.historical_span_style((255, 255, 255), (0, 0, 0), true),
        );
        ui.paint_str(
            Rect::new(19, 39, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(20, 39, 7, 1),
            "Refresh",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(27, 39, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(29, 39, 1, 1),
            "e",
            self.historical_span_style((255, 255, 255), (0, 0, 0), true),
        );
        ui.paint_str(
            Rect::new(30, 39, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(31, 39, 5, 1),
            "Edit…",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(36, 39, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(38, 39, 5, 1),
            "Space",
            self.historical_span_style((255, 255, 255), (0, 0, 0), true),
        );
        ui.paint_str(
            Rect::new(43, 39, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(44, 39, 7, 1),
            "Default",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(51, 39, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(53, 39, 1, 1),
            "d",
            self.historical_span_style((255, 255, 255), (0, 0, 0), true),
        );
        ui.paint_str(
            Rect::new(54, 39, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(55, 39, 7, 1),
            "Disable",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(62, 39, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(64, 39, 1, 1),
            "v",
            self.historical_span_style((255, 255, 255), (0, 0, 0), true),
        );
        ui.paint_str(
            Rect::new(65, 39, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(66, 39, 8, 1),
            "Validate",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(74, 39, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(76, 39, 1, 1),
            "x",
            self.historical_span_style((255, 255, 255), (0, 0, 0), true),
        );
        ui.paint_str(
            Rect::new(77, 39, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(78, 39, 7, 1),
            "Remove…",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(85, 39, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(87, 39, 1, 1),
            "a",
            self.historical_span_style((255, 255, 255), (0, 0, 0), true),
        );
        ui.paint_str(
            Rect::new(88, 39, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(89, 39, 4, 1),
            "Add…",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(93, 39, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(95, 39, 1, 1),
            "/",
            self.historical_span_style((255, 255, 255), (0, 0, 0), true),
        );
        ui.paint_str(
            Rect::new(96, 39, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(97, 39, 6, 1),
            "Filter",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(103, 39, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(105, 39, 1, 1),
            "m",
            self.historical_span_style((255, 255, 255), (0, 0, 0), true),
        );
        ui.paint_str(
            Rect::new(106, 39, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(107, 39, 5, 1),
            "Usage",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(112, 39, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(114, 39, 1, 1),
            "…",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(115, 39, 5, 1),
            "     ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
    }

    pub(super) fn draw_historical_accounts_drawer_120_40(&self, ui: &mut Ui<'_>, area: Rect) {
        ui.fill(
            area,
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(0, 0, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 0, 9, 1),
            " jackin❯ ",
            self.historical_span_style((25, 25, 28), (72, 224, 84), true),
        );
        ui.paint_str(
            Rect::new(10, 0, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(12, 0, 6, 1),
            " File ",
            self.historical_span_style((179, 179, 179), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(18, 0, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(19, 0, 4, 1),
            " Go ",
            self.historical_span_style((179, 179, 179), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(23, 0, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(24, 0, 6, 1),
            " Help ",
            self.historical_span_style((179, 179, 179), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(30, 0, 16, 1),
            "                ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(46, 0, 24, 1),
            "Accounts › Claude › Work",
            self.historical_span_style((179, 179, 179), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(70, 0, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(72, 0, 14, 1),
            "⠋ refreshing 1",
            self.historical_span_style((179, 179, 179), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(86, 0, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(88, 0, 20, 1),
            "inside the Construct",
            self.historical_span_style((179, 179, 179), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(108, 0, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(110, 0, 9, 1),
            "2 running",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(119, 0, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(Rect::new(0, 1, 120, 1), "                                                                                                                        ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(
            Rect::new(0, 2, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 2, 2, 1),
            "╭─",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(3, 2, 10, 1),
            " Accounts ",
            self.historical_span_style((179, 179, 179), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(13, 2, 9, 1),
            "─────────",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(22, 2, 16, 1),
            " 12 · 4 ▲ · 4 ! ",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(38, 2, 2, 1),
            "─╮",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(40, 2, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(41, 2, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(42, 2, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(43, 2, 1, 1),
            "▎",
            self.historical_span_style((72, 224, 84), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(44, 2, 13, 1),
            "Claude · Work",
            self.historical_span_style((255, 255, 255), (17, 17, 17), true),
        );
        ui.paint_str(
            Rect::new(57, 2, 41, 1),
            "                                         ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(98, 2, 19, 1),
            "account · exhausted",
            self.historical_span_style((77, 77, 77), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(117, 2, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 2, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 3, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 3, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 3, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(3, 3, 1, 1),
            " ",
            self.historical_span_style((0, 0, 0), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(4, 3, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(5, 3, 1, 1),
            " ",
            self.historical_span_style((179, 179, 179), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(6, 3, 33, 1),
            " Overview                        ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(39, 3, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(40, 3, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(41, 3, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(42, 3, 77, 1),
            "                                                                             ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 3, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 4, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 4, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 4, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(3, 4, 1, 1),
            " ",
            self.historical_span_style((0, 0, 0), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(4, 4, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(5, 4, 1, 1),
            "▾",
            self.historical_span_style((179, 179, 179), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(6, 4, 29, 1),
            " Claude                      ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(35, 4, 1, 1),
            "!",
            self.historical_span_style((228, 69, 69), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(36, 4, 3, 1),
            "   ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(39, 4, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(40, 4, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(41, 4, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(42, 4, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(44, 4, 8, 1),
            "Provider",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(52, 4, 67, 1),
            "            Anthropic / Claude                                     ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 4, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 5, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 5, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 5, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(3, 5, 1, 1),
            " ",
            self.historical_span_style((0, 0, 0), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(4, 5, 3, 1),
            "   ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(7, 5, 1, 1),
            "★",
            self.historical_span_style((179, 179, 179), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(8, 5, 31, 1),
            " Personal                      ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(39, 5, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(40, 5, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(41, 5, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(42, 5, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(44, 5, 13, 1),
            "Agent runtime",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(57, 5, 62, 1),
            "       Claude Code                                            ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 5, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 6, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 6, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 6, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(3, 6, 1, 1),
            " ",
            self.historical_span_style((0, 0, 0), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(4, 6, 3, 1),
            "   ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(7, 6, 1, 1),
            " ",
            self.historical_span_style((179, 179, 179), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(8, 6, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(9, 6, 28, 1),
            "Archived contractor laptop …",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(37, 6, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(39, 6, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(40, 6, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(41, 6, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(42, 6, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(44, 6, 13, 1),
            "Usage surface",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(57, 6, 62, 1),
            "       Claude                                                 ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 6, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 7, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 7, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 7, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(3, 7, 1, 1),
            " ",
            self.historical_span_style((0, 0, 0), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(4, 7, 3, 1),
            "   ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(7, 7, 1, 1),
            " ",
            self.historical_span_style((179, 179, 179), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(8, 7, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(9, 7, 26, 1),
            "Work                      ",
            self.historical_span_style((72, 224, 84), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(35, 7, 1, 1),
            "!",
            self.historical_span_style((228, 69, 69), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(36, 7, 3, 1),
            "   ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(39, 7, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(40, 7, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(41, 7, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(42, 7, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(44, 7, 8, 1),
            "Identity",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(52, 7, 67, 1),
            "            alexey@chainargos.com                                  ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 7, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 8, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 8, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 8, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(3, 8, 1, 1),
            " ",
            self.historical_span_style((0, 0, 0), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(4, 8, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(5, 8, 1, 1),
            "▾",
            self.historical_span_style((179, 179, 179), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(6, 8, 33, 1),
            " Codex                           ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(39, 8, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(40, 8, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(41, 8, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(42, 8, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(44, 8, 4, 1),
            "Plan",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(48, 8, 71, 1),
            "                Team                                                   ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 8, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 9, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 9, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 9, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(3, 9, 1, 1),
            " ",
            self.historical_span_style((0, 0, 0), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(4, 9, 3, 1),
            "   ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(7, 9, 1, 1),
            "★",
            self.historical_span_style((179, 179, 179), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(8, 9, 31, 1),
            " Primary                       ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(39, 9, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(40, 9, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(41, 9, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(42, 9, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(44, 9, 10, 1),
            "Credential",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(54, 9, 65, 1),
            "          1Password · Engineering › Anthropic · Work ›           ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 9, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 10, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 10, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 10, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(3, 10, 1, 1),
            " ",
            self.historical_span_style((0, 0, 0), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(4, 10, 3, 1),
            "   ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(7, 10, 1, 1),
            " ",
            self.historical_span_style((179, 179, 179), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(8, 10, 31, 1),
            " Experiments                   ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(39, 10, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(40, 10, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(41, 10, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(42, 10, 77, 1),
            "                      credential                                             ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 10, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 11, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 11, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 11, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(3, 11, 1, 1),
            " ",
            self.historical_span_style((0, 0, 0), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(4, 11, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(5, 11, 1, 1),
            "▾",
            self.historical_span_style((179, 179, 179), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(6, 11, 29, 1),
            " Amp                         ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(35, 11, 1, 1),
            "▲",
            self.historical_span_style((245, 158, 9), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(36, 11, 3, 1),
            "   ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(39, 11, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(40, 11, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(41, 11, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(42, 11, 22, 1),
            "                      ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(64, 11, 26, 1),
            "chainargos.1password.com ·",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(90, 11, 29, 1),
            "                             ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 11, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 12, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 12, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 12, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(3, 12, 1, 1),
            " ",
            self.historical_span_style((0, 0, 0), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(4, 12, 3, 1),
            "   ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(7, 12, 1, 1),
            " ",
            self.historical_span_style((179, 179, 179), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(8, 12, 27, 1),
            " discovered                ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(35, 12, 1, 1),
            "▲",
            self.historical_span_style((245, 158, 9), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(36, 12, 3, 1),
            "   ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(39, 12, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(40, 12, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(41, 12, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(42, 12, 22, 1),
            "                      ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(64, 12, 48, 1),
            "op://v_eng01/it_ant01/credential · ••••••••…3c9e",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(112, 12, 7, 1),
            "       ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 12, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 13, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 13, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 13, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(3, 13, 1, 1),
            " ",
            self.historical_span_style((0, 0, 0), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(4, 13, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(5, 13, 1, 1),
            "▾",
            self.historical_span_style((179, 179, 179), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(6, 13, 33, 1),
            " Grok                            ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(39, 13, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(40, 13, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(41, 13, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(42, 13, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(44, 13, 10, 1),
            "Provenance",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(54, 13, 10, 1),
            "          ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(64, 13, 46, 1),
            "configured source · confidence authoritative ·",
            self.historical_span_style((179, 179, 179), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(110, 13, 9, 1),
            "         ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 13, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 14, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 14, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 14, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(3, 14, 1, 1),
            " ",
            self.historical_span_style((0, 0, 0), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(4, 14, 3, 1),
            "   ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(7, 14, 1, 1),
            "★",
            self.historical_span_style((179, 179, 179), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(8, 14, 31, 1),
            " Team                          ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(39, 14, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(40, 14, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(41, 14, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(42, 14, 22, 1),
            "                      ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(64, 14, 10, 1),
            "registered",
            self.historical_span_style((179, 179, 179), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(74, 14, 45, 1),
            "                                             ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 14, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 15, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 15, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 15, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(3, 15, 1, 1),
            " ",
            self.historical_span_style((0, 0, 0), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(4, 15, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(5, 15, 1, 1),
            "▾",
            self.historical_span_style((179, 179, 179), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(6, 15, 29, 1),
            " Z.AI                        ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(35, 15, 1, 1),
            "▲",
            self.historical_span_style((245, 158, 9), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(36, 15, 3, 1),
            "   ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(39, 15, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(40, 15, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(41, 15, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(42, 15, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(44, 15, 9, 1),
            "Lifecycle",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(53, 15, 66, 1),
            "           available                                              ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 15, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 16, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 16, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 16, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(3, 16, 1, 1),
            " ",
            self.historical_span_style((0, 0, 0), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(4, 16, 3, 1),
            "   ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(7, 16, 1, 1),
            " ",
            self.historical_span_style((179, 179, 179), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(8, 16, 27, 1),
            " discovered                ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(35, 16, 1, 1),
            "▲",
            self.historical_span_style((245, 158, 9), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(36, 16, 3, 1),
            "   ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(39, 16, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(40, 16, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(41, 16, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(42, 16, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(44, 16, 7, 1),
            "Default",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(51, 16, 68, 1),
            "             no                                                     ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 16, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 17, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 17, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 17, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(3, 17, 1, 1),
            " ",
            self.historical_span_style((0, 0, 0), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(4, 17, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(5, 17, 1, 1),
            "▾",
            self.historical_span_style((179, 179, 179), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(6, 17, 29, 1),
            " Kimi                        ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(35, 17, 1, 1),
            "!",
            self.historical_span_style((228, 69, 69), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(36, 17, 3, 1),
            "   ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(39, 17, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(40, 17, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(41, 17, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(42, 17, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(44, 17, 7, 1),
            "Enabled",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(51, 17, 68, 1),
            "             yes                                                    ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 17, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 18, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 18, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 18, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(3, 18, 1, 1),
            " ",
            self.historical_span_style((0, 0, 0), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(4, 18, 3, 1),
            "   ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(7, 18, 1, 1),
            " ",
            self.historical_span_style((179, 179, 179), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(8, 18, 27, 1),
            " discovered                ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(35, 18, 1, 1),
            "!",
            self.historical_span_style((228, 69, 69), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(36, 18, 3, 1),
            "   ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(39, 18, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(40, 18, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(41, 18, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(42, 18, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(44, 18, 7, 1),
            "Purpose",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(51, 18, 68, 1),
            "             work                                                   ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 18, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 19, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 19, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 19, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(3, 19, 1, 1),
            " ",
            self.historical_span_style((0, 0, 0), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(4, 19, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(5, 19, 1, 1),
            "▾",
            self.historical_span_style((179, 179, 179), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(6, 19, 29, 1),
            " MiniMax                     ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(35, 19, 1, 1),
            "!",
            self.historical_span_style((228, 69, 69), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(36, 19, 3, 1),
            "   ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(39, 19, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(40, 19, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(41, 19, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(42, 19, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(44, 19, 7, 1),
            "Used by",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(51, 19, 13, 1),
            "             ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(64, 19, 37, 1),
            "Workspace choice in payments-platform",
            self.historical_span_style((179, 179, 179), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(101, 19, 18, 1),
            "                  ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 19, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 20, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 20, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 20, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(3, 20, 1, 1),
            " ",
            self.historical_span_style((0, 0, 0), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(4, 20, 3, 1),
            "   ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(7, 20, 1, 1),
            " ",
            self.historical_span_style((179, 179, 179), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(8, 20, 27, 1),
            " discovered                ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(35, 20, 1, 1),
            "!",
            self.historical_span_style((228, 69, 69), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(36, 20, 3, 1),
            "   ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(39, 20, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(40, 20, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(41, 20, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(42, 20, 77, 1),
            "                                                                             ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 20, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 21, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 21, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 21, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(3, 21, 1, 1),
            " ",
            self.historical_span_style((0, 0, 0), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(4, 21, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(5, 21, 1, 1),
            "▾",
            self.historical_span_style((179, 179, 179), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(6, 21, 29, 1),
            " OpenCode                    ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(35, 21, 1, 1),
            "!",
            self.historical_span_style((228, 69, 69), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(36, 21, 3, 1),
            "   ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(39, 21, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(40, 21, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(41, 21, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(42, 21, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(44, 21, 5, 1),
            "Quota",
            self.historical_span_style((179, 179, 179), (17, 17, 17), true),
        );
        ui.paint_str(
            Rect::new(49, 21, 40, 1),
            "                                        ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(89, 21, 28, 1),
            "stale · last good 47 min ago",
            self.historical_span_style((77, 77, 77), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(117, 21, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 21, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 22, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 22, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 22, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(3, 22, 1, 1),
            " ",
            self.historical_span_style((0, 0, 0), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(4, 22, 3, 1),
            "   ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(7, 22, 1, 1),
            "★",
            self.historical_span_style((179, 179, 179), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(8, 22, 27, 1),
            " Go subscription           ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(35, 22, 1, 1),
            "!",
            self.historical_span_style((228, 69, 69), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(36, 22, 3, 1),
            "   ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(39, 22, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(40, 22, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(41, 22, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(42, 22, 22, 1),
            "  Session · 5-hour    ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(64, 22, 26, 1),
            "  76%                     ",
            self.historical_span_style((179, 179, 179), (77, 77, 77), true),
        );
        ui.paint_str(
            Rect::new(90, 22, 8, 1),
            "        ",
            self.historical_span_style((128, 128, 128), (39, 39, 42), false),
        );
        ui.paint_str(
            Rect::new(98, 22, 2, 1),
            "  ",
            self.historical_span_style((77, 77, 77), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(100, 22, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(102, 22, 15, 1),
            "resets in 1 h …",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(117, 22, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 22, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 23, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 23, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 23, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(3, 23, 1, 1),
            " ",
            self.historical_span_style((0, 0, 0), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(4, 23, 3, 1),
            "   ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(7, 23, 1, 1),
            " ",
            self.historical_span_style((179, 179, 179), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(8, 23, 31, 1),
            " ci-bot · discovered           ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(39, 23, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(40, 23, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(41, 23, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(42, 23, 22, 1),
            "  Weekly · all models ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(64, 23, 30, 1),
            "  88%                         ",
            self.historical_span_style((179, 179, 179), (77, 77, 77), true),
        );
        ui.paint_str(
            Rect::new(94, 23, 4, 1),
            "    ",
            self.historical_span_style((128, 128, 128), (39, 39, 42), false),
        );
        ui.paint_str(
            Rect::new(98, 23, 2, 1),
            "  ",
            self.historical_span_style((77, 77, 77), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(100, 23, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(102, 23, 15, 1),
            "resets Sun 19:…",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(117, 23, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 23, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 24, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 24, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 24, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(3, 24, 1, 1),
            " ",
            self.historical_span_style((0, 0, 0), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(4, 24, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(5, 24, 1, 1),
            " ",
            self.historical_span_style((179, 179, 179), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(6, 24, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(7, 24, 30, 1),
            "Unsupported                   ",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(37, 24, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(39, 24, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(40, 24, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(41, 24, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(42, 24, 22, 1),
            "  Weekly · Opus       ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(64, 24, 34, 1),
            " 100%                             ",
            self.historical_span_style((179, 179, 179), (77, 77, 77), true),
        );
        ui.paint_str(
            Rect::new(98, 24, 2, 1),
            "  ",
            self.historical_span_style((77, 77, 77), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(100, 24, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(102, 24, 15, 1),
            "resets Sun 19:…",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(117, 24, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 24, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 25, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 25, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 25, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(3, 25, 1, 1),
            " ",
            self.historical_span_style((0, 0, 0), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(4, 25, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(5, 25, 1, 1),
            " ",
            self.historical_span_style((179, 179, 179), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(6, 25, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(7, 25, 30, 1),
            "+ Add account…                ",
            self.historical_span_style((179, 179, 179), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(37, 25, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(39, 25, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(40, 25, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(41, 25, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(42, 25, 22, 1),
            "  Extra usage credits ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(64, 25, 10, 1),
            "  28%     ",
            self.historical_span_style((179, 179, 179), (77, 77, 77), true),
        );
        ui.paint_str(
            Rect::new(74, 25, 24, 1),
            "                        ",
            self.historical_span_style((128, 128, 128), (39, 39, 42), false),
        );
        ui.paint_str(
            Rect::new(98, 25, 2, 1),
            "  ",
            self.historical_span_style((77, 77, 77), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(100, 25, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(102, 25, 15, 1),
            "1,420 / 5,000 …",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(117, 25, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 25, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 26, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 26, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 26, 37, 1),
            "                                     ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(39, 26, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(40, 26, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(41, 26, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(42, 26, 77, 1),
            "                                                                             ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 26, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 27, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 27, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 27, 37, 1),
            "                                     ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(39, 27, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(40, 27, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(41, 27, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(42, 27, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(44, 27, 10, 1),
            "Validation",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(54, 27, 65, 1),
            "          ✓ material   ✓ identity   ✓ quota access · quota       ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 27, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 28, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 28, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 28, 37, 1),
            "                                     ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(39, 28, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(40, 28, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(41, 28, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(42, 28, 77, 1),
            "                      readable                                               ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 28, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 29, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 29, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 29, 37, 1),
            "                                     ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(39, 29, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(40, 29, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(41, 29, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(42, 29, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(44, 29, 6, 1),
            "Status",
            self.historical_span_style((128, 128, 128), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(50, 29, 14, 1),
            "              ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(64, 29, 53, 1),
            "Usage stale · last good 47 min ago · retry  in 13 min",
            self.historical_span_style((245, 158, 9), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(117, 29, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 29, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 30, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 30, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 30, 37, 1),
            "                                     ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(39, 30, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(40, 30, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(41, 30, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(42, 30, 22, 1),
            "                      ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(64, 30, 11, 1),
            "· retryable",
            self.historical_span_style((245, 158, 9), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(75, 30, 44, 1),
            "                                            ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 30, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 31, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 31, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 31, 37, 1),
            "                                     ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(39, 31, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(40, 31, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(41, 31, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(42, 31, 77, 1),
            "                                                                             ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 31, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 32, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 32, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 32, 37, 1),
            "                                     ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(39, 32, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(40, 32, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(41, 32, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(42, 32, 77, 1),
            "                                                                             ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 32, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 33, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 33, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 33, 37, 1),
            "                                     ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(39, 33, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(40, 33, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(41, 33, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(42, 33, 77, 1),
            "                                                                             ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 33, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 34, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 34, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 34, 37, 1),
            "                                     ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(39, 34, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(40, 34, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(41, 34, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(42, 34, 77, 1),
            "                                                                             ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 34, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 35, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 35, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 35, 37, 1),
            "                                     ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(39, 35, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(40, 35, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(41, 35, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(42, 35, 77, 1),
            "                                                                             ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 35, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 36, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 36, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 36, 37, 1),
            "                                     ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(39, 36, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(40, 36, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(41, 36, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(42, 36, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(43, 36, 1, 1),
            " ",
            self.historical_span_style((39, 39, 42), (39, 39, 42), false),
        );
        ui.paint_str(
            Rect::new(44, 36, 8, 1),
            "Refresh ",
            self.historical_span_style((255, 255, 255), (39, 39, 42), false),
        );
        ui.paint_str(
            Rect::new(52, 36, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(54, 36, 1, 1),
            " ",
            self.historical_span_style((39, 39, 42), (39, 39, 42), false),
        );
        ui.paint_str(
            Rect::new(55, 36, 9, 1),
            "Validate ",
            self.historical_span_style((255, 255, 255), (39, 39, 42), false),
        );
        ui.paint_str(
            Rect::new(64, 36, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(66, 36, 1, 1),
            " ",
            self.historical_span_style((39, 39, 42), (39, 39, 42), false),
        );
        ui.paint_str(
            Rect::new(67, 36, 6, 1),
            "Edit… ",
            self.historical_span_style((255, 255, 255), (39, 39, 42), false),
        );
        ui.paint_str(
            Rect::new(73, 36, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(75, 36, 1, 1),
            " ",
            self.historical_span_style((39, 39, 42), (39, 39, 42), false),
        );
        ui.paint_str(
            Rect::new(76, 36, 12, 1),
            "Set default ",
            self.historical_span_style((255, 255, 255), (39, 39, 42), false),
        );
        ui.paint_str(
            Rect::new(88, 36, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(90, 36, 1, 1),
            " ",
            self.historical_span_style((39, 39, 42), (39, 39, 42), false),
        );
        ui.paint_str(
            Rect::new(91, 36, 8, 1),
            "Disable ",
            self.historical_span_style((255, 255, 255), (39, 39, 42), false),
        );
        ui.paint_str(
            Rect::new(99, 36, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(101, 36, 1, 1),
            " ",
            self.historical_span_style((39, 39, 42), (39, 39, 42), false),
        );
        ui.paint_str(
            Rect::new(102, 36, 8, 1),
            "Remove… ",
            self.historical_span_style((228, 69, 69), (39, 39, 42), false),
        );
        ui.paint_str(
            Rect::new(110, 36, 9, 1),
            "         ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 36, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 37, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 37, 39, 1),
            "╰─────────────────────────────────────╯",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(40, 37, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(41, 37, 1, 1),
            "│",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(42, 37, 77, 1),
            "                                                                             ",
            self.historical_span_style((255, 255, 255), (17, 17, 17), false),
        );
        ui.paint_str(
            Rect::new(119, 37, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(Rect::new(0, 38, 120, 1), "                                                                                                                        ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(
            Rect::new(0, 39, 43, 1),
            "                                           ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(43, 39, 2, 1),
            "↑↓",
            self.historical_span_style((255, 255, 255), (0, 0, 0), true),
        );
        ui.paint_str(
            Rect::new(45, 39, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(46, 39, 6, 1),
            "Scroll",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(52, 39, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(54, 39, 3, 1),
            "Tab",
            self.historical_span_style((255, 255, 255), (0, 0, 0), true),
        );
        ui.paint_str(
            Rect::new(57, 39, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(58, 39, 7, 1),
            "Actions",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(65, 39, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(67, 39, 3, 1),
            "Esc",
            self.historical_span_style((255, 255, 255), (0, 0, 0), true),
        );
        ui.paint_str(
            Rect::new(70, 39, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(71, 39, 4, 1),
            "Back",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(75, 39, 45, 1),
            "                                             ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
    }

    pub(super) fn draw_historical_settings_route_120_40(&self, ui: &mut Ui<'_>, area: Rect) {
        ui.fill(
            area,
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 0, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 0, 9, 1),
            " jackin❯ ",
            self.historical_span_style((25, 25, 28), (72, 224, 84), true),
        );
        ui.paint_str(
            Rect::new(10, 0, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(12, 0, 6, 1),
            " File ",
            self.historical_span_style((179, 179, 179), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(18, 0, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(19, 0, 4, 1),
            " Go ",
            self.historical_span_style((179, 179, 179), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(23, 0, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(24, 0, 6, 1),
            " Help ",
            self.historical_span_style((179, 179, 179), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(30, 0, 29, 1),
            "                             ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(59, 0, 27, 1),
            "Settings › global › General",
            self.historical_span_style((179, 179, 179), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(86, 0, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(88, 0, 20, 1),
            "inside the Construct",
            self.historical_span_style((179, 179, 179), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(108, 0, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(110, 0, 9, 1),
            "2 running",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(119, 0, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(Rect::new(0, 1, 120, 1), "                                                                                                                        ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(Rect::new(0, 2, 120, 1), "                                                                                                                        ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(
            Rect::new(0, 3, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 3, 10, 1),
            " General  ",
            self.historical_span_style((255, 255, 255), (24, 24, 27), true),
        );
        ui.paint_str(
            Rect::new(12, 3, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(13, 3, 9, 1),
            " Mounts  ",
            self.historical_span_style((179, 179, 179), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(22, 3, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(23, 3, 15, 1),
            " Environments  ",
            self.historical_span_style((179, 179, 179), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(38, 3, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(39, 3, 9, 1),
            " Agents  ",
            self.historical_span_style((179, 179, 179), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(48, 3, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(49, 3, 8, 1),
            " Trust  ",
            self.historical_span_style((179, 179, 179), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(57, 3, 63, 1),
            "                                                               ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 4, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 4, 10, 1),
            "━━━━━━━━━━",
            self.historical_span_style((72, 224, 84), (0, 0, 0), false),
        );
        ui.paint_str(Rect::new(12, 4, 106, 1), "──────────────────────────────────────────────────────────────────────────────────────────────────────────", self.historical_span_style((38, 38, 38), (0, 0, 0), false));
        ui.paint_str(
            Rect::new(118, 4, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(Rect::new(0, 5, 120, 1), "                                                                                                                        ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(
            Rect::new(0, 6, 4, 1),
            "    ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(4, 6, 7, 1),
            "Commits",
            self.historical_span_style((179, 179, 179), (0, 0, 0), true),
        );
        ui.paint_str(Rect::new(11, 6, 109, 1), "                                                                                                             ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(
            Rect::new(0, 7, 4, 1),
            "    ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(4, 7, 1, 1),
            " ",
            self.historical_span_style((0, 0, 0), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(5, 7, 3, 1),
            "[✓]",
            self.historical_span_style((72, 224, 84), (0, 0, 0), false),
        );
        ui.paint_str(Rect::new(8, 7, 112, 1), " Add Co-authored-by trailer                                                                                     ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(
            Rect::new(0, 8, 4, 1),
            "    ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(4, 8, 1, 1),
            " ",
            self.historical_span_style((0, 0, 0), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(5, 8, 3, 1),
            "[✓]",
            self.historical_span_style((72, 224, 84), (0, 0, 0), false),
        );
        ui.paint_str(Rect::new(8, 8, 112, 1), " Sign off commits (DCO)                                                                                         ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(Rect::new(0, 9, 120, 1), "                                                                                                                        ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(
            Rect::new(0, 10, 4, 1),
            "    ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(4, 10, 53, 1),
            "Two independent flags; both apply to every Workspace.",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(57, 10, 63, 1),
            "                                                               ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(Rect::new(0, 11, 120, 1), "                                                                                                                        ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(
            Rect::new(0, 12, 4, 1),
            "    ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(4, 12, 8, 1),
            "Trailers",
            self.historical_span_style((179, 179, 179), (0, 0, 0), true),
        );
        ui.paint_str(Rect::new(12, 12, 108, 1), "                                                                                                            ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(
            Rect::new(0, 13, 4, 1),
            "    ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(Rect::new(4, 13, 95, 1), "Co-authored-by: <agent> <noreply@…>   ·   Signed-off-by: Alexey Zhokhov <alexey@chainargos.com>", self.historical_span_style((128, 128, 128), (0, 0, 0), false));
        ui.paint_str(
            Rect::new(99, 13, 21, 1),
            "                     ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(Rect::new(0, 14, 120, 1), "                                                                                                                        ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(Rect::new(0, 15, 120, 1), "                                                                                                                        ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(Rect::new(0, 16, 120, 1), "                                                                                                                        ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(Rect::new(0, 17, 120, 1), "                                                                                                                        ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(Rect::new(0, 18, 120, 1), "                                                                                                                        ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(Rect::new(0, 19, 120, 1), "                                                                                                                        ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(Rect::new(0, 20, 120, 1), "                                                                                                                        ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(Rect::new(0, 21, 120, 1), "                                                                                                                        ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(Rect::new(0, 22, 120, 1), "                                                                                                                        ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(Rect::new(0, 23, 120, 1), "                                                                                                                        ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(Rect::new(0, 24, 120, 1), "                                                                                                                        ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(Rect::new(0, 25, 120, 1), "                                                                                                                        ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(Rect::new(0, 26, 120, 1), "                                                                                                                        ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(Rect::new(0, 27, 120, 1), "                                                                                                                        ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(Rect::new(0, 28, 120, 1), "                                                                                                                        ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(Rect::new(0, 29, 120, 1), "                                                                                                                        ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(Rect::new(0, 30, 120, 1), "                                                                                                                        ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(Rect::new(0, 31, 120, 1), "                                                                                                                        ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(Rect::new(0, 32, 120, 1), "                                                                                                                        ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(Rect::new(0, 33, 120, 1), "                                                                                                                        ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(Rect::new(0, 34, 120, 1), "                                                                                                                        ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(Rect::new(0, 35, 120, 1), "                                                                                                                        ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(Rect::new(0, 36, 120, 1), "                                                                                                                        ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(Rect::new(0, 37, 97, 1), "                                                                                                 ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(
            Rect::new(97, 37, 1, 1),
            " ",
            self.historical_span_style((0, 0, 0), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(98, 37, 7, 1),
            "Cancel ",
            self.historical_span_style((179, 179, 179), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(105, 37, 3, 1),
            "   ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(108, 37, 1, 1),
            " ",
            self.historical_span_style((24, 24, 27), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(109, 37, 6, 1),
            "Save… ",
            self.historical_span_style((77, 77, 77), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(115, 37, 5, 1),
            "     ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(Rect::new(0, 38, 120, 1), "                                                                                                                        ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(
            Rect::new(0, 39, 25, 1),
            "                         ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(25, 39, 3, 1),
            "← →",
            self.historical_span_style((255, 255, 255), (0, 0, 0), true),
        );
        ui.paint_str(
            Rect::new(28, 39, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(29, 39, 3, 1),
            "Tab",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(32, 39, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(34, 39, 3, 1),
            "1–5",
            self.historical_span_style((255, 255, 255), (0, 0, 0), true),
        );
        ui.paint_str(
            Rect::new(37, 39, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(38, 39, 4, 1),
            "Jump",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(42, 39, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(44, 39, 5, 1),
            "Enter",
            self.historical_span_style((255, 255, 255), (0, 0, 0), true),
        );
        ui.paint_str(
            Rect::new(49, 39, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(50, 39, 4, 1),
            "Body",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(54, 39, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(56, 39, 3, 1),
            "[ ]",
            self.historical_span_style((255, 255, 255), (0, 0, 0), true),
        );
        ui.paint_str(
            Rect::new(59, 39, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(60, 39, 10, 1),
            "Switch tab",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(70, 39, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(72, 39, 6, 1),
            "Ctrl+S",
            self.historical_span_style((255, 255, 255), (0, 0, 0), true),
        );
        ui.paint_str(
            Rect::new(78, 39, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(79, 39, 4, 1),
            "Save",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(83, 39, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(85, 39, 3, 1),
            "Esc",
            self.historical_span_style((255, 255, 255), (0, 0, 0), true),
        );
        ui.paint_str(
            Rect::new(88, 39, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(89, 39, 4, 1),
            "Back",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(93, 39, 27, 1),
            "                           ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
    }

    pub(super) fn draw_historical_settings_mounts_120_40(&self, ui: &mut Ui<'_>, area: Rect) {
        ui.fill(
            area,
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 0, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 0, 9, 1),
            " jackin❯ ",
            self.historical_span_style((25, 25, 28), (72, 224, 84), true),
        );
        ui.paint_str(
            Rect::new(10, 0, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(12, 0, 6, 1),
            " File ",
            self.historical_span_style((179, 179, 179), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(18, 0, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(19, 0, 4, 1),
            " Go ",
            self.historical_span_style((179, 179, 179), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(23, 0, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(24, 0, 6, 1),
            " Help ",
            self.historical_span_style((179, 179, 179), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(30, 0, 30, 1),
            "                              ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(60, 0, 26, 1),
            "Settings › global › Mounts",
            self.historical_span_style((179, 179, 179), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(86, 0, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(88, 0, 20, 1),
            "inside the Construct",
            self.historical_span_style((179, 179, 179), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(108, 0, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(110, 0, 9, 1),
            "2 running",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(119, 0, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(Rect::new(0, 1, 120, 1), "                                                                                                                        ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(Rect::new(0, 2, 120, 1), "                                                                                                                        ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(
            Rect::new(0, 3, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 3, 10, 1),
            " General  ",
            self.historical_span_style((179, 179, 179), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(12, 3, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(13, 3, 9, 1),
            " Mounts  ",
            self.historical_span_style((255, 255, 255), (24, 24, 27), true),
        );
        ui.paint_str(
            Rect::new(22, 3, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(23, 3, 15, 1),
            " Environments  ",
            self.historical_span_style((179, 179, 179), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(38, 3, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(39, 3, 9, 1),
            " Agents  ",
            self.historical_span_style((179, 179, 179), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(48, 3, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(49, 3, 8, 1),
            " Trust  ",
            self.historical_span_style((179, 179, 179), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(57, 3, 63, 1),
            "                                                               ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 4, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 4, 11, 1),
            "───────────",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(13, 4, 9, 1),
            "━━━━━━━━━",
            self.historical_span_style((72, 224, 84), (0, 0, 0), false),
        );
        ui.paint_str(Rect::new(22, 4, 96, 1), "────────────────────────────────────────────────────────────────────────────────────────────────", self.historical_span_style((38, 38, 38), (0, 0, 0), false));
        ui.paint_str(
            Rect::new(118, 4, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(Rect::new(0, 5, 120, 1), "                                                                                                                        ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(
            Rect::new(0, 6, 8, 1),
            "        ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(8, 6, 11, 1),
            "Destination",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(19, 6, 20, 1),
            "                    ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(39, 6, 5, 1),
            "Scope",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(44, 6, 11, 1),
            "           ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(55, 6, 4, 1),
            "Mode",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(59, 6, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(61, 6, 9, 1),
            "Isolation",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(70, 6, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(72, 6, 4, 1),
            "Kind",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(76, 6, 4, 1),
            "    ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(80, 6, 6, 1),
            "Source",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(86, 6, 34, 1),
            "                                  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 7, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 7, 2, 1),
            "▎›",
            self.historical_span_style((72, 224, 84), (15, 46, 19), true),
        );
        ui.paint_str(
            Rect::new(4, 7, 35, 1),
            "    /home/agent/.gitconfig         ",
            self.historical_span_style((255, 255, 255), (15, 46, 19), true),
        );
        ui.paint_str(
            Rect::new(39, 7, 14, 1),
            "global        ",
            self.historical_span_style((128, 128, 128), (15, 46, 19), true),
        );
        ui.paint_str(
            Rect::new(53, 7, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (15, 46, 19), true),
        );
        ui.paint_str(
            Rect::new(55, 7, 4, 1),
            "ro  ",
            self.historical_span_style((179, 179, 179), (15, 46, 19), true),
        );
        ui.paint_str(
            Rect::new(59, 7, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (15, 46, 19), true),
        );
        ui.paint_str(
            Rect::new(61, 7, 9, 1),
            "shared   ",
            self.historical_span_style((179, 179, 179), (15, 46, 19), true),
        );
        ui.paint_str(
            Rect::new(70, 7, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (15, 46, 19), true),
        );
        ui.paint_str(
            Rect::new(72, 7, 6, 1),
            "host  ",
            self.historical_span_style((128, 128, 128), (15, 46, 19), true),
        );
        ui.paint_str(
            Rect::new(78, 7, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (15, 46, 19), true),
        );
        ui.paint_str(
            Rect::new(80, 7, 37, 1),
            "~/.gitconfig                         ",
            self.historical_span_style((128, 128, 128), (15, 46, 19), true),
        );
        ui.paint_str(
            Rect::new(117, 7, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (15, 46, 19), true),
        );
        ui.paint_str(
            Rect::new(118, 7, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 8, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 8, 1, 1),
            " ",
            self.historical_span_style((0, 0, 0), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(3, 8, 1, 1),
            " ",
            self.historical_span_style((179, 179, 179), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(4, 8, 35, 1),
            "    /home/agent/.cargo/registry    ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(39, 8, 14, 1),
            "global        ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(53, 8, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(55, 8, 4, 1),
            "rw  ",
            self.historical_span_style((179, 179, 179), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(59, 8, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(61, 8, 9, 1),
            "shared   ",
            self.historical_span_style((179, 179, 179), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(70, 8, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(72, 8, 6, 1),
            "host  ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(78, 8, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(80, 8, 37, 1),
            "~/.cache/cargo-registry              ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(117, 8, 3, 1),
            "   ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 9, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 9, 1, 1),
            " ",
            self.historical_span_style((0, 0, 0), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(3, 9, 1, 1),
            " ",
            self.historical_span_style((179, 179, 179), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(4, 9, 35, 1),
            "    /home/agent/.kube              ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(39, 9, 14, 1),
            "role sre      ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(53, 9, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(55, 9, 4, 1),
            "ro  ",
            self.historical_span_style((179, 179, 179), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(59, 9, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(61, 9, 9, 1),
            "shared   ",
            self.historical_span_style((179, 179, 179), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(70, 9, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(72, 9, 6, 1),
            "host  ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(78, 9, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(80, 9, 37, 1),
            "~/roles/sre-kube                     ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(117, 9, 3, 1),
            "   ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 10, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 10, 1, 1),
            " ",
            self.historical_span_style((0, 0, 0), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(3, 10, 1, 1),
            " ",
            self.historical_span_style((179, 179, 179), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(4, 10, 4, 1),
            "    ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(8, 10, 11, 1),
            "+ Add mount",
            self.historical_span_style((179, 179, 179), (0, 0, 0), false),
        );
        ui.paint_str(Rect::new(19, 10, 101, 1), "                                                                                                     ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(Rect::new(0, 11, 120, 1), "                                                                                                                        ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(Rect::new(0, 12, 120, 1), "                                                                                                                        ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(Rect::new(0, 13, 120, 1), "                                                                                                                        ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(Rect::new(0, 14, 120, 1), "                                                                                                                        ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(Rect::new(0, 15, 120, 1), "                                                                                                                        ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(Rect::new(0, 16, 120, 1), "                                                                                                                        ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(Rect::new(0, 17, 120, 1), "                                                                                                                        ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(Rect::new(0, 18, 120, 1), "                                                                                                                        ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(Rect::new(0, 19, 120, 1), "                                                                                                                        ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(Rect::new(0, 20, 120, 1), "                                                                                                                        ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(Rect::new(0, 21, 120, 1), "                                                                                                                        ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(Rect::new(0, 22, 120, 1), "                                                                                                                        ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(Rect::new(0, 23, 120, 1), "                                                                                                                        ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(Rect::new(0, 24, 120, 1), "                                                                                                                        ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(Rect::new(0, 25, 120, 1), "                                                                                                                        ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(Rect::new(0, 26, 120, 1), "                                                                                                                        ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(Rect::new(0, 27, 120, 1), "                                                                                                                        ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(Rect::new(0, 28, 120, 1), "                                                                                                                        ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(Rect::new(0, 29, 120, 1), "                                                                                                                        ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(Rect::new(0, 30, 120, 1), "                                                                                                                        ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(Rect::new(0, 31, 120, 1), "                                                                                                                        ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(Rect::new(0, 32, 120, 1), "                                                                                                                        ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(Rect::new(0, 33, 120, 1), "                                                                                                                        ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(Rect::new(0, 34, 120, 1), "                                                                                                                        ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(Rect::new(0, 35, 120, 1), "                                                                                                                        ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(Rect::new(0, 36, 120, 1), "                                                                                                                        ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(Rect::new(0, 37, 97, 1), "                                                                                                 ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(
            Rect::new(97, 37, 1, 1),
            " ",
            self.historical_span_style((0, 0, 0), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(98, 37, 7, 1),
            "Cancel ",
            self.historical_span_style((179, 179, 179), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(105, 37, 3, 1),
            "   ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(108, 37, 1, 1),
            " ",
            self.historical_span_style((24, 24, 27), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(109, 37, 6, 1),
            "Save… ",
            self.historical_span_style((77, 77, 77), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(115, 37, 5, 1),
            "     ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(Rect::new(0, 38, 120, 1), "                                                                                                                        ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(
            Rect::new(0, 39, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 39, 5, 1),
            "Enter",
            self.historical_span_style((255, 255, 255), (0, 0, 0), true),
        );
        ui.paint_str(
            Rect::new(6, 39, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(7, 39, 5, 1),
            "Edit…",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(12, 39, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(14, 39, 1, 1),
            "r",
            self.historical_span_style((255, 255, 255), (0, 0, 0), true),
        );
        ui.paint_str(
            Rect::new(15, 39, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(16, 39, 9, 1),
            "Read-only",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(25, 39, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(27, 39, 1, 1),
            "i",
            self.historical_span_style((255, 255, 255), (0, 0, 0), true),
        );
        ui.paint_str(
            Rect::new(28, 39, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(29, 39, 9, 1),
            "Isolation",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(38, 39, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(40, 39, 1, 1),
            "o",
            self.historical_span_style((255, 255, 255), (0, 0, 0), true),
        );
        ui.paint_str(
            Rect::new(41, 39, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(42, 39, 11, 1),
            "Open source",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(53, 39, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(55, 39, 1, 1),
            "d",
            self.historical_span_style((255, 255, 255), (0, 0, 0), true),
        );
        ui.paint_str(
            Rect::new(56, 39, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(57, 39, 6, 1),
            "Remove",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(63, 39, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(65, 39, 1, 1),
            "s",
            self.historical_span_style((255, 255, 255), (0, 0, 0), true),
        );
        ui.paint_str(
            Rect::new(66, 39, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(67, 39, 6, 1),
            "Scope…",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(73, 39, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(75, 39, 1, 1),
            "a",
            self.historical_span_style((255, 255, 255), (0, 0, 0), true),
        );
        ui.paint_str(
            Rect::new(76, 39, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(77, 39, 10, 1),
            "Add mount…",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(87, 39, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(89, 39, 3, 1),
            "[ ]",
            self.historical_span_style((255, 255, 255), (0, 0, 0), true),
        );
        ui.paint_str(
            Rect::new(92, 39, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(93, 39, 10, 1),
            "Switch tab",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(103, 39, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(105, 39, 6, 1),
            "Ctrl+S",
            self.historical_span_style((255, 255, 255), (0, 0, 0), true),
        );
        ui.paint_str(
            Rect::new(111, 39, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(112, 39, 4, 1),
            "Save",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(116, 39, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(118, 39, 1, 1),
            "…",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(119, 39, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
    }

    pub(super) fn draw_historical_settings_env_120_40(&self, ui: &mut Ui<'_>, area: Rect) {
        ui.fill(
            area,
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 0, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 0, 9, 1),
            " jackin❯ ",
            self.historical_span_style((25, 25, 28), (72, 224, 84), true),
        );
        ui.paint_str(
            Rect::new(10, 0, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(12, 0, 6, 1),
            " File ",
            self.historical_span_style((179, 179, 179), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(18, 0, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(19, 0, 4, 1),
            " Go ",
            self.historical_span_style((179, 179, 179), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(23, 0, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(24, 0, 6, 1),
            " Help ",
            self.historical_span_style((179, 179, 179), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(30, 0, 24, 1),
            "                        ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(54, 0, 32, 1),
            "Settings › global › Environments",
            self.historical_span_style((179, 179, 179), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(86, 0, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(88, 0, 20, 1),
            "inside the Construct",
            self.historical_span_style((179, 179, 179), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(108, 0, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(110, 0, 9, 1),
            "2 running",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(119, 0, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(Rect::new(0, 1, 120, 1), "                                                                                                                        ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(Rect::new(0, 2, 120, 1), "                                                                                                                        ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(
            Rect::new(0, 3, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 3, 10, 1),
            " General  ",
            self.historical_span_style((179, 179, 179), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(12, 3, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(13, 3, 9, 1),
            " Mounts  ",
            self.historical_span_style((179, 179, 179), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(22, 3, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(23, 3, 15, 1),
            " Environments  ",
            self.historical_span_style((255, 255, 255), (24, 24, 27), true),
        );
        ui.paint_str(
            Rect::new(38, 3, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(39, 3, 9, 1),
            " Agents  ",
            self.historical_span_style((179, 179, 179), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(48, 3, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(49, 3, 8, 1),
            " Trust  ",
            self.historical_span_style((179, 179, 179), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(57, 3, 63, 1),
            "                                                               ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 4, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 4, 21, 1),
            "─────────────────────",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(23, 4, 15, 1),
            "━━━━━━━━━━━━━━━",
            self.historical_span_style((72, 224, 84), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(38, 4, 80, 1),
            "────────────────────────────────────────────────────────────────────────────────",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(118, 4, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(Rect::new(0, 5, 120, 1), "                                                                                                                        ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(
            Rect::new(0, 6, 6, 1),
            "      ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(6, 6, 6, 1),
            "Global",
            self.historical_span_style((179, 179, 179), (0, 0, 0), true),
        );
        ui.paint_str(Rect::new(12, 6, 99, 1), "                                                                                                   ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(
            Rect::new(111, 6, 6, 1),
            "3 vars",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(117, 6, 3, 1),
            "   ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 7, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 7, 2, 1),
            "▎›",
            self.historical_span_style((72, 224, 84), (15, 46, 19), true),
        );
        ui.paint_str(
            Rect::new(4, 7, 24, 1),
            "    GH_TOKEN            ",
            self.historical_span_style((255, 255, 255), (15, 46, 19), true),
        );
        ui.paint_str(
            Rect::new(28, 7, 14, 1),
            "global        ",
            self.historical_span_style((128, 128, 128), (15, 46, 19), true),
        );
        ui.paint_str(
            Rect::new(42, 7, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (15, 46, 19), true),
        );
        ui.paint_str(
            Rect::new(44, 7, 22, 1),
            "****************      ",
            self.historical_span_style((179, 179, 179), (15, 46, 19), true),
        );
        ui.paint_str(
            Rect::new(66, 7, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (15, 46, 19), true),
        );
        ui.paint_str(
            Rect::new(68, 7, 49, 1),
            "[op] Engineering › GitHub · CLI token › credenti…",
            self.historical_span_style((128, 128, 128), (15, 46, 19), true),
        );
        ui.paint_str(
            Rect::new(117, 7, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (15, 46, 19), true),
        );
        ui.paint_str(
            Rect::new(118, 7, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 8, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 8, 1, 1),
            " ",
            self.historical_span_style((0, 0, 0), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(3, 8, 1, 1),
            " ",
            self.historical_span_style((179, 179, 179), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(4, 8, 24, 1),
            "    EDITOR              ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(28, 8, 14, 1),
            "global        ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(42, 8, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(44, 8, 22, 1),
            "****                  ",
            self.historical_span_style((179, 179, 179), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(66, 8, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(68, 8, 49, 1),
            "plain                                            ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(117, 8, 3, 1),
            "   ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 9, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 9, 1, 1),
            " ",
            self.historical_span_style((0, 0, 0), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(3, 9, 1, 1),
            " ",
            self.historical_span_style((179, 179, 179), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(4, 9, 24, 1),
            "    CARGO_NET_GIT_FET…  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(28, 9, 14, 1),
            "global        ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(42, 9, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(44, 9, 22, 1),
            "****                  ",
            self.historical_span_style((179, 179, 179), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(66, 9, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(68, 9, 49, 1),
            "plain                                            ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(117, 9, 3, 1),
            "   ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 10, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 10, 1, 1),
            " ",
            self.historical_span_style((0, 0, 0), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(3, 10, 1, 1),
            " ",
            self.historical_span_style((179, 179, 179), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(4, 10, 4, 1),
            "    ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(8, 10, 26, 1),
            "+ Add environment variable",
            self.historical_span_style((179, 179, 179), (0, 0, 0), false),
        );
        ui.paint_str(Rect::new(34, 10, 86, 1), "                                                                                      ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(
            Rect::new(0, 11, 6, 1),
            "      ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(6, 11, 14, 1),
            "Role overrides",
            self.historical_span_style((179, 179, 179), (0, 0, 0), true),
        );
        ui.paint_str(
            Rect::new(20, 11, 64, 1),
            "                                                                ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(84, 11, 33, 1),
            "1 configured · 46 in the registry",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(117, 11, 3, 1),
            "   ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 12, 4, 1),
            "    ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(4, 12, 1, 1),
            "▾",
            self.historical_span_style((179, 179, 179), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(5, 12, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(6, 12, 9, 1),
            "Role: sre",
            self.historical_span_style((179, 179, 179), (0, 0, 0), true),
        );
        ui.paint_str(Rect::new(15, 12, 97, 1), "                                                                                                 ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(
            Rect::new(112, 12, 5, 1),
            "1 var",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(117, 12, 3, 1),
            "   ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 13, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 13, 1, 1),
            " ",
            self.historical_span_style((0, 0, 0), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(3, 13, 1, 1),
            " ",
            self.historical_span_style((179, 179, 179), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(4, 13, 24, 1),
            "    KUBECONFIG          ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(28, 13, 14, 1),
            "role sre      ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(42, 13, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(44, 13, 22, 1),
            "****************      ",
            self.historical_span_style((179, 179, 179), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(66, 13, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(68, 13, 49, 1),
            "plain                                            ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(117, 13, 3, 1),
            "   ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 14, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 14, 1, 1),
            " ",
            self.historical_span_style((0, 0, 0), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(3, 14, 1, 1),
            " ",
            self.historical_span_style((179, 179, 179), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(4, 14, 4, 1),
            "    ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(8, 14, 30, 1),
            "+ Add sre environment variable",
            self.historical_span_style((179, 179, 179), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(38, 14, 82, 1),
            "                                                                                  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 15, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 15, 1, 1),
            " ",
            self.historical_span_style((0, 0, 0), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(3, 15, 1, 1),
            " ",
            self.historical_span_style((179, 179, 179), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(4, 15, 4, 1),
            "    ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(8, 15, 20, 1),
            "+ Add role override…",
            self.historical_span_style((179, 179, 179), (0, 0, 0), false),
        );
        ui.paint_str(Rect::new(28, 15, 92, 1), "                                                                                            ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(Rect::new(0, 16, 120, 1), "                                                                                                                        ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(Rect::new(0, 17, 120, 1), "                                                                                                                        ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(Rect::new(0, 18, 120, 1), "                                                                                                                        ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(Rect::new(0, 19, 120, 1), "                                                                                                                        ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(Rect::new(0, 20, 120, 1), "                                                                                                                        ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(Rect::new(0, 21, 120, 1), "                                                                                                                        ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(Rect::new(0, 22, 120, 1), "                                                                                                                        ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(Rect::new(0, 23, 120, 1), "                                                                                                                        ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(Rect::new(0, 24, 120, 1), "                                                                                                                        ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(Rect::new(0, 25, 120, 1), "                                                                                                                        ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(Rect::new(0, 26, 120, 1), "                                                                                                                        ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(Rect::new(0, 27, 120, 1), "                                                                                                                        ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(Rect::new(0, 28, 120, 1), "                                                                                                                        ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(Rect::new(0, 29, 120, 1), "                                                                                                                        ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(Rect::new(0, 30, 120, 1), "                                                                                                                        ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(Rect::new(0, 31, 120, 1), "                                                                                                                        ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(Rect::new(0, 32, 120, 1), "                                                                                                                        ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(Rect::new(0, 33, 120, 1), "                                                                                                                        ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(Rect::new(0, 34, 120, 1), "                                                                                                                        ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(Rect::new(0, 35, 120, 1), "                                                                                                                        ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(Rect::new(0, 36, 120, 1), "                                                                                                                        ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(Rect::new(0, 37, 97, 1), "                                                                                                 ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(
            Rect::new(97, 37, 1, 1),
            " ",
            self.historical_span_style((0, 0, 0), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(98, 37, 7, 1),
            "Cancel ",
            self.historical_span_style((179, 179, 179), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(105, 37, 3, 1),
            "   ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(108, 37, 1, 1),
            " ",
            self.historical_span_style((24, 24, 27), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(109, 37, 6, 1),
            "Save… ",
            self.historical_span_style((77, 77, 77), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(115, 37, 5, 1),
            "     ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(Rect::new(0, 38, 120, 1), "                                                                                                                        ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(
            Rect::new(0, 39, 9, 1),
            "         ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(9, 39, 5, 1),
            "Enter",
            self.historical_span_style((255, 255, 255), (0, 0, 0), true),
        );
        ui.paint_str(
            Rect::new(14, 39, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(15, 39, 4, 1),
            "Edit",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(19, 39, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(21, 39, 1, 1),
            "m",
            self.historical_span_style((255, 255, 255), (0, 0, 0), true),
        );
        ui.paint_str(
            Rect::new(22, 39, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(23, 39, 4, 1),
            "Show",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(27, 39, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(29, 39, 1, 1),
            "p",
            self.historical_span_style((255, 255, 255), (0, 0, 0), true),
        );
        ui.paint_str(
            Rect::new(30, 39, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(31, 39, 10, 1),
            "1Password…",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(41, 39, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(43, 39, 1, 1),
            "s",
            self.historical_span_style((255, 255, 255), (0, 0, 0), true),
        );
        ui.paint_str(
            Rect::new(44, 39, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(45, 39, 6, 1),
            "Scope…",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(51, 39, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(53, 39, 1, 1),
            "d",
            self.historical_span_style((255, 255, 255), (0, 0, 0), true),
        );
        ui.paint_str(
            Rect::new(54, 39, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(55, 39, 7, 1),
            "Remove…",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(62, 39, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(64, 39, 1, 1),
            "a",
            self.historical_span_style((255, 255, 255), (0, 0, 0), true),
        );
        ui.paint_str(
            Rect::new(65, 39, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(66, 39, 4, 1),
            "Add…",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(70, 39, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(72, 39, 3, 1),
            "[ ]",
            self.historical_span_style((255, 255, 255), (0, 0, 0), true),
        );
        ui.paint_str(
            Rect::new(75, 39, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(76, 39, 10, 1),
            "Switch tab",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(86, 39, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(88, 39, 6, 1),
            "Ctrl+S",
            self.historical_span_style((255, 255, 255), (0, 0, 0), true),
        );
        ui.paint_str(
            Rect::new(94, 39, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(95, 39, 4, 1),
            "Save",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(99, 39, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(101, 39, 3, 1),
            "Esc",
            self.historical_span_style((255, 255, 255), (0, 0, 0), true),
        );
        ui.paint_str(
            Rect::new(104, 39, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(105, 39, 4, 1),
            "Back",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(109, 39, 11, 1),
            "           ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
    }

    pub(super) fn draw_historical_settings_agents_120_40(&self, ui: &mut Ui<'_>, area: Rect) {
        ui.fill(
            area,
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 0, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 0, 9, 1),
            " jackin❯ ",
            self.historical_span_style((25, 25, 28), (72, 224, 84), true),
        );
        ui.paint_str(
            Rect::new(10, 0, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(12, 0, 6, 1),
            " File ",
            self.historical_span_style((179, 179, 179), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(18, 0, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(19, 0, 4, 1),
            " Go ",
            self.historical_span_style((179, 179, 179), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(23, 0, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(24, 0, 6, 1),
            " Help ",
            self.historical_span_style((179, 179, 179), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(30, 0, 30, 1),
            "                              ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(60, 0, 26, 1),
            "Settings › global › Agents",
            self.historical_span_style((179, 179, 179), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(86, 0, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(88, 0, 20, 1),
            "inside the Construct",
            self.historical_span_style((179, 179, 179), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(108, 0, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(110, 0, 9, 1),
            "2 running",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(119, 0, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(Rect::new(0, 1, 120, 1), "                                                                                                                        ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(Rect::new(0, 2, 120, 1), "                                                                                                                        ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(
            Rect::new(0, 3, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 3, 10, 1),
            " General  ",
            self.historical_span_style((179, 179, 179), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(12, 3, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(13, 3, 9, 1),
            " Mounts  ",
            self.historical_span_style((179, 179, 179), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(22, 3, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(23, 3, 15, 1),
            " Environments  ",
            self.historical_span_style((179, 179, 179), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(38, 3, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(39, 3, 9, 1),
            " Agents  ",
            self.historical_span_style((255, 255, 255), (24, 24, 27), true),
        );
        ui.paint_str(
            Rect::new(48, 3, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(49, 3, 8, 1),
            " Trust  ",
            self.historical_span_style((179, 179, 179), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(57, 3, 63, 1),
            "                                                               ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 4, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 4, 37, 1),
            "─────────────────────────────────────",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(39, 4, 9, 1),
            "━━━━━━━━━",
            self.historical_span_style((72, 224, 84), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(48, 4, 70, 1),
            "──────────────────────────────────────────────────────────────────────",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(118, 4, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(Rect::new(0, 5, 120, 1), "                                                                                                                        ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(
            Rect::new(0, 6, 4, 1),
            "    ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(4, 6, 18, 1),
            "Agent runtime mode",
            self.historical_span_style((179, 179, 179), (0, 0, 0), true),
        );
        ui.paint_str(
            Rect::new(22, 6, 55, 1),
            "                                                       ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(77, 6, 39, 1),
            "accounts are registered in Accounts (c)",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(116, 6, 4, 1),
            "    ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 7, 7, 1),
            "       ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(7, 7, 43, 1),
            "Agent         Mode         Registry default",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(50, 7, 70, 1),
            "                                                                      ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 8, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 8, 2, 1),
            "▎›",
            self.historical_span_style((72, 224, 84), (15, 46, 19), true),
        );
        ui.paint_str(
            Rect::new(4, 8, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (15, 46, 19), true),
        );
        ui.paint_str(
            Rect::new(5, 8, 1, 1),
            " ",
            self.historical_span_style((245, 158, 9), (15, 46, 19), true),
        );
        ui.paint_str(
            Rect::new(6, 8, 15, 1),
            " Claude Code   ",
            self.historical_span_style((255, 255, 255), (15, 46, 19), true),
        );
        ui.paint_str(
            Rect::new(21, 8, 12, 1),
            "sync        ",
            self.historical_span_style((179, 179, 179), (15, 46, 19), true),
        );
        ui.paint_str(
            Rect::new(33, 8, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (15, 46, 19), true),
        );
        ui.paint_str(
            Rect::new(34, 8, 19, 1),
            "★ Claude · Personal",
            self.historical_span_style((128, 128, 128), (15, 46, 19), true),
        );
        ui.paint_str(
            Rect::new(53, 8, 65, 1),
            "                                                                 ",
            self.historical_span_style((255, 255, 255), (15, 46, 19), true),
        );
        ui.paint_str(
            Rect::new(118, 8, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 9, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 9, 1, 1),
            " ",
            self.historical_span_style((0, 0, 0), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(3, 9, 1, 1),
            " ",
            self.historical_span_style((179, 179, 179), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(4, 9, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(5, 9, 1, 1),
            " ",
            self.historical_span_style((245, 158, 9), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(6, 9, 15, 1),
            " Codex         ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(21, 9, 12, 1),
            "api key     ",
            self.historical_span_style((179, 179, 179), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(33, 9, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(34, 9, 17, 1),
            "★ Codex · Primary",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(51, 9, 69, 1),
            "                                                                     ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 10, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 10, 1, 1),
            " ",
            self.historical_span_style((0, 0, 0), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(3, 10, 1, 1),
            " ",
            self.historical_span_style((179, 179, 179), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(4, 10, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(5, 10, 1, 1),
            " ",
            self.historical_span_style((245, 158, 9), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(6, 10, 15, 1),
            " Amp           ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(21, 10, 12, 1),
            "sync        ",
            self.historical_span_style((179, 179, 179), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(33, 10, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(34, 10, 39, 1),
            "discovered · ~/.config/amp/secrets.json",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(73, 10, 47, 1),
            "                                               ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 11, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 11, 1, 1),
            " ",
            self.historical_span_style((0, 0, 0), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(3, 11, 1, 1),
            " ",
            self.historical_span_style((179, 179, 179), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(4, 11, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(5, 11, 1, 1),
            " ",
            self.historical_span_style((245, 158, 9), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(6, 11, 15, 1),
            " Kimi Code     ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(21, 11, 12, 1),
            "ignore      ",
            self.historical_span_style((179, 179, 179), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(33, 11, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(34, 11, 38, 1),
            "no credentials handed to the container",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(72, 11, 48, 1),
            "                                                ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 12, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 12, 1, 1),
            " ",
            self.historical_span_style((0, 0, 0), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(3, 12, 1, 1),
            " ",
            self.historical_span_style((179, 179, 179), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(4, 12, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(5, 12, 1, 1),
            " ",
            self.historical_span_style((245, 158, 9), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(6, 12, 15, 1),
            " OpenCode      ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(21, 12, 12, 1),
            "sync        ",
            self.historical_span_style((179, 179, 179), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(33, 12, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(34, 12, 28, 1),
            "★ OpenCode · Go subscription",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(62, 12, 58, 1),
            "                                                          ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 13, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 13, 1, 1),
            " ",
            self.historical_span_style((0, 0, 0), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(3, 13, 1, 1),
            " ",
            self.historical_span_style((179, 179, 179), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(4, 13, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(5, 13, 1, 1),
            " ",
            self.historical_span_style((245, 158, 9), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(6, 13, 15, 1),
            " Grok Build    ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(21, 13, 12, 1),
            "api key     ",
            self.historical_span_style((179, 179, 179), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(33, 13, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(34, 13, 13, 1),
            "★ Grok · Team",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(47, 13, 73, 1),
            "                                                                         ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(Rect::new(0, 14, 120, 1), "                                                                                                                        ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(
            Rect::new(0, 15, 4, 1),
            "    ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(Rect::new(4, 15, 93, 1), "sync mirrors the host login · api key and oauth token take material from the registry account", self.historical_span_style((77, 77, 77), (0, 0, 0), false));
        ui.paint_str(
            Rect::new(97, 15, 23, 1),
            "                       ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 16, 4, 1),
            "    ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(4, 16, 79, 1),
            "ignore starts the agent without credentials and removes it from session pickers",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(83, 16, 37, 1),
            "                                     ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(Rect::new(0, 17, 120, 1), "                                                                                                                        ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(Rect::new(0, 18, 120, 1), "                                                                                                                        ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(Rect::new(0, 19, 120, 1), "                                                                                                                        ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(Rect::new(0, 20, 120, 1), "                                                                                                                        ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(Rect::new(0, 21, 120, 1), "                                                                                                                        ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(Rect::new(0, 22, 120, 1), "                                                                                                                        ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(Rect::new(0, 23, 120, 1), "                                                                                                                        ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(Rect::new(0, 24, 120, 1), "                                                                                                                        ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(Rect::new(0, 25, 120, 1), "                                                                                                                        ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(Rect::new(0, 26, 120, 1), "                                                                                                                        ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(Rect::new(0, 27, 120, 1), "                                                                                                                        ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(Rect::new(0, 28, 120, 1), "                                                                                                                        ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(Rect::new(0, 29, 120, 1), "                                                                                                                        ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(Rect::new(0, 30, 120, 1), "                                                                                                                        ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(Rect::new(0, 31, 120, 1), "                                                                                                                        ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(Rect::new(0, 32, 120, 1), "                                                                                                                        ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(Rect::new(0, 33, 120, 1), "                                                                                                                        ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(Rect::new(0, 34, 120, 1), "                                                                                                                        ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(Rect::new(0, 35, 120, 1), "                                                                                                                        ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(Rect::new(0, 36, 120, 1), "                                                                                                                        ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(Rect::new(0, 37, 97, 1), "                                                                                                 ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(
            Rect::new(97, 37, 1, 1),
            " ",
            self.historical_span_style((0, 0, 0), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(98, 37, 7, 1),
            "Cancel ",
            self.historical_span_style((179, 179, 179), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(105, 37, 3, 1),
            "   ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(108, 37, 1, 1),
            " ",
            self.historical_span_style((24, 24, 27), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(109, 37, 6, 1),
            "Save… ",
            self.historical_span_style((77, 77, 77), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(115, 37, 5, 1),
            "     ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(Rect::new(0, 38, 120, 1), "                                                                                                                        ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(
            Rect::new(0, 39, 13, 1),
            "             ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(13, 39, 5, 1),
            "Space",
            self.historical_span_style((255, 255, 255), (0, 0, 0), true),
        );
        ui.paint_str(
            Rect::new(18, 39, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(19, 39, 10, 1),
            "Cycle mode",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(29, 39, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(31, 39, 1, 1),
            "d",
            self.historical_span_style((255, 255, 255), (0, 0, 0), true),
        );
        ui.paint_str(
            Rect::new(32, 39, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(33, 39, 13, 1),
            "Reset to sync",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(46, 39, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(48, 39, 1, 1),
            "c",
            self.historical_span_style((255, 255, 255), (0, 0, 0), true),
        );
        ui.paint_str(
            Rect::new(49, 39, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(50, 39, 15, 1),
            "Manage accounts",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(65, 39, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(67, 39, 3, 1),
            "[ ]",
            self.historical_span_style((255, 255, 255), (0, 0, 0), true),
        );
        ui.paint_str(
            Rect::new(70, 39, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(71, 39, 10, 1),
            "Switch tab",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(81, 39, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(83, 39, 6, 1),
            "Ctrl+S",
            self.historical_span_style((255, 255, 255), (0, 0, 0), true),
        );
        ui.paint_str(
            Rect::new(89, 39, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(90, 39, 4, 1),
            "Save",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(94, 39, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(96, 39, 3, 1),
            "Esc",
            self.historical_span_style((255, 255, 255), (0, 0, 0), true),
        );
        ui.paint_str(
            Rect::new(99, 39, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(100, 39, 4, 1),
            "Back",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(104, 39, 16, 1),
            "                ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
    }

    pub(super) fn draw_historical_settings_trust_120_40(&self, ui: &mut Ui<'_>, area: Rect) {
        ui.fill(
            area,
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 0, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 0, 9, 1),
            " jackin❯ ",
            self.historical_span_style((25, 25, 28), (72, 224, 84), true),
        );
        ui.paint_str(
            Rect::new(10, 0, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(12, 0, 6, 1),
            " File ",
            self.historical_span_style((179, 179, 179), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(18, 0, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(19, 0, 4, 1),
            " Go ",
            self.historical_span_style((179, 179, 179), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(23, 0, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(24, 0, 6, 1),
            " Help ",
            self.historical_span_style((179, 179, 179), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(30, 0, 31, 1),
            "                               ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(61, 0, 25, 1),
            "Settings › global › Trust",
            self.historical_span_style((179, 179, 179), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(86, 0, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(88, 0, 20, 1),
            "inside the Construct",
            self.historical_span_style((179, 179, 179), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(108, 0, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(110, 0, 9, 1),
            "2 running",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(119, 0, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(Rect::new(0, 1, 120, 1), "                                                                                                                        ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(Rect::new(0, 2, 120, 1), "                                                                                                                        ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(
            Rect::new(0, 3, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 3, 10, 1),
            " General  ",
            self.historical_span_style((179, 179, 179), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(12, 3, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(13, 3, 9, 1),
            " Mounts  ",
            self.historical_span_style((179, 179, 179), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(22, 3, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(23, 3, 15, 1),
            " Environments  ",
            self.historical_span_style((179, 179, 179), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(38, 3, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(39, 3, 9, 1),
            " Agents  ",
            self.historical_span_style((179, 179, 179), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(48, 3, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(49, 3, 8, 1),
            " Trust  ",
            self.historical_span_style((255, 255, 255), (24, 24, 27), true),
        );
        ui.paint_str(
            Rect::new(57, 3, 63, 1),
            "                                                               ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 4, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 4, 47, 1),
            "───────────────────────────────────────────────",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(49, 4, 8, 1),
            "━━━━━━━━",
            self.historical_span_style((72, 224, 84), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(57, 4, 61, 1),
            "─────────────────────────────────────────────────────────────",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(118, 4, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(Rect::new(0, 5, 120, 1), "                                                                                                                        ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(
            Rect::new(0, 6, 4, 1),
            "    ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(4, 6, 12, 1),
            "Role sources",
            self.historical_span_style((179, 179, 179), (0, 0, 0), true),
        );
        ui.paint_str(
            Rect::new(16, 6, 77, 1),
            "                                                                             ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(93, 6, 23, 1),
            "3 trusted · 1 untrusted",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(116, 6, 4, 1),
            "    ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 7, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 7, 2, 1),
            "▎›",
            self.historical_span_style((72, 224, 84), (15, 46, 19), true),
        );
        ui.paint_str(
            Rect::new(4, 7, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (15, 46, 19), true),
        );
        ui.paint_str(
            Rect::new(5, 7, 1, 1),
            " ",
            self.historical_span_style((245, 158, 9), (15, 46, 19), true),
        );
        ui.paint_str(
            Rect::new(6, 7, 43, 1),
            " github.com/chainargos/roles               ",
            self.historical_span_style((255, 255, 255), (15, 46, 19), true),
        );
        ui.paint_str(
            Rect::new(49, 7, 5, 1),
            "git  ",
            self.historical_span_style((128, 128, 128), (15, 46, 19), true),
        );
        ui.paint_str(
            Rect::new(54, 7, 16, 1),
            " [✓] trusted    ",
            self.historical_span_style((255, 255, 255), (15, 46, 19), true),
        );
        ui.paint_str(
            Rect::new(70, 7, 7, 1),
            "4 roles",
            self.historical_span_style((77, 77, 77), (15, 46, 19), true),
        );
        ui.paint_str(
            Rect::new(77, 7, 41, 1),
            "                                         ",
            self.historical_span_style((255, 255, 255), (15, 46, 19), true),
        );
        ui.paint_str(
            Rect::new(118, 7, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 8, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 8, 1, 1),
            " ",
            self.historical_span_style((0, 0, 0), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(3, 8, 1, 1),
            " ",
            self.historical_span_style((179, 179, 179), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(4, 8, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(5, 8, 1, 1),
            " ",
            self.historical_span_style((245, 158, 9), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(6, 8, 43, 1),
            " github.com/acme-labs/roles-experimental   ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(49, 8, 5, 1),
            "git  ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(54, 8, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(55, 8, 13, 1),
            "[ ] untrusted",
            self.historical_span_style((245, 158, 9), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(68, 8, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(70, 8, 6, 1),
            "1 role",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(76, 8, 44, 1),
            "                                            ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 9, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 9, 1, 1),
            " ",
            self.historical_span_style((0, 0, 0), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(3, 9, 1, 1),
            " ",
            self.historical_span_style((179, 179, 179), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(4, 9, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(5, 9, 1, 1),
            " ",
            self.historical_span_style((245, 158, 9), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(6, 9, 43, 1),
            " ~/roles                                   ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(49, 9, 5, 1),
            "path ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(54, 9, 16, 1),
            " [✓] trusted    ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(70, 9, 6, 1),
            "1 role",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(76, 9, 44, 1),
            "                                            ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 10, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 10, 1, 1),
            " ",
            self.historical_span_style((0, 0, 0), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(3, 10, 1, 1),
            " ",
            self.historical_span_style((179, 179, 179), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(4, 10, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(5, 10, 1, 1),
            " ",
            self.historical_span_style((245, 158, 9), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(6, 10, 43, 1),
            " git@corp:infra/roles                      ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(49, 10, 5, 1),
            "git  ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(54, 10, 16, 1),
            " [✓] trusted    ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(70, 10, 7, 1),
            "2 roles",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(77, 10, 43, 1),
            "                                           ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(Rect::new(0, 11, 120, 1), "                                                                                                                        ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(
            Rect::new(0, 12, 4, 1),
            "    ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(4, 12, 66, 1),
            "An untrusted source blocks + Load role until it is trusted here or",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(70, 12, 50, 1),
            "                                                  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 13, 4, 1),
            "    ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(4, 13, 40, 1),
            "in the trust dialog that the load opens.",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(44, 13, 76, 1),
            "                                                                            ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(Rect::new(0, 14, 120, 1), "                                                                                                                        ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(Rect::new(0, 15, 120, 1), "                                                                                                                        ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(Rect::new(0, 16, 120, 1), "                                                                                                                        ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(Rect::new(0, 17, 120, 1), "                                                                                                                        ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(Rect::new(0, 18, 120, 1), "                                                                                                                        ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(Rect::new(0, 19, 120, 1), "                                                                                                                        ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(Rect::new(0, 20, 120, 1), "                                                                                                                        ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(Rect::new(0, 21, 120, 1), "                                                                                                                        ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(Rect::new(0, 22, 120, 1), "                                                                                                                        ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(Rect::new(0, 23, 120, 1), "                                                                                                                        ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(Rect::new(0, 24, 120, 1), "                                                                                                                        ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(Rect::new(0, 25, 120, 1), "                                                                                                                        ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(Rect::new(0, 26, 120, 1), "                                                                                                                        ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(Rect::new(0, 27, 120, 1), "                                                                                                                        ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(Rect::new(0, 28, 120, 1), "                                                                                                                        ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(Rect::new(0, 29, 120, 1), "                                                                                                                        ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(Rect::new(0, 30, 120, 1), "                                                                                                                        ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(Rect::new(0, 31, 120, 1), "                                                                                                                        ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(Rect::new(0, 32, 120, 1), "                                                                                                                        ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(Rect::new(0, 33, 120, 1), "                                                                                                                        ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(Rect::new(0, 34, 120, 1), "                                                                                                                        ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(Rect::new(0, 35, 120, 1), "                                                                                                                        ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(Rect::new(0, 36, 120, 1), "                                                                                                                        ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(Rect::new(0, 37, 97, 1), "                                                                                                 ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(
            Rect::new(97, 37, 1, 1),
            " ",
            self.historical_span_style((0, 0, 0), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(98, 37, 7, 1),
            "Cancel ",
            self.historical_span_style((179, 179, 179), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(105, 37, 3, 1),
            "   ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(108, 37, 1, 1),
            " ",
            self.historical_span_style((24, 24, 27), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(109, 37, 6, 1),
            "Save… ",
            self.historical_span_style((77, 77, 77), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(115, 37, 5, 1),
            "     ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(Rect::new(0, 38, 120, 1), "                                                                                                                        ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(
            Rect::new(0, 39, 23, 1),
            "                       ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(23, 39, 5, 1),
            "Space",
            self.historical_span_style((255, 255, 255), (0, 0, 0), true),
        );
        ui.paint_str(
            Rect::new(28, 39, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(29, 39, 12, 1),
            "Toggle trust",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(41, 39, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(43, 39, 1, 1),
            "o",
            self.historical_span_style((255, 255, 255), (0, 0, 0), true),
        );
        ui.paint_str(
            Rect::new(44, 39, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(45, 39, 11, 1),
            "Open source",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(56, 39, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(58, 39, 3, 1),
            "[ ]",
            self.historical_span_style((255, 255, 255), (0, 0, 0), true),
        );
        ui.paint_str(
            Rect::new(61, 39, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(62, 39, 10, 1),
            "Switch tab",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(72, 39, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(74, 39, 6, 1),
            "Ctrl+S",
            self.historical_span_style((255, 255, 255), (0, 0, 0), true),
        );
        ui.paint_str(
            Rect::new(80, 39, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(81, 39, 4, 1),
            "Save",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(85, 39, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(87, 39, 3, 1),
            "Esc",
            self.historical_span_style((255, 255, 255), (0, 0, 0), true),
        );
        ui.paint_str(
            Rect::new(90, 39, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(91, 39, 4, 1),
            "Back",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(95, 39, 25, 1),
            "                         ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
    }

    pub(super) fn draw_historical_settings_save_preview_120_40(&self, ui: &mut Ui<'_>, area: Rect) {
        ui.fill(
            area,
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 0, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 0, 9, 1),
            " jackin❯ ",
            self.historical_span_style((77, 77, 77), (39, 39, 42), false),
        );
        ui.paint_str(
            Rect::new(10, 0, 2, 1),
            "  ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(12, 0, 6, 1),
            " File ",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(18, 0, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(19, 0, 4, 1),
            " Go ",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(23, 0, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(24, 0, 6, 1),
            " Help ",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(30, 0, 18, 1),
            "                  ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(48, 0, 25, 1),
            "Settings › global › Trust",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(73, 0, 15, 1),
            "  • 2 changes  ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(88, 0, 20, 1),
            "inside the Construct",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(108, 0, 2, 1),
            "  ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(110, 0, 9, 1),
            "2 running",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(119, 0, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(Rect::new(0, 1, 120, 1), "                                                                                                                        ", self.historical_span_style((128, 128, 128), (0, 0, 0), false));
        ui.paint_str(Rect::new(0, 2, 120, 1), "                                                                                                                        ", self.historical_span_style((128, 128, 128), (0, 0, 0), false));
        ui.paint_str(
            Rect::new(0, 3, 2, 1),
            "  ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 3, 10, 1),
            " General  ",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(12, 3, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(13, 3, 9, 1),
            " Mounts  ",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(22, 3, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(23, 3, 15, 1),
            " Environments  ",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(38, 3, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(39, 3, 9, 1),
            " Agents  ",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(48, 3, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(49, 3, 10, 1),
            " Trust •  ",
            self.historical_span_style((128, 128, 128), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(59, 3, 61, 1),
            "                                                             ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 4, 2, 1),
            "  ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 4, 47, 1),
            "───────────────────────────────────────────────",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(49, 4, 10, 1),
            "━━━━━━━━━━",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(59, 4, 59, 1),
            "───────────────────────────────────────────────────────────",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(118, 4, 2, 1),
            "  ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(Rect::new(0, 5, 120, 1), "                                                                                                                        ", self.historical_span_style((128, 128, 128), (0, 0, 0), false));
        ui.paint_str(
            Rect::new(0, 6, 4, 1),
            "    ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(4, 6, 12, 1),
            "Role sources",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(16, 6, 77, 1),
            "                                                                             ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(93, 6, 23, 1),
            "3 trusted · 1 untrusted",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(116, 6, 4, 1),
            "    ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 7, 2, 1),
            "  ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 7, 1, 1),
            " ",
            self.historical_span_style((0, 0, 0), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(3, 7, 1, 1),
            " ",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(4, 7, 45, 1),
            " • github.com/chainargos/roles               ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(49, 7, 5, 1),
            "git  ",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(54, 7, 16, 1),
            " [ ] untrusted  ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(70, 7, 7, 1),
            "4 roles",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(77, 7, 43, 1),
            "                                           ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 8, 2, 1),
            "  ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 8, 1, 1),
            " ",
            self.historical_span_style((0, 0, 0), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(3, 8, 1, 1),
            "›",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(4, 8, 45, 1),
            " • github.com/acme-labs/roles-experimental   ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(49, 8, 5, 1),
            "git  ",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(54, 8, 16, 1),
            " [✓] trusted    ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(70, 8, 6, 1),
            "1 role",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(76, 8, 44, 1),
            "                                            ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 9, 2, 1),
            "  ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 9, 1, 1),
            " ",
            self.historical_span_style((0, 0, 0), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(3, 9, 1, 1),
            " ",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(4, 9, 45, 1),
            "   ~/roles                                   ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(49, 9, 5, 1),
            "path ",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(54, 9, 16, 1),
            " [✓] trusted    ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(70, 9, 6, 1),
            "1 role",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(76, 9, 44, 1),
            "                                            ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 10, 2, 1),
            "  ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 10, 1, 1),
            " ",
            self.historical_span_style((0, 0, 0), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(3, 10, 1, 1),
            " ",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(4, 10, 45, 1),
            "   git@corp:infra/roles                      ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(49, 10, 5, 1),
            "git  ",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(54, 10, 16, 1),
            " [✓] trusted    ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(70, 10, 7, 1),
            "2 roles",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(77, 10, 43, 1),
            "                                           ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(Rect::new(0, 11, 120, 1), "                                                                                                                        ", self.historical_span_style((128, 128, 128), (0, 0, 0), false));
        ui.paint_str(
            Rect::new(0, 12, 4, 1),
            "    ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(4, 12, 66, 1),
            "An untrusted source blocks + Load role until it is trusted here or",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(70, 12, 50, 1),
            "                                                  ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 13, 4, 1),
            "    ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(4, 13, 23, 1),
            "in the trust dialog tha",
            self.historical_span_style((38, 38, 38), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(27, 13, 66, 1),
            "╭────────────────────────────────────────────────────────────────╮",
            self.historical_span_style((77, 77, 77), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(93, 13, 27, 1),
            "                           ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 14, 27, 1),
            "                           ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(27, 14, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(28, 14, 64, 1),
            "                                                                ",
            self.historical_span_style((128, 128, 128), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(92, 14, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(93, 14, 27, 1),
            "                           ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 15, 27, 1),
            "                           ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(27, 15, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(28, 15, 2, 1),
            "  ",
            self.historical_span_style((128, 128, 128), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(30, 15, 13, 1),
            "Save settings",
            self.historical_span_style((255, 255, 255), (24, 24, 27), true),
        );
        ui.paint_str(
            Rect::new(43, 15, 49, 1),
            "                                                 ",
            self.historical_span_style((128, 128, 128), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(92, 15, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(93, 15, 27, 1),
            "                           ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 16, 27, 1),
            "                           ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(27, 16, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(28, 16, 64, 1),
            "                                                                ",
            self.historical_span_style((128, 128, 128), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(92, 16, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(93, 16, 27, 1),
            "                           ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 17, 27, 1),
            "                           ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(27, 17, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(28, 17, 11, 1),
            "  Scope    ",
            self.historical_span_style((128, 128, 128), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(39, 17, 37, 1),
            "global config · ~/.jackin/config.toml",
            self.historical_span_style((255, 255, 255), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(76, 17, 16, 1),
            "                ",
            self.historical_span_style((128, 128, 128), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(92, 17, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(93, 17, 27, 1),
            "                           ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 18, 27, 1),
            "                           ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(27, 18, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(28, 18, 11, 1),
            "  Changes  ",
            self.historical_span_style((128, 128, 128), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(39, 18, 9, 1),
            "2 changes",
            self.historical_span_style((255, 255, 255), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(48, 18, 44, 1),
            "                                            ",
            self.historical_span_style((128, 128, 128), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(92, 18, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(93, 18, 27, 1),
            "                           ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 19, 27, 1),
            "                           ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(27, 19, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(28, 19, 11, 1),
            "  Trust    ",
            self.historical_span_style((128, 128, 128), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(39, 19, 51, 1),
            "github.com/chainargos/roles → untrusted · github.c…",
            self.historical_span_style((255, 255, 255), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(90, 19, 2, 1),
            "  ",
            self.historical_span_style((128, 128, 128), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(92, 19, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(93, 19, 27, 1),
            "                           ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 20, 27, 1),
            "                           ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(27, 20, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(28, 20, 64, 1),
            "                                                                ",
            self.historical_span_style((128, 128, 128), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(92, 20, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(93, 20, 27, 1),
            "                           ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 21, 27, 1),
            "                           ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(27, 21, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(28, 21, 2, 1),
            "  ",
            self.historical_span_style((128, 128, 128), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(30, 21, 48, 1),
            "~ trust github.com/chainargos/roles true → false",
            self.historical_span_style((179, 179, 179), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(78, 21, 14, 1),
            "              ",
            self.historical_span_style((128, 128, 128), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(92, 21, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(93, 21, 27, 1),
            "                           ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 22, 27, 1),
            "                           ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(27, 22, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(28, 22, 2, 1),
            "  ",
            self.historical_span_style((128, 128, 128), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(30, 22, 60, 1),
            "~ trust github.com/acme-labs/roles-experimental false → true",
            self.historical_span_style((179, 179, 179), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(90, 22, 2, 1),
            "  ",
            self.historical_span_style((128, 128, 128), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(92, 22, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(93, 22, 27, 1),
            "                           ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 23, 27, 1),
            "                           ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(27, 23, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(28, 23, 64, 1),
            "                                                                ",
            self.historical_span_style((128, 128, 128), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(92, 23, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(93, 23, 27, 1),
            "                           ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 24, 27, 1),
            "                           ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(27, 24, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(28, 24, 47, 1),
            "                                               ",
            self.historical_span_style((128, 128, 128), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(75, 24, 1, 1),
            "▎",
            self.historical_span_style((72, 224, 84), (39, 39, 42), false),
        );
        ui.paint_str(
            Rect::new(76, 24, 7, 1),
            "Cancel ",
            self.historical_span_style((255, 255, 255), (39, 39, 42), true),
        );
        ui.paint_str(
            Rect::new(83, 24, 1, 1),
            " ",
            self.historical_span_style((128, 128, 128), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(84, 24, 1, 1),
            " ",
            self.historical_span_style((72, 224, 84), (72, 224, 84), false),
        );
        ui.paint_str(
            Rect::new(85, 24, 5, 1),
            "Save ",
            self.historical_span_style((25, 25, 28), (72, 224, 84), true),
        );
        ui.paint_str(
            Rect::new(90, 24, 2, 1),
            "  ",
            self.historical_span_style((128, 128, 128), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(92, 24, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(93, 24, 27, 1),
            "                           ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 25, 27, 1),
            "                           ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(27, 25, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(28, 25, 64, 1),
            "                                                                ",
            self.historical_span_style((128, 128, 128), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(92, 25, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(93, 25, 27, 1),
            "                           ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 26, 27, 1),
            "                           ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(27, 26, 66, 1),
            "╰────────────────────────────────────────────────────────────────╯",
            self.historical_span_style((77, 77, 77), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(93, 26, 27, 1),
            "                           ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(Rect::new(0, 27, 120, 1), "                                                                                                                        ", self.historical_span_style((128, 128, 128), (0, 0, 0), false));
        ui.paint_str(Rect::new(0, 28, 120, 1), "                                                                                                                        ", self.historical_span_style((128, 128, 128), (0, 0, 0), false));
        ui.paint_str(Rect::new(0, 29, 120, 1), "                                                                                                                        ", self.historical_span_style((128, 128, 128), (0, 0, 0), false));
        ui.paint_str(Rect::new(0, 30, 120, 1), "                                                                                                                        ", self.historical_span_style((128, 128, 128), (0, 0, 0), false));
        ui.paint_str(Rect::new(0, 31, 120, 1), "                                                                                                                        ", self.historical_span_style((128, 128, 128), (0, 0, 0), false));
        ui.paint_str(Rect::new(0, 32, 120, 1), "                                                                                                                        ", self.historical_span_style((128, 128, 128), (0, 0, 0), false));
        ui.paint_str(Rect::new(0, 33, 120, 1), "                                                                                                                        ", self.historical_span_style((128, 128, 128), (0, 0, 0), false));
        ui.paint_str(Rect::new(0, 34, 120, 1), "                                                                                                                        ", self.historical_span_style((128, 128, 128), (0, 0, 0), false));
        ui.paint_str(Rect::new(0, 35, 120, 1), "                                                                                                                        ", self.historical_span_style((128, 128, 128), (0, 0, 0), false));
        ui.paint_str(Rect::new(0, 36, 120, 1), "                                                                                                                        ", self.historical_span_style((128, 128, 128), (0, 0, 0), false));
        ui.paint_str(Rect::new(0, 37, 97, 1), "                                                                                                 ", self.historical_span_style((128, 128, 128), (0, 0, 0), false));
        ui.paint_str(
            Rect::new(97, 37, 1, 1),
            " ",
            self.historical_span_style((0, 0, 0), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(98, 37, 7, 1),
            "Cancel ",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(105, 37, 3, 1),
            "   ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(108, 37, 1, 1),
            " ",
            self.historical_span_style((39, 39, 42), (39, 39, 42), false),
        );
        ui.paint_str(
            Rect::new(109, 37, 6, 1),
            "Save… ",
            self.historical_span_style((77, 77, 77), (39, 39, 42), false),
        );
        ui.paint_str(
            Rect::new(115, 37, 5, 1),
            "     ",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(Rect::new(0, 38, 120, 1), "                                                                                                                        ", self.historical_span_style((128, 128, 128), (0, 0, 0), false));
        ui.paint_str(
            Rect::new(0, 39, 13, 1),
            "             ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(13, 39, 4, 1),
            "← → ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), true),
        );
        ui.paint_str(
            Rect::new(17, 39, 3, 1),
            "Cho",
            self.historical_span_style((128, 128, 128), (0, 0, 0), true),
        );
        ui.paint_str(
            Rect::new(20, 39, 3, 1),
            "ose",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(23, 39, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(25, 39, 5, 1),
            "Enter",
            self.historical_span_style((255, 255, 255), (0, 0, 0), true),
        );
        ui.paint_str(
            Rect::new(30, 39, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(31, 39, 4, 1),
            "Conf",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(35, 39, 1, 1),
            "i",
            self.historical_span_style((128, 128, 128), (0, 0, 0), true),
        );
        ui.paint_str(
            Rect::new(36, 39, 2, 1),
            "rm",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(38, 39, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(40, 39, 3, 1),
            "Esc",
            self.historical_span_style((255, 255, 255), (0, 0, 0), true),
        );
        ui.paint_str(
            Rect::new(43, 39, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(44, 39, 6, 1),
            "Cancel",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(50, 39, 4, 1),
            "    ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(54, 39, 65, 1),
            "github.com/acme-labs/roles-experimental · trusted · save to apply",
            self.historical_span_style((179, 179, 179), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(119, 39, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
    }

    pub(super) fn draw_historical_usage_overview_120_40(&self, ui: &mut Ui<'_>, area: Rect) {
        ui.fill(
            area,
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 0, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 0, 9, 1),
            " jackin❯ ",
            self.historical_span_style((25, 25, 28), (72, 224, 84), true),
        );
        ui.paint_str(
            Rect::new(10, 0, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(12, 0, 6, 1),
            " File ",
            self.historical_span_style((179, 179, 179), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(18, 0, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(19, 0, 4, 1),
            " Go ",
            self.historical_span_style((179, 179, 179), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(23, 0, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(24, 0, 6, 1),
            " Help ",
            self.historical_span_style((179, 179, 179), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(30, 0, 40, 1),
            "                                        ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(70, 0, 16, 1),
            "Usage › Overview",
            self.historical_span_style((179, 179, 179), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(86, 0, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(88, 0, 20, 1),
            "inside the Construct",
            self.historical_span_style((179, 179, 179), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(108, 0, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(110, 0, 9, 1),
            "2 running",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(119, 0, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(Rect::new(0, 1, 120, 1), "                                                                                                                        ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(
            Rect::new(0, 2, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 2, 2, 1),
            "╭─",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(3, 2, 19, 1),
            " Usage · read-only ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), true),
        );
        ui.paint_str(Rect::new(22, 2, 97, 1), "─────────────────────────────────────────────────────────────────────────────── ⠋ refreshing 1 ─╮", self.historical_span_style((77, 77, 77), (0, 0, 0), false));
        ui.paint_str(
            Rect::new(119, 2, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 3, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 3, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 3, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(4, 3, 2, 1),
            "▎›",
            self.historical_span_style((72, 224, 84), (15, 46, 19), true),
        );
        ui.paint_str(
            Rect::new(6, 3, 36, 1),
            " Overview                           ",
            self.historical_span_style((255, 255, 255), (15, 46, 19), true),
        );
        ui.paint_str(
            Rect::new(42, 3, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(44, 3, 8, 1),
            "Overview",
            self.historical_span_style((255, 255, 255), (0, 0, 0), true),
        );
        ui.paint_str(
            Rect::new(52, 3, 40, 1),
            "                                        ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(92, 3, 25, 1),
            "12 accounts · 8 providers",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(117, 3, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(118, 3, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(119, 3, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 4, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 4, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 4, 4, 1),
            "    ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(6, 4, 9, 1),
            "Anthropic",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(Rect::new(15, 4, 103, 1), "                                                                                                       ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(
            Rect::new(118, 4, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(119, 4, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 5, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 5, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 5, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(4, 5, 1, 1),
            " ",
            self.historical_span_style((0, 0, 0), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(5, 5, 39, 1),
            "    Personal  ★                        ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(44, 5, 73, 1),
            "Health       degraded · 4 warnings · 1 exhausted · 2 stale · 1 refreshin…",
            self.historical_span_style((228, 69, 69), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(117, 5, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(118, 5, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(119, 5, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 6, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 6, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 6, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(4, 6, 1, 1),
            " ",
            self.historical_span_style((0, 0, 0), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(5, 6, 4, 1),
            "    ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(9, 6, 23, 1),
            "Archived contractor la…",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(32, 6, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(33, 6, 8, 1),
            "disabled",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(41, 6, 77, 1),
            "   Freshness    5 of 11 current · broker projection 3 s ago                  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(118, 6, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(119, 6, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 7, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 7, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 7, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(4, 7, 1, 1),
            " ",
            self.historical_span_style((0, 0, 0), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(5, 7, 35, 1),
            "    Work                           ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(40, 7, 1, 1),
            "!",
            self.historical_span_style((228, 69, 69), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(41, 7, 77, 1),
            "   Registry     12 accounts · 11 enabled · 1 disabled · 8 providers          ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(118, 7, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(119, 7, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 8, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 8, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 8, 4, 1),
            "    ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(6, 8, 6, 1),
            "OpenAI",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(Rect::new(12, 8, 106, 1), "                                                                                                          ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(
            Rect::new(118, 8, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(119, 8, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 9, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 9, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 9, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(4, 9, 1, 1),
            " ",
            self.historical_span_style((0, 0, 0), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(5, 9, 39, 1),
            "    Primary  ★                         ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(44, 9, 56, 1),
            "Provider     Accounts  Worst window               Status",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(100, 9, 18, 1),
            "                  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(118, 9, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(119, 9, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 10, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 10, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 10, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(4, 10, 1, 1),
            " ",
            self.historical_span_style((0, 0, 0), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(5, 10, 35, 1),
            "    Experiments                    ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(40, 10, 1, 1),
            "⠋",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(41, 10, 3, 1),
            "   ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(44, 10, 61, 1),
            "Anthropic    2         Weekly · Opus 100%         ! exhausted",
            self.historical_span_style((228, 69, 69), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(105, 10, 13, 1),
            "             ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(118, 10, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(119, 10, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 11, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 11, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 11, 4, 1),
            "    ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(6, 11, 3, 1),
            "Amp",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(Rect::new(9, 11, 109, 1), "                                   OpenAI       2         Weekly · 7-day 59%         current                 ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(
            Rect::new(118, 11, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(119, 11, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 12, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 12, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 12, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(4, 12, 1, 1),
            " ",
            self.historical_span_style((0, 0, 0), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(5, 12, 35, 1),
            "    discovered                     ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(40, 12, 1, 1),
            "▲",
            self.historical_span_style((245, 158, 9), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(41, 12, 3, 1),
            "   ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(44, 12, 59, 1),
            "Amp          1         Amp Free · daily 91%       ▲ warning",
            self.historical_span_style((245, 158, 9), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(103, 12, 15, 1),
            "               ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(118, 12, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(119, 12, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 13, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 13, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 13, 4, 1),
            "    ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(6, 13, 3, 1),
            "xAI",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(Rect::new(9, 13, 109, 1), "                                   xAI          1         On-demand usage 100%       current                 ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(
            Rect::new(118, 13, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(119, 13, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 14, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 14, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 14, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(4, 14, 1, 1),
            " ",
            self.historical_span_style((0, 0, 0), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(5, 14, 39, 1),
            "    Team  ★                            ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(44, 14, 59, 1),
            "Z.AI         1         Weekly 76%                 ▲ warning",
            self.historical_span_style((245, 158, 9), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(103, 14, 15, 1),
            "               ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(118, 14, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(119, 14, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 15, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 15, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 15, 4, 1),
            "    ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(6, 15, 4, 1),
            "Z.AI",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(10, 15, 34, 1),
            "                                  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(44, 15, 62, 1),
            "Kimi         1         —                          needs secret",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(106, 15, 12, 1),
            "            ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(118, 15, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(119, 15, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 16, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 16, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 16, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(4, 16, 1, 1),
            " ",
            self.historical_span_style((0, 0, 0), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(5, 16, 35, 1),
            "    discovered                     ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(40, 16, 1, 1),
            "▲",
            self.historical_span_style((245, 158, 9), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(41, 16, 3, 1),
            "   ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(44, 16, 72, 1),
            "MiniMax      1         MiniMax-M2 · Weekly 47%    ! provider unavailable",
            self.historical_span_style((228, 69, 69), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(116, 16, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(118, 16, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(119, 16, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 17, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 17, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 17, 4, 1),
            "    ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(6, 17, 4, 1),
            "Kimi",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(10, 17, 34, 1),
            "                                  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(44, 17, 64, 1),
            "OpenCode     2         Rolling 57%                ! rate limited",
            self.historical_span_style((228, 69, 69), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(108, 17, 10, 1),
            "          ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(118, 17, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(119, 17, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 18, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 18, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 18, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(4, 18, 1, 1),
            " ",
            self.historical_span_style((0, 0, 0), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(5, 18, 22, 1),
            "    discovered        ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(27, 18, 1, 1),
            "!",
            self.historical_span_style((228, 69, 69), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(28, 18, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(29, 18, 12, 1),
            "needs secret",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(41, 18, 3, 1),
            "   ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(44, 18, 70, 1),
            "—            —         —                          unsupported sentinel",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(114, 18, 4, 1),
            "    ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(118, 18, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(119, 18, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 19, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 19, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 19, 4, 1),
            "    ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(6, 19, 7, 1),
            "MiniMax",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(Rect::new(13, 19, 105, 1), "                                                                                                         ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(
            Rect::new(118, 19, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(119, 19, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 20, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 20, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 20, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(4, 20, 1, 1),
            " ",
            self.historical_span_style((0, 0, 0), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(5, 20, 23, 1),
            "    discovered         ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(28, 20, 1, 1),
            "!",
            self.historical_span_style((228, 69, 69), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(29, 20, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(30, 20, 11, 1),
            "unavailable",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(41, 20, 3, 1),
            "   ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(44, 20, 73, 1),
            "Unresolved   Amp · ~/.config/amp/secrets.json (presence only) — not an a…",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(117, 20, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(118, 20, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(119, 20, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 21, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 21, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 21, 4, 1),
            "    ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(6, 21, 8, 1),
            "OpenCode",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(14, 21, 30, 1),
            "                              ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(44, 21, 73, 1),
            "Unresolved   Kimi · ~/.kimi (presence only) — not an authenticated accou…",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(117, 21, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(118, 21, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(119, 21, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 22, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 22, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 22, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(4, 22, 1, 1),
            " ",
            self.historical_span_style((0, 0, 0), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(5, 22, 35, 1),
            "    Go subscription  ★             ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(40, 22, 1, 1),
            "!",
            self.historical_span_style((228, 69, 69), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(41, 22, 77, 1),
            "                                                                             ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(118, 22, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(119, 22, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 23, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 23, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 23, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(4, 23, 1, 1),
            " ",
            self.historical_span_style((0, 0, 0), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(5, 23, 25, 1),
            "    ci-bot               ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(30, 23, 11, 1),
            "unsupported",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(41, 23, 3, 1),
            "   ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(44, 23, 73, 1),
            "Rollups sum identical windows and units only; a provider with mixed wind…",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(117, 23, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(118, 23, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(119, 23, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 24, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 24, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 24, 4, 1),
            "    ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(6, 24, 5, 1),
            "Usage",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(Rect::new(11, 24, 107, 1), "                                                                                                           ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(
            Rect::new(118, 24, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(119, 24, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 25, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 25, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 25, 42, 1),
            "                                          ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(44, 25, 73, 1),
            "Comparable   Anthropic · Session · 5-hour · 2 accounts · 24–62% remaining",
            self.historical_span_style((179, 179, 179), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(117, 25, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(118, 25, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(119, 25, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 26, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 26, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 26, 42, 1),
            "                                          ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(44, 26, 73, 1),
            "Comparable   Anthropic · Weekly · all models · 2 accounts · 12–67% remai…",
            self.historical_span_style((179, 179, 179), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(117, 26, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(118, 26, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(119, 26, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 27, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 27, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 27, 42, 1),
            "                                          ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(44, 27, 69, 1),
            "Comparable   Anthropic · Weekly · Opus · 2 accounts · 0–46% remaining",
            self.historical_span_style((179, 179, 179), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(113, 27, 5, 1),
            "     ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(118, 27, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(119, 27, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 28, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 28, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 28, 42, 1),
            "                                          ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(44, 28, 61, 1),
            "Comparable   OpenAI · Credits · 2 accounts · 76–82% remaining",
            self.historical_span_style((179, 179, 179), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(105, 28, 13, 1),
            "             ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(118, 28, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(119, 28, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 29, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 29, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 29, 42, 1),
            "                                          ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(44, 29, 70, 1),
            "Comparable   OpenAI · Session · 5-hour · 2 accounts · 88–96% remaining",
            self.historical_span_style((179, 179, 179), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(114, 29, 4, 1),
            "    ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(118, 29, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(119, 29, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 30, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 30, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 30, 42, 1),
            "                                          ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(44, 30, 68, 1),
            "Comparable   OpenAI · Weekly · 7-day · 2 accounts · 41–88% remaining",
            self.historical_span_style((179, 179, 179), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(112, 30, 6, 1),
            "      ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(118, 30, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(119, 30, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 31, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 31, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(Rect::new(2, 31, 116, 1), "                                                                                                                    ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(
            Rect::new(118, 31, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(119, 31, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 32, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 32, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(Rect::new(2, 32, 116, 1), "                                                                                                                    ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(
            Rect::new(118, 32, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(119, 32, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 33, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 33, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(Rect::new(2, 33, 116, 1), "                                                                                                                    ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(
            Rect::new(118, 33, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(119, 33, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 34, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 34, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(Rect::new(2, 34, 116, 1), "                                                                                                                    ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(
            Rect::new(118, 34, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(119, 34, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 35, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 35, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(Rect::new(2, 35, 116, 1), "                                                                                                                    ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(
            Rect::new(118, 35, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(119, 35, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 36, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 36, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(Rect::new(2, 36, 116, 1), "                                                                                                                    ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(
            Rect::new(118, 36, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(119, 36, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 37, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(Rect::new(1, 37, 118, 1), "╰────────────────────────────────────────────────────────────────────────────────────────────────────────────────────╯", self.historical_span_style((77, 77, 77), (0, 0, 0), false));
        ui.paint_str(
            Rect::new(119, 37, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(Rect::new(0, 38, 120, 1), "                                                                                                                        ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(
            Rect::new(0, 39, 26, 1),
            "                          ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(26, 39, 2, 1),
            "↑↓",
            self.historical_span_style((255, 255, 255), (0, 0, 0), true),
        );
        ui.paint_str(
            Rect::new(28, 39, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(29, 39, 4, 1),
            "Move",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(33, 39, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(35, 39, 5, 1),
            "Enter",
            self.historical_span_style((255, 255, 255), (0, 0, 0), true),
        );
        ui.paint_str(
            Rect::new(40, 39, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(41, 39, 6, 1),
            "Detail",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(47, 39, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(49, 39, 1, 1),
            "r",
            self.historical_span_style((255, 255, 255), (0, 0, 0), true),
        );
        ui.paint_str(
            Rect::new(50, 39, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(51, 39, 7, 1),
            "Refresh",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(58, 39, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(60, 39, 1, 1),
            "m",
            self.historical_span_style((255, 255, 255), (0, 0, 0), true),
        );
        ui.paint_str(
            Rect::new(61, 39, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(62, 39, 18, 1),
            "Manage in Accounts",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(80, 39, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(82, 39, 3, 1),
            "Esc",
            self.historical_span_style((255, 255, 255), (0, 0, 0), true),
        );
        ui.paint_str(
            Rect::new(85, 39, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(86, 39, 5, 1),
            "Close",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(91, 39, 29, 1),
            "                             ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
    }

    pub(super) fn draw_historical_usage_detail_120_40(&self, ui: &mut Ui<'_>, area: Rect) {
        ui.fill(
            area,
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 0, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 0, 9, 1),
            " jackin❯ ",
            self.historical_span_style((25, 25, 28), (72, 224, 84), true),
        );
        ui.paint_str(
            Rect::new(10, 0, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(12, 0, 6, 1),
            " File ",
            self.historical_span_style((179, 179, 179), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(18, 0, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(19, 0, 4, 1),
            " Go ",
            self.historical_span_style((179, 179, 179), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(23, 0, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(24, 0, 6, 1),
            " Help ",
            self.historical_span_style((179, 179, 179), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(30, 0, 31, 1),
            "                               ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(61, 0, 25, 1),
            "Usage › Claude › Personal",
            self.historical_span_style((179, 179, 179), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(86, 0, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(88, 0, 20, 1),
            "inside the Construct",
            self.historical_span_style((179, 179, 179), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(108, 0, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(110, 0, 9, 1),
            "2 running",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(119, 0, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(Rect::new(0, 1, 120, 1), "                                                                                                                        ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(
            Rect::new(0, 2, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 2, 2, 1),
            "╭─",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(3, 2, 19, 1),
            " Usage · read-only ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), true),
        );
        ui.paint_str(Rect::new(22, 2, 97, 1), "─────────────────────────────────────────────────────────────────────────────── ⠋ refreshing 1 ─╮", self.historical_span_style((77, 77, 77), (0, 0, 0), false));
        ui.paint_str(
            Rect::new(119, 2, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 3, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 3, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 3, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(4, 3, 1, 1),
            " ",
            self.historical_span_style((0, 0, 0), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(5, 3, 39, 1),
            "  Overview                             ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(44, 3, 17, 1),
            "Claude · Personal",
            self.historical_span_style((255, 255, 255), (0, 0, 0), true),
        );
        ui.paint_str(
            Rect::new(61, 3, 54, 1),
            "                                                      ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(115, 3, 2, 1),
            "ok",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(117, 3, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(118, 3, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(119, 3, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 4, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 4, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 4, 4, 1),
            "    ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(6, 4, 9, 1),
            "Anthropic",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(Rect::new(15, 4, 103, 1), "                                                                                                       ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(
            Rect::new(118, 4, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(119, 4, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 5, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 5, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 5, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(4, 5, 1, 1),
            " ",
            self.historical_span_style((0, 0, 0), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(5, 5, 1, 1),
            "›",
            self.historical_span_style((179, 179, 179), (0, 0, 0), false),
        );
        ui.paint_str(Rect::new(6, 5, 112, 1), "   Personal  ★                        Provider     Anthropic / Claude · surface Claude                          ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(
            Rect::new(118, 5, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(119, 5, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 6, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 6, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 6, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(4, 6, 1, 1),
            " ",
            self.historical_span_style((0, 0, 0), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(5, 6, 4, 1),
            "    ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(9, 6, 23, 1),
            "Archived contractor la…",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(32, 6, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(33, 6, 8, 1),
            "disabled",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(41, 6, 77, 1),
            "   Account      alexey@donbeave.dev · Max 5x                                 ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(118, 6, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(119, 6, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 7, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 7, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 7, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(4, 7, 1, 1),
            " ",
            self.historical_span_style((0, 0, 0), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(5, 7, 35, 1),
            "    Work                           ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(40, 7, 1, 1),
            "!",
            self.historical_span_style((228, 69, 69), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(41, 7, 77, 1),
            "   Credential   Local folder · ~/.claude                                     ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(118, 7, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(119, 7, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 8, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 8, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 8, 4, 1),
            "    ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(6, 8, 6, 1),
            "OpenAI",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(Rect::new(12, 8, 106, 1), "                                Status       available · current · refreshed 4 min ago                    ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(
            Rect::new(118, 8, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(119, 8, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 9, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 9, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 9, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(4, 9, 1, 1),
            " ",
            self.historical_span_style((0, 0, 0), (0, 0, 0), false),
        );
        ui.paint_str(Rect::new(5, 9, 113, 1), "    Primary  ★                                                                                                   ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(
            Rect::new(118, 9, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(119, 9, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 10, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 10, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 10, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(4, 10, 1, 1),
            " ",
            self.historical_span_style((0, 0, 0), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(5, 10, 35, 1),
            "    Experiments                    ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(40, 10, 1, 1),
            "⠋",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(41, 10, 3, 1),
            "   ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(44, 10, 6, 1),
            "Limits",
            self.historical_span_style((179, 179, 179), (0, 0, 0), true),
        );
        ui.paint_str(
            Rect::new(50, 10, 68, 1),
            "                                                                    ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(118, 10, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(119, 10, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 11, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 11, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 11, 4, 1),
            "    ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(6, 11, 3, 1),
            "Amp",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(9, 11, 51, 1),
            "                                     Session · 5-… ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(60, 11, 11, 1),
            "  38%      ",
            self.historical_span_style((0, 0, 0), (179, 179, 179), true),
        );
        ui.paint_str(
            Rect::new(71, 11, 17, 1),
            "                 ",
            self.historical_span_style((255, 255, 255), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(88, 11, 2, 1),
            "  ",
            self.historical_span_style((179, 179, 179), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(90, 11, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(92, 11, 20, 1),
            "resets in 3 h 12 min",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(112, 11, 6, 1),
            "      ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(118, 11, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(119, 11, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 12, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 12, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 12, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(4, 12, 1, 1),
            " ",
            self.historical_span_style((0, 0, 0), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(5, 12, 35, 1),
            "    discovered                     ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(40, 12, 1, 1),
            "▲",
            self.historical_span_style((245, 158, 9), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(41, 12, 19, 1),
            "     Weekly · all… ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(60, 12, 9, 1),
            "  33%    ",
            self.historical_span_style((0, 0, 0), (179, 179, 179), true),
        );
        ui.paint_str(
            Rect::new(69, 12, 19, 1),
            "                   ",
            self.historical_span_style((255, 255, 255), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(88, 12, 2, 1),
            "  ",
            self.historical_span_style((179, 179, 179), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(90, 12, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(92, 12, 16, 1),
            "resets Sun 19:14",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(108, 12, 10, 1),
            "          ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(118, 12, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(119, 12, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 13, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 13, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 13, 4, 1),
            "    ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(6, 13, 3, 1),
            "xAI",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(9, 13, 51, 1),
            "                                     Weekly · Son… ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(60, 13, 6, 1),
            "  21% ",
            self.historical_span_style((0, 0, 0), (179, 179, 179), true),
        );
        ui.paint_str(
            Rect::new(66, 13, 22, 1),
            "                      ",
            self.historical_span_style((255, 255, 255), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(88, 13, 2, 1),
            "  ",
            self.historical_span_style((179, 179, 179), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(90, 13, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(92, 13, 16, 1),
            "resets Sun 19:14",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(108, 13, 10, 1),
            "          ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(118, 13, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(119, 13, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 14, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 14, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 14, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(4, 14, 1, 1),
            " ",
            self.historical_span_style((0, 0, 0), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(5, 14, 55, 1),
            "    Team  ★                              Weekly · Opus ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(60, 14, 15, 1),
            "  54%          ",
            self.historical_span_style((0, 0, 0), (179, 179, 179), true),
        );
        ui.paint_str(
            Rect::new(75, 14, 13, 1),
            "             ",
            self.historical_span_style((255, 255, 255), (24, 24, 27), false),
        );
        ui.paint_str(
            Rect::new(88, 14, 2, 1),
            "  ",
            self.historical_span_style((179, 179, 179), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(90, 14, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(92, 14, 16, 1),
            "resets Sun 19:14",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(108, 14, 10, 1),
            "          ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(118, 14, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(119, 14, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 15, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 15, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 15, 4, 1),
            "    ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(6, 15, 4, 1),
            "Z.AI",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(Rect::new(10, 15, 108, 1), "                                                                                                            ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(
            Rect::new(118, 15, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(119, 15, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 16, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 16, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 16, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(4, 16, 1, 1),
            " ",
            self.historical_span_style((0, 0, 0), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(5, 16, 35, 1),
            "    discovered                     ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(40, 16, 1, 1),
            "▲",
            self.historical_span_style((245, 158, 9), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(41, 16, 77, 1),
            "                                                                             ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(118, 16, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(119, 16, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 17, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 17, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 17, 4, 1),
            "    ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(6, 17, 4, 1),
            "Kimi",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(Rect::new(10, 17, 108, 1), "                                                                                                            ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(
            Rect::new(118, 17, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(119, 17, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 18, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 18, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 18, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(4, 18, 1, 1),
            " ",
            self.historical_span_style((0, 0, 0), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(5, 18, 22, 1),
            "    discovered        ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(27, 18, 1, 1),
            "!",
            self.historical_span_style((228, 69, 69), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(28, 18, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(29, 18, 12, 1),
            "needs secret",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(41, 18, 77, 1),
            "                                                                             ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(118, 18, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(119, 18, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 19, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 19, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 19, 4, 1),
            "    ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(6, 19, 7, 1),
            "MiniMax",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(Rect::new(13, 19, 105, 1), "                                                                                                         ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(
            Rect::new(118, 19, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(119, 19, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 20, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 20, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 20, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(4, 20, 1, 1),
            " ",
            self.historical_span_style((0, 0, 0), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(5, 20, 23, 1),
            "    discovered         ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(28, 20, 1, 1),
            "!",
            self.historical_span_style((228, 69, 69), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(29, 20, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(30, 20, 11, 1),
            "unavailable",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(41, 20, 77, 1),
            "                                                                             ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(118, 20, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(119, 20, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 21, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 21, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 21, 4, 1),
            "    ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(6, 21, 8, 1),
            "OpenCode",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(Rect::new(14, 21, 104, 1), "                                                                                                        ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(
            Rect::new(118, 21, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(119, 21, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 22, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 22, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 22, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(4, 22, 1, 1),
            " ",
            self.historical_span_style((0, 0, 0), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(5, 22, 35, 1),
            "    Go subscription  ★             ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(40, 22, 1, 1),
            "!",
            self.historical_span_style((228, 69, 69), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(41, 22, 77, 1),
            "                                                                             ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(118, 22, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(119, 22, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 23, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 23, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 23, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(4, 23, 1, 1),
            " ",
            self.historical_span_style((0, 0, 0), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(5, 23, 25, 1),
            "    ci-bot               ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(30, 23, 11, 1),
            "unsupported",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(41, 23, 77, 1),
            "                                                                             ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(118, 23, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(119, 23, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 24, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 24, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(2, 24, 4, 1),
            "    ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(6, 24, 5, 1),
            "Usage",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(Rect::new(11, 24, 107, 1), "                                                                                                           ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(
            Rect::new(118, 24, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(119, 24, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 25, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 25, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(Rect::new(2, 25, 116, 1), "                                                                                                                    ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(
            Rect::new(118, 25, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(119, 25, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 26, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 26, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(Rect::new(2, 26, 116, 1), "                                                                                                                    ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(
            Rect::new(118, 26, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(119, 26, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 27, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 27, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(Rect::new(2, 27, 116, 1), "                                                                                                                    ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(
            Rect::new(118, 27, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(119, 27, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 28, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 28, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(Rect::new(2, 28, 116, 1), "                                                                                                                    ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(
            Rect::new(118, 28, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(119, 28, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 29, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 29, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(Rect::new(2, 29, 116, 1), "                                                                                                                    ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(
            Rect::new(118, 29, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(119, 29, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 30, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 30, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(Rect::new(2, 30, 116, 1), "                                                                                                                    ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(
            Rect::new(118, 30, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(119, 30, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 31, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 31, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(Rect::new(2, 31, 116, 1), "                                                                                                                    ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(
            Rect::new(118, 31, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(119, 31, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 32, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 32, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(Rect::new(2, 32, 116, 1), "                                                                                                                    ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(
            Rect::new(118, 32, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(119, 32, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 33, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 33, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(Rect::new(2, 33, 116, 1), "                                                                                                                    ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(
            Rect::new(118, 33, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(119, 33, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 34, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 34, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(Rect::new(2, 34, 116, 1), "                                                                                                                    ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(
            Rect::new(118, 34, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(119, 34, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 35, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 35, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(Rect::new(2, 35, 116, 1), "                                                                                                                    ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(
            Rect::new(118, 35, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(119, 35, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 36, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(1, 36, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(Rect::new(2, 36, 116, 1), "                                                                                                                    ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(
            Rect::new(118, 36, 1, 1),
            "│",
            self.historical_span_style((77, 77, 77), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(119, 36, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(0, 37, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(Rect::new(1, 37, 118, 1), "╰────────────────────────────────────────────────────────────────────────────────────────────────────────────────────╯", self.historical_span_style((77, 77, 77), (0, 0, 0), false));
        ui.paint_str(
            Rect::new(119, 37, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(Rect::new(0, 38, 120, 1), "                                                                                                                        ", self.historical_span_style((255, 255, 255), (0, 0, 0), false));
        ui.paint_str(
            Rect::new(0, 39, 29, 1),
            "                             ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(29, 39, 2, 1),
            "↑↓",
            self.historical_span_style((255, 255, 255), (0, 0, 0), true),
        );
        ui.paint_str(
            Rect::new(31, 39, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(32, 39, 6, 1),
            "Scroll",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(38, 39, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(40, 39, 1, 1),
            "r",
            self.historical_span_style((255, 255, 255), (0, 0, 0), true),
        );
        ui.paint_str(
            Rect::new(41, 39, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(42, 39, 7, 1),
            "Refresh",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(49, 39, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(51, 39, 1, 1),
            "m",
            self.historical_span_style((255, 255, 255), (0, 0, 0), true),
        );
        ui.paint_str(
            Rect::new(52, 39, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(53, 39, 18, 1),
            "Manage in Accounts",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(71, 39, 2, 1),
            "  ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(73, 39, 3, 1),
            "Esc",
            self.historical_span_style((255, 255, 255), (0, 0, 0), true),
        );
        ui.paint_str(
            Rect::new(76, 39, 1, 1),
            " ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(77, 39, 12, 1),
            "Back to list",
            self.historical_span_style((128, 128, 128), (0, 0, 0), false),
        );
        ui.paint_str(
            Rect::new(89, 39, 31, 1),
            "                               ",
            self.historical_span_style((255, 255, 255), (0, 0, 0), false),
        );
    }
}
