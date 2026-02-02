## Goal

Implement the "spire-agent healthcheck" command.

```
/ # spire-agent-go api fetch jwt -h
Usage of fetch jwt:
  -audience value
        comma separated list of audience values
  -format value
        deprecated; use -output
  -output value
        Desired output format (pretty, json); default: pretty.
  -socketPath string
        Path to the SPIRE Agent API Unix domain socket (default "/tmp/spire-agent/public/api.sock")
  -spiffeID string
        SPIFFE ID subject (optional)
  -timeout value
        Time to wait for a response (default 5s)
```

Use the Go spire-agent as a reference, compare its output to the Rust spire-agent in the sandbox, and implement the
equivalent in the Rust spire-agent.

## Steps

- [ ] Execute the spire-agent-go api fetch jwt in sandbox to understand its options and output. Save output examples in
      the tast manifest's reference section for later reference.
- [ ] Update spire-agent-mock to mimic the function of return a JWT SVID. Running Go spire-agent against the mock and
      real Go spire-agent in sandbox should produce the same output.
- [ ] Implement the Rust spire-agent "api fetch jwt" command. Test it against spire-agent-mock to ensure its output is
      the same as the Go spire-agent.
- [ ] Integration-test the Rust spire-agent in the sandbox to ensure it passes end-to-end tests.
- [ ] Create a PR from this feature branch to the main branch.

## References

- [How to run spire-agent in sandbox to compare](docs/sandbox_guide.md)
- [SPIRE Go Source Code](https://github.com/spiffe/spire)
