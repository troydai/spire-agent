---
name: cross-session-iterative
description: Accomplish large tasks by collaborating across multiple Agent Coding sessions so the context window does not limit success.
---
# Purpose

Enable cross-session execution of large tasks by persisting state in a task
manifest. This mitigates context window limits and supports later course
correction across sessions coordinated by an external orchestrator.

## Trigger

Use when prompted to work from a task manifest or when "cross-session-iterative"
is named explicitly (for example: "prompt.md", "task_xxx.md", "issue_xxx.md").

## Task Manifest

A task manifest is a Markdown file named by the prompt (often "prompt.md",
"task_xxx.md", or "issue_xxx.md").

### Format

Only the Goal and Steps sections are required.

- Goal: Rationale, implementation guidance, end state.
- Steps: Checklist with leading brackets; `[x]` means done. Sub-steps have no
  leading brackets. Unless specified, use the Step Execution Framework.
- References: Supporting documents or links.
- Tools: Programs or procedures that help execute steps.

### Step Execution Framework

Execute one step per session, then exit:

1. Read the manifest and select the first incomplete step.
2. Review references/tools as needed.
3. Add or adjust tests first when applicable.
4. Implement in small iterations with tests.
5. Run any necessary manual checks.
6. Mark the step `[x]` in the manifest.
7. Commit changes (and push if the branch has a remote).

### Logistical steps

The manifest may define logistical steps (concrete tasks) such as:

- Create a PR so another party can start reviewing changes.
- Call a webhook for service integration.

For these steps, execute directly without the full framework.

### Early exit

Exit early if:

- There are no more steps to execute.
- The current step cannot be fulfilled for technical reasons.

Before exit:

- Create `.agent_iteration_done` in the repository root.
- If exiting due to an unfulfillable step, record the reason in the file.

### Example Task Manifest

See `EXAMPLE_TASK_MANIFEST.md`.
