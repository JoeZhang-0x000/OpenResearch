---
name: orx-agent-delegation
description: "Delegate independent work to helper agent sessions with `orx agent spawn` and follow up in the same session with `orx agent send`: task selection, self-contained briefs, branch ownership, compute authorization, wakeups, and nesting or concurrency constraints. Use before spawning a helper, sending it a follow-up, or interpreting its result; do not delegate the literature retrieval loop."
---

# Delegate work to another agent

`orx agent spawn` creates a new top-level session in the same project. The
helper is visible to the user, receives its own worktree and transcript, and
works independently from this session.

```sh
orx agent spawn "<self-contained task>"
orx agent spawn --title "<session title>" --stdin
orx agent spawn "<task>" --harness <harness> --model <model>
orx agent spawn "<task>" --no-wake
orx agent send <session-id> "<follow-up message>"
orx agent send <session-id> --stdin
```

By default, this chat resumes with the helper's closing reply. Use `--no-wake`
only when no follow-up is needed. A spawned session cannot spawn another helper,
and the CLI enforces the number of helpers a session may have in flight. If the
command refuses a spawn for either reason, do the work here or wait for a helper
to finish. It also refuses a `--harness` that OpenResearch cannot find installed;
spawn on this session's harness instead, or tell the user.

## Follow up instead of re-spawning

Once a helper has reported back, `orx agent send` keeps the conversation going
in the same session: the message lands in the helper's transcript and runs as
its next turn on the harness, model, and worktree it was spawned with. This
chat resumes with the reply, exactly like the closing report. Prefer `send`
over a fresh spawn for multi-turn work — every spawn is a new sidebar session
with an empty transcript, so follow-ups via spawn waste sessions and re-brief
the same context each round.

`send` only reaches helpers this chat spawned (the id `orx agent spawn`
printed), only while the helper is idle — a send against a helper still
working is refused, so wait for its reply before sending another. A spawned
session cannot send follow-ups to helpers of other sessions.

## Choose tasks with a clean boundary

Delegate work that is genuinely independent from the node this session owns,
such as surveying an unfamiliar codebase or writing up completed results. Do
the work here when it is a step in the experiment loop already underway.

Never delegate the retrieval loop covered by `orx-lit-review`: the main agent
must inspect and rank the combined literature candidates itself.

## Protect branches and compute

- Never give a helper a branch checked out by this session. If it must change
  experiment code, tell it to create its own node and work on that node's branch.
- A frozen experiment node may not be edited by either session.
- State exactly which `orx exp run` calls the helper may launch. Explicitly
  forbid launches when none are authorized; otherwise the helper may infer that
  the normal research loop is available.
- The helper's edits remain in its worktree. Nothing merges into this branch
  automatically.

## Write a standalone brief

The helper starts with an empty transcript and cannot see this conversation.
Include the project, relevant experiment and branch, metric, constraints,
allowed compute, expected output, and a concrete definition of done. Use
`--stdin` for a multi-paragraph brief.

A Claude helper's closing reply waits for its background commands to exit (or
about 30 quiet minutes), so do not ask it to leave servers or watchers running
unless the task needs them.
