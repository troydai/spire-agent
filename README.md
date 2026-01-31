# spire-agent (Rust)

This repository contains a Rust implementation of SPIRE Agent.

## Development guide

### Sandbox environment (local kind cluster)

The sandbox spins up a local kind cluster with SPIRE server/agent and a small test workload. The sandbox scripts and manifests live under `sandbox/` and are wired to top-level Makefile targets.

Prerequisites:
- kind
- kubectl
- helm
- jq
- openssl

Verify tooling:

```sh
make tools
```

Bring the environment up:

```sh
make env-up
```

Tear it down:

```sh
make env-down
```

Common targets:
- `make cluster-up` / `make cluster-down`: create or delete the kind cluster.
- `make certs`: generate CA and SPIRE server certs under `sandbox/artifacts/certs`.
- `make deploy-spire-server` / `make undeploy-spire-server`
- `make deploy-spire-agent` / `make undeploy-spire-agent`
- `make deploy-spire-csi` / `make undeploy-spire-csi`
- `make deploy-registration` / `make undeploy-registration`
- `make deploy-httpbin` / `make undeploy-httpbin`

Notes:
- Artifacts (kubeconfig, certs) are written to `sandbox/artifacts/`.
- The default SPIRE server address used by the agent is `spire-server.spire-server.svc.cluster.local`.
- If you need to rotate certificates, re-run `make certs` and redeploy the SPIRE server.
- `deploy-httpbin` uses public images and should not require pre-loading images into kind.
