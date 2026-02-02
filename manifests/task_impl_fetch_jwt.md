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

- [x] Execute the spire-agent-go api fetch jwt in sandbox to understand its options and output. Save output examples in
      the tast manifest's reference section for later reference.
- [x] Update spire-agent-mock to mimic the function of return a JWT SVID. Running Go spire-agent against the mock and
      real Go spire-agent in sandbox should produce the same output.
- [x] Implement the Rust spire-agent "api fetch jwt" command. Test it against spire-agent-mock to ensure its output is
      the same as the Go spire-agent.
- [x] Integration-test the Rust spire-agent in the sandbox to ensure it passes end-to-end tests.
- [x] Create a PR from this feature branch to the main branch.

## References

- [How to run spire-agent in sandbox to compare](docs/sandbox_guide.md)
- [SPIRE Go Source Code](https://github.com/spiffe/spire)

### `spire-agent-go api fetch jwt` output samples (captured 2026-02-02)

#### Help output

```text
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

#### Missing audience error output

```text
audience must be specified
command terminated with exit code 1
```

#### Pretty output sample

```text
token(spiffe://spiffe-helper.local/ns/httpbin/sa/httpbin):
        eyJhbGciOiJFUzI1NiIsImtpZCI6Ijl2WGd6MUhmd1ZOVW1NUDZnZFB1V3hwa254RktSWXJEIiwidHlwIjoiSldUIn0.eyJhdWQiOlsiZXhhbXBsZS5vcmciXSwiZXhwIjoxNzcwMDQ5MzE3LCJpYXQiOjE3NzAwNDkwMTcsInN1YiI6InNwaWZmZTovL3NwaWZmZS1oZWxwZXIubG9jYWwvbnMvaHR0cGJpbi9zYS9odHRwYmluIn0.l7XBbEbLcCoYtRmJGsv3cDMvlvF5r8F-NZE-Mjk1P5RN_4FIIG-EWSwhH8NG6dILY4wVfDeYXuV2ab0BGDrT6g
bundle(spiffe://spiffe-helper.local):
        {
    "keys": [
        {
            "kty": "EC",
            "kid": "8ViYGVeIo8wtu5nJdxyeGPZRyFkoel6v",
            "crv": "P-256",
            "x": "qSmmVpv-o3iFyG9HS68UOlB004h1HW4n1sWwTvoap4c",
            "y": "V2kSsFvFKK2KLGMzhLr68OA5Zw4z_IfjfArFfquk--A"
        },
        {
            "kty": "EC",
            "kid": "9vXgz1HfwVNUmMP6gdPuWxpknxFKRYrD",
            "crv": "P-256",
            "x": "oACq_ZESRtbrKxDb4JiqrhcRDKIFwwZkfYsbY95QcAI",
            "y": "pmras2BYAsvojJ8lWdI5zer4ck4RQ_0tnnYir2K_Gdg"
        },
        {
            "kty": "EC",
            "kid": "uxIVEPTh44AMNWa7bP3oYYioqdhX3rqq",
            "crv": "P-256",
            "x": "FwrEC5fDyKMimyVPPQrmaudWXvgP9Riv6YGuODHSkPI",
            "y": "rdRWVk7KKTU6ZR569UkMwpDww4g54ZCplrLeUAHLT9U"
        }
    ]
}
```

#### JSON output sample

```text
[{"svids":[{"hint":"","spiffe_id":"spiffe://spiffe-helper.local/ns/httpbin/sa/httpbin","svid":"eyJhbGciOiJFUzI1NiIsImtpZCI6Ijl2WGd6MUhmd1ZOVW1NUDZnZFB1V3hwa254RktSWXJEIiwidHlwIjoiSldUIn0.eyJhdWQiOlsiZXhhbXBsZS5vcmciXSwiZXhwIjoxNzcwMDQ5MzE3LCJpYXQiOjE3NzAwNDkwMTcsInN1YiI6InNwaWZmZTovL3NwaWZmZS1oZWxwZXIubG9jYWwvbnMvaHR0cGJpbi9zYS9odHRwYmluIn0.l7XBbEbLcCoYtRmJGsv3cDMvlvF5r8F-NZE-Mjk1P5RN_4FIIG-EWSwhH8NG6dILY4wVfDeYXuV2ab0BGDrT6g"}]},{"bundles":{"spiffe://spiffe-helper.local":"ewogICAgImtleXMiOiBbCiAgICAgICAgewogICAgICAgICAgICAia3R5IjogIkVDIiwKICAgICAgICAgICAgImtpZCI6ICJ1eElWRVBUaDQ0QU1OV2E3YlAzb1lZaW9xZGhYM3JxcSIsCiAgICAgICAgICAgICJjcnYiOiAiUC0yNTYiLAogICAgICAgICAgICAieCI6ICJGd3JFQzVmRHlLTWlteVZQUFFybWF1ZFdYdmdQOVJpdjZZR3VPREhTa1BJIiwKICAgICAgICAgICAgInkiOiAicmRSV1ZrN0tLVFU2WlI1NjlVa013cER3dzRnNTRaQ3BsckxlVUFITFQ5VSIKICAgICAgICB9LAogICAgICAgIHsKICAgICAgICAgICAgImt0eSI6ICJFQyIsCiAgICAgICAgICAgICJraWQiOiAiOFZpWUdWZUlvOHd0dTVuSmR4eWVHUFpSeUZrb2VsNnYiLAogICAgICAgICAgICAiY3J2IjogIlAtMjU2IiwKICAgICAgICAgICAgIngiOiAicVNtbVZwdi1vM2lGeUc5SFM2OFVPbEIwMDRoMUhXNG4xc1d3VHZvYXA0YyIsCiAgICAgICAgICAgICJ5IjogIlYya1NzRnZGS0syS0xHTXpoTHI2OE9BNVp3NHpfSWZqZkFyRmZxdWstLUEiCiAgICAgICAgfSwKICAgICAgICB7CiAgICAgICAgICAgICJrdHkiOiAiRUMiLAogICAgICAgICAgICAia2lkIjogIjl2WGd6MUhmd1ZOVW1NUDZnZFB1V3hwa254RktSWXJEIiwKICAgICAgICAgICAgImNydiI6ICJQLTI1NiIsCiAgICAgICAgICAgICJ4IjogIm9BQ3FfWkVTUnRickt4RGI0SmlxcmhjUkRLSUZ3d1prZllzYlk5NVFjQUkiLAogICAgICAgICAgICAieSI6ICJwbXJhczJCWUFzdm9qSjhsV2RJNXplcjRjazRSUV8wdG5uWWlyMktfR2RnIgogICAgICAgIH0KICAgIF0KfQ=="}}]
```
