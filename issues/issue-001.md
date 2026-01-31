# Migrate spiffe-helper-rust test environment to this repo

## Goal

Migrate the scripts that create a spire server/agent environment in a kind cluster to facilitate the integration test from another repoistory.

## Facts

In "~/code/github.com/troydai/spiffe-helper-rust" there is set of scripts that help set up a kind cluster, running spire server and agent, as well as a test application in it. Inspect the Makefile you will find env-up, env-down, cluster-up, and cluster-down target.

## Expected outcome

- Migrate the relevant scripts to this repo;
- Place them under sandbox directory;
- Introduce Makefile at the root of the repo and enable the above four targets;

