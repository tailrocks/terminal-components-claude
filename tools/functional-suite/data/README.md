# Functional case data

The exporter creates every case directory listed in its fixed directory inventory. The runner copies each directory into the case output before it starts the application.

The `cases.toml` inventory names each fixture directory. The first30-v1 cases use these fixed directories:

- `first30-v1/showcase.buttons`
- `first30-v1/holla.first_use`
- `first30-v1/jackin.settings`
- `first30-v1/tablepro.connections`

The separate Holla frame-one variant uses `first30-v1/holla.first_use.frame1`. It uses the same fixed input data as the primary Holla case.

The Showcase help journey uses `showcase-help/80x24-v1`. The Holla action journey uses `holla-first-use/action-identity-v1`. Keep both journeys as requirements.

The older `first/` fixture paths remain in the exporter inventory for historical continuity. The first30-v1 selected lane uses its versioned paths. Both branches use the same exported case data. Do not add branch-specific files.
