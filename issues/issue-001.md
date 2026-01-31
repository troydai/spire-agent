# Migrate spiffe-helper-rust test environment to this repo

## Goal

Migrate the scripts that create a SPIRE server/agent environment in a kind cluster to support the integration test from another repository.

## Facts

In `~/code/github.com/troydai/spiffe-helper-rust`, there is a set of scripts that set up a kind cluster, run the SPIRE server and agent, and deploy a test application. Inspect the Makefile and you will find the `env-up`, `env-down`, `cluster-up`, and `cluster-down` targets.

## Expected outcome

- Migrate the relevant scripts to this repo;
- Place them under sandbox directory;
- Introduce Makefile at the root of the repo and enable the above four targets;

## Plan

1) Inspect the source repo's scripts and Makefile targets to identify all required files and dependencies.
2) Copy or adapt the scripts into this repo under a `sandbox/` directory, preserving paths referenced by the targets.
3) Add a root-level `Makefile` that exposes `env-up`, `env-down`, `cluster-up`, and `cluster-down`, wiring them to the migrated scripts.
4) Validate the targets locally (or via a dry run) to confirm the scripts resolve paths and prerequisites correctly.
