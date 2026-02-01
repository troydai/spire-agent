## Goal

Implement the "spire-agent healthcheck" command.

Example in Go spire-agent

```
/ # spire-agent-go healthcheck --help
Usage of health:
  -shallow
        Perform a less stringent health check
  -socketPath string
        Path to the SPIRE Agent API socket (default "/tmp/spire-agent/public/api.sock")
  -verbose
        Print verbose information

/ #
```

Use the Go spire-agent as a reference, compare its output to the Rust spire-agent in the sandbox, and implement the equivalent in the Rust spire-agent.

## Steps

- [x] Update spire-agent-mock so that running the Go spire-agent "healthcheck" command against its workload API has the same output as running the Go spire-agent against the workload API in the sandbox. The purpose of this task is to provide a test harness for later tasks.
- [x] Implement the Rust spire-agent "healthcheck" command. Test it against spire-agent-mock to ensure its output is the same as the Go spire-agent.
- [x] Integration-test the Rust spire-agent in the sandbox to ensure it passes end-to-end tests.
- [x] Create a PR from this feature branch to the main branch.

## References

- [How to run spire-agent in sandbox to compare](docs/sandbox_guide.md)
- [SPIRE Go Source Code](https://github.com/spiffe/spire)