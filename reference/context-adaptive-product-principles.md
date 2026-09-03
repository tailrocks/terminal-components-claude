# Context-adaptive product principles

These principles define product behavior and user understanding, not visual
design. Any screen, component, layout, styling, or navigation form remains open
for independent exploration.

## Context before catalog

Begin with the user's location and state. Do not present every possible command
with equal weight.

## Recommendations before categories

The initial experience should offer a few plausible next actions. Categories
remain stable exploration paths, not the primary information structure.

## Intent before syntax

Search should understand goals, resources, and domain language. Exact executable
names and flags remain visible in preview, not required as input.

## Resources plus actions

Represent a project, service, file, process, or cleanup candidate once. Give it a
state-aware primary action and contextual alternatives.

## Here first, then expand

Prefer exact folder, then project, workspace, host, and global capabilities.
Crossing a scope boundary must remain visible.

Current working directory is the anchor, not the limit. Users must be able to
explore detected ancestors, bounded child projects, and host-wide capabilities
without relaunching. Every action retains its defining and execution scope.

## Understand both ancestors and children

When launched inside a child folder, ancestor roots may contribute ecosystem,
container, or workspace tasks. When launched at a monorepository root, child
projects may contribute their own tasks and processes, including tasks defined
by child `mise.toml` files.

Discover structure, not arbitrary depth. Explain why a parent or child action is
relevant and run it from its correct working directory.

## Explain relevance

Every recommendation should have a short reason tied to live state, context, or
observed use. Users should be able to inspect deeper reasoning.

## Keep adaptation predictable

Learn within context, preserve stable ordering where evidence is weak, and never
override exact aliases. Give users direct ranking controls.

## Separate relevance, confidence, and risk

A highly relevant action may still be destructive. A safe action may be weakly
inferred. Display and handle these dimensions independently. Risk must not hide
an intentional action or override a strong query match. Apply risk controls when
the user selects and executes it.

## Destructive actions remain first-class

Stopping every container, removing all containers, deleting all images, pruning
volumes, or performing complete Docker cleanup may be a normal repeated workflow
for a particular user and host. These actions should be searchable,
recommendable, aliasable, and learnable like other actions.

Their destructive nature requires truthful preview, explicit scope, deliberate
confirmation, and staged results. It does not imply low relevance.

## Bind confirmation to the exact target

Broad destructive work needs two distinct decisions:

1. Review the fully resolved plan and choose to continue.
2. Type a phrase containing the destructive operation and exact path, host, or
   resource set.

`DELETE EVERYTHING IN /work/scratch` is meaningful. Generic `yes` is not. Any
material plan change invalidates prior confirmation. Aliases and usage history
must never bypass either gate.

## Preview scope truthfully

Show exact path, project, host, affected resources, generated command, and
recoverability. Multi-step behavior must not hide behind a simplified preview.

## Earn trust for contributed workflows

Personal and team actions need provenance. Changed definitions need renewed
review. Trust should be narrow and inspectable.

## Stream without disruption

Show useful content immediately and add discoveries progressively. Preserve the
selected item's identity after navigation begins.

## Prefer progressive disclosure

Keep Root Search small. Open focused flows for disk exploration, cleanup,
multi-project results, file browsing, monitoring, and long-running output.

## Preserve active work

Long-running tasks, logs, monitors, and interactive programs should remain
available as named activities. Users can return to discovery, start other work,
and switch back without losing output or scope. The concept requires tab-capable,
multiplexer-like continuity; exact visual treatment remains open.

## Represent compound intent as a plan

A multi-command intent should become an inspectable dependency graph. Users can
review steps, commands, scope, privileges, optional branches, and parallelism
before execution.

Optional steps may be excluded. Required dependencies cannot disappear silently;
excluding one must disable dependent work or require a different valid plan.

## Preserve graph meaning during execution

Run independent steps concurrently only when their resource use is known not to
conflict. Show which steps are ready, running, waiting, blocked, failed,
excluded, or complete. Retain live and completed output per step.

A failure blocks dependents, not necessarily unrelated branches. Retry should
target failed work without repeating successful independent work. Exact graph
visualization remains open.

## Coordinate specialists

Own context, recommendation, safety, and navigation. Hand off deeper domain work
when a specialized terminal tool provides a stronger experience.

## Default to local control

Keep contextual memory local and minimal. Exclude secrets. Make history
inspectable, clearable, and optional.

## Evaluate by user understanding

The product succeeds when users quickly know:

- where they are;
- what matters now;
- why an item appeared;
- what the primary action will do;
- which scope it affects;
- whether it is safe;
- what happened afterward.
