# Universal launcher patterns

These are abstract product patterns. Terms such as root, result, panel, and list
describe relationships and decisions, not required UI components, layout, or
visual treatment.

## One root surface

One entry point should expose applications, commands, files, links, scripts,
resources, and contextual actions without requiring category selection first.

Product lesson: let users express intent before choosing a domain.

## Useful empty state

Opening the launcher should already provide value through a small combination of
contextual suggestions, pinned items, recent actions, and stable exploration
paths.

Product lesson: empty search is a recommendation surface, not an empty box or
complete catalog.

## Mixed result types

Search can combine resources and actions when each result clearly communicates
its type, target, and primary behavior.

Product lesson: users should not switch between separate search engines for
tasks, files, services, and tools.

## Primary action and contextual alternatives

Every result should have one obvious primary action. Related operations should
remain coherently accessible rather than becoming unrelated duplicates.

Product lesson: fast execution and broad discoverability can coexist.

## Fuzzy discovery and deterministic shortcuts

Fuzzy matching helps users find vaguely remembered items. Exact aliases and
direct shortcuts create stable muscle memory. Learned ranking may resolve
ambiguity but must not defeat an exact shortcut.

Product lesson: adaptation needs deterministic escape hatches.

## Native treatment for contributed capabilities

Personal scripts, team workflows, installed integrations, and built-in actions
should share search, metadata, arguments, previews, and safety conventions.

Product lesson: provenance may differ while interaction remains coherent.

## User control over ranking

Users should be able to pin, alias, hide, demote, restore, and reset. Ranking
should explain itself when asked.

Product lesson: personalization without correction feels arbitrary.

## Context changes available actions

Actions should reflect the selected resource and current state. A project,
service, process, or file should expose different operations without changing
the launcher's overall interaction language.

Product lesson: stable interaction can sit above dynamic capabilities.

## Consistent navigation

Text input filters, navigation changes focus, direct invocation performs the
primary action, a separate interaction exposes alternatives, and a back action
returns to prior context. Exact controls and their representation remain open.

Product lesson: speed comes from transferable motor memory, not many hidden
shortcuts.

## Holla interpretation

Desktop launchers often begin globally. Holla should begin locally and expand:

1. current folder;
2. surrounding project;
3. workspace or neighborhood;
4. host;
5. personal and team capabilities.

The terminal's current path, session, and host provide richer ambient context
than a generic global launcher. Holla should build around that advantage rather
than imitate a desktop layout.
