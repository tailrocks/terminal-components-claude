// Historical settings visual frames for 120x40 truecolor conformance.
// The accounts frames were removed once the live AccountsScreen matched
// the visual-baseline matrix.
use super::App;
use termrock::{Rect, Ui};

impl App {
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
}
