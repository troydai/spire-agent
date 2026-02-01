# Implement "api fetch" command

Use the Go spire-agent as a reference, compare its output to the Rust spire-agent in the sandbox, and implement the equivalent in the Rust spire-agent.

## References

- docs/sandbox_guide.md (read first): How to run spire-agent in sandbox to compare
- https://github.com/spiffe/spire: spire source code

## Instructions

When you start, look at the task section below. Find the first available task to accomplish. Whether a task has been accomplished is marked by its preceding checkbox.
If all tasks are already checked, exit immediately without making any changes.

Completion marker:
- Create a file named `.codex_done` in the repository root when you finish the final task.
- If `.codex_done` already exists, do nothing and exit.

Execute the task in the following sequence:
1. Understand the task's purpose.
2. Analyze and plan the actions.
3. Collect data.
4. Implement necessary tests first to drive the development. This includes updating spire-agent-mock to provide a quicker test harness than the sandbox.
5. Implement in Rust spire-agent.
6. Test it against unit tests, spire-agent-mock, and the sandbox.
7. Run the pre-commit hook and fix issues.
8. Update this file to check off the accomplished task.
9. Commit the change with a comprehensive git commit description and a concise title 
10. Exit the session.

## Tasks

- [x] Update spire-agent-mock so that running the Go spire-agent "api fetch" command against its workload API has the same output as running the Go spire-agent against the workload API in the sandbox. The purpose of this task is to provide a test harness for later tasks.
- [ ] Implement the Rust spire-agent "api fetch" command. Test it against spire-agent-mock to ensure its output is the same as the Go spire-agent.
- [ ] Integration-test the Rust spire-agent in the sandbox to ensure it passes end-to-end tests.
- [ ] Create a PR from this feature branch to the main branch.
- [ ] Create the `.codex_done` completion marker file in the repository root.
