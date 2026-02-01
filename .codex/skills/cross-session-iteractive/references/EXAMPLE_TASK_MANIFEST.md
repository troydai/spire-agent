# Implement "spire-agent api fetch" command

## Goal

Use the Go spire-agent as a reference, compare its output to the Rust spire-agent in the sandbox, and implement the equivalent in the Rust spire-agent.

## Steps

- [x] Update spire-agent-mock so that running the Go spire-agent "api fetch" command against its workload API has the same output as running the Go spire-agent against the workload API in the sandbox. The purpose of this task is to provide a test harness for later tasks.
  - Sub-step 1
  - Sub-step 2
- [ ] Implement the Rust spire-agent "api fetch" command. Test it against spire-agent-mock to ensure its output is the same as the Go spire-agent.
- [ ] Integration-test the Rust spire-agent in the sandbox to ensure it passes end-to-end tests.
- [ ] Create a PR from this feature branch to the main branch.

## References

- [How to run spire-agent in sandbox to compare](docs/sandbox_guide.md)
- [SPIRE Go Source Code](https://github.com/spiffe/spire)

## Tools

- git
- cargo
- kubectl
