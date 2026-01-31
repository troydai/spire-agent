# Sandbox usage guide

## Overall

The sandbox environment refers to the local kind cluster that provides a
representative SPIRE environment for testing.

This repo targets rebuilding the SPIRE agent in Rust. Utilizing the sandbox
environment to compare and ensure feature parity is key to the success of
the project.

## Terminology

Referring to the same thing using the same word is important. This section goes
through the terminology used here.

### Different implementation of spire-agent

This repo implements the SPIRE agent in Rust. The official SPIRE agent is implemented
in Go. Both are present in the sandbox environment. When referring to "SPIRE agent,"
it must be in context to determine which is which. When in doubt, decorate
the name with the language it is implemented in, for example "Rust SPIRE agent".

### General words

- Kubernetes and k8s are equivalent.

## Architecture

The sandbox is a Kubernetes cluster. We use a kind cluster as the implementation.
If better technology surfaces, we can move on to another local k8s implementation,
thus avoiding build-specific dependencies on kind.

A SPIRE server is set up. The server takes in a self-signed certificate chain that
mimics a CA.

A SPIRE agent DaemonSet is set up. This is the Go implementation of the SPIRE agent.
Before the Rust SPIRE agent is in a more complete state, the default SPIRE agent
DaemonSet is the Go version.

An httpbin service is set up under the httpbin namespace. It is a test service. With its
pod there are two more containers. The spiffe-helper is an upstream Go implementation
that is able to pull certs into files. It is set up for testing and inspection of the
certs. Another container is spiffe-debug; it is built to allow executing tests through
a shell or other means. This container environment is provided for customization.

## Test strategy

During local development, the cluster can be used to test various scenarios. You
can be creative. Here's an example:

To test the `spire-agent api fetch x509` command implementation, a Workload API that
returns a realistically bootstrapped cert chain, key, and trust bundle is needed. While
local spire-agent-mock can provide a mock that can be quickly thrown into action, it
is not a 1:1 representation of a real-world SPIRE server. The sandbox provides that. To
test the implementation during development, the Rust SPIRE agent can be built and loaded
into the spiffe-debug container of the httpbin pod and tested against the Go SPIRE agent.

