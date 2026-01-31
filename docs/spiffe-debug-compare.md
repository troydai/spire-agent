# Compare Rust vs Go SPIRE Agent in the httpbin spiffe-debug container

This guide runs both the Rust and Go `spire-agent` CLIs inside the `spiffe-debug` container of the `httpbin` pod and compares the X.509 SVID output.

## Prerequisites

- Kind cluster is running with SPIRE server and workload API socket mounted into the `httpbin` pod.
- The `spiffe-debug` container uses the local image `spiffe-debugger:local`.
- Kubeconfig path:

```bash
KUBECONFIG=/Users/troydai/code/github.com/troydai/spire-agent/sandbox/artifacts/kubeconfig
```

## Find the httpbin pod

```bash
kubectl --kubeconfig "$KUBECONFIG" -n httpbin get pods -o wide
```

Pick the running pod name, for example `httpbin-ff85dbfd4-mpf7r`.

## Locate the workload API socket

```bash
kubectl --kubeconfig "$KUBECONFIG" -n httpbin exec <POD> -c spiffe-debug -- ls -la /run/spire/sockets
```

Expected socket path:

```
/run/spire/sockets/agent.sock
```

## Run the Rust spire-agent

```bash
kubectl --kubeconfig "$KUBECONFIG" -n httpbin exec <POD> -c spiffe-debug -- \
  /usr/local/bin/spire-agent-rust api fetch x509 --socket-path /run/spire/sockets/agent.sock
```

## Run the Go spire-agent

```bash
kubectl --kubeconfig "$KUBECONFIG" -n httpbin exec <POD> -c spiffe-debug -- \
  /usr/local/bin/spire-agent-go api fetch x509 --socketPath /run/spire/sockets/agent.sock
```

## Compare outputs

A quick way to compare is to capture both outputs locally and run `diff`:

```bash
kubectl --kubeconfig "$KUBECONFIG" -n httpbin exec <POD> -c spiffe-debug -- \
  /usr/local/bin/spire-agent-rust api fetch x509 --socket-path /run/spire/sockets/agent.sock \
  > /tmp/spire-agent-rust.txt

kubectl --kubeconfig "$KUBECONFIG" -n httpbin exec <POD> -c spiffe-debug -- \
  /usr/local/bin/spire-agent-go api fetch x509 --socketPath /run/spire/sockets/agent.sock \
  > /tmp/spire-agent-go.txt

diff -u /tmp/spire-agent-rust.txt /tmp/spire-agent-go.txt
```

### What to look for

- **SPIFFE ID**: should match in both outputs.
- **SVID Valid After/Until**: should be in the same range (minor skew is normal).
- **CA chain details**: Go output usually includes both the intermediate and root CA.
  - If Rust output only shows one CA entry, it is likely missing the root CA.
