# ScreenAlign geometric center

`ScreenAlign::GeometricCenter` provides true vertical midpoint placement for
layers that need it. `ScreenAlign::Center` keeps its historical upper-third
placement unchanged, including its existing geometry assertion, so current
screens retain their output. Horizontal placement uses the existing centered
geometry. Fixed sizes still clamp to the viewport before placement.

This public library option is not demonstrated by the showcase in this
change. Adding a showcase state would alter baseline output. Showcase coverage
remains a required follow-up when the baseline permits a demonstration.

Focused geometry coverage checks offset viewports, oversize fixed layers,
small viewports, and unchanged legacy center placement.
