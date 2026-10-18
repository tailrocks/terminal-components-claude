# Terminal workflow patterns

These notes describe useful capabilities and workflow relationships only. They
do not prescribe screens, components, layout, styling, or visual translation.

## General-purpose fuzzy finding

A fuzzy finder can filter arbitrary terminal data, preview selection, and attach
key-driven actions.

Holla lesson: fuzzy filtering is a useful primitive, but it does not provide
context discovery, recommendation policy, scope, or safety by itself.

## Search channels

One interaction can search different domains such as files, text, Git projects,
environment values, containers, and processes. Each domain may define its own
preview and actions.

Holla lesson: diverse capability domains can share one interaction language when
provenance and scope stay visible.

## Interactive command knowledge

Searchable command recipes and dynamic argument suggestions help users execute
complex commands without memorizing syntax.

Holla lesson: guidance and parameter collection can reduce recall while keeping
final behavior transparent.

## Contextual shell history

History becomes more useful when filtered by host, session, exact directory, or
current project. Frequency and recency should operate within those scopes.

Holla lesson: “used here” carries stronger meaning than global frequency.

## Directory frecency

Frequently and recently visited paths can become reachable with a few stable
characters.

Holla lesson: learned behavior feels predictable when applied to a clear object
type and combined with deterministic input.

## Resource-oriented navigation

Directory trees work well when browsing, filtering, previewing, navigation, and
actions operate on the same selected resource.

Holla lesson: resources plus contextual actions are often clearer than flat
command lists.

## Specialist system monitoring

System monitoring benefits from persistent, information-dense views for CPU,
memory, network, disks, and processes.

Holla lesson: detect relevance and hand off to a focused monitor instead of
recreating its entire experience.

## Progressive disk analysis

Disk investigation benefits from fast largest-first results, directory
drill-down, selection, and deliberate cleanup.

Holla lesson: observation, review, and deletion should be separate stages.

## Destructive maintenance workflows

Some users regularly stop and remove every container, clear images, prune
networks and volumes, or reset an entire local Docker environment. These are not
exceptional commands merely because they are destructive.

Holla lesson: expose them when query and context match. Learn their contextual
frequency. Keep broad scope visible, preview every stage, confirm deliberately,
then report each stage separately.

For complete cleanup, confirmation should be target-bound. First review the
resolved Docker plan and host. Then type a phrase such as `REMOVE ALL DOCKER DATA
ON devbox`. This preserves fast discovery without turning fast selection into
blind execution.

## Coordination opportunity

Individual terminal tools solve search, history, navigation, monitoring, and
disk analysis well. Holla's distinct role is coordination:

- understand current context;
- decide what matters now;
- explain why;
- preserve scope and safety;
- launch the strongest available workflow.

## Hierarchical project discovery

A terminal launcher may begin at an exact folder while discovering meaningful
ancestors and bounded descendants. Parent roots can define ecosystem-wide tasks;
child projects can define local tasks and processes. Each action must retain the
directory that gives it meaning.

Holla lesson: current folder has highest priority, but parent, child, and system
scope should remain easy to explore from one session.

## Activity multiplexing

Development work often starts several long-running commands: frontend server,
API, worker, logs, tests, and monitors. Users need to move between discovery and
these activities without losing output, state, or effective directory.

Holla lesson: preserve named activities and program insights with fast switching.
Tabs or terminal-multiplexer integration are possible representations, not
requirements.

## Dependency-aware command plans

Compound maintenance and setup intents are often directed acyclic graphs rather
than scripts. A shared preflight may unlock independent branches; each branch may
contain ordered work; final verification may wait for several branches.

Holla lesson: let users inspect and edit the plan, exclude optional steps, see
dependency consequences, run proven-independent branches concurrently, follow
per-step output, and understand failure propagation.

## System-upgrade orchestration

“Upgrade everything” may span operating-system packages, global tool managers,
cleanup, service checks, and reboot detection. Discovery and review are distinct
from applying changes.

Holla lesson: present available upgrades before mutation. Build a host-specific
plan from detected managers, allow exclusions, show standard system operations,
then execute with dependency-aware progress and retained output.
