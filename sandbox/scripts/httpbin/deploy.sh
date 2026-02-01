#!/usr/bin/env bash
# Deploys the httpbin service to the Kubernetes cluster.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

ROOT_DIR="${ROOT_DIR:-$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)}"
KUBECONFIG_PATH="${KUBECONFIG_PATH:-${ROOT_DIR}/artifacts/kubeconfig}"
DEPLOY_DIR="${DEPLOY_DIR:-${ROOT_DIR}/deploy/httpbin}"

export KUBECONFIG="${KUBECONFIG_PATH}"

echo -e "[deploy] Deploying httpbin service..."

if kubectl apply -f "${DEPLOY_DIR}/httpbin.yaml" 2>/dev/null; then
	echo -e "[deploy] Waiting for httpbin pod to be ready..."
	if kubectl wait --for=condition=ready pod -l app=httpbin -n httpbin --timeout=60s 2>/dev/null; then
		echo -e "✓ httpbin pod is ready"
	else
		echo -e "[deploy] httpbin deployment may still be in progress"
	fi
else
	echo -e "[deploy] Failed to deploy httpbin (may already exist)"
fi

echo ""
echo -e "[deploy] httpbin deployed successfully!"
echo -e "[deploy] Pod status:"
kubectl get pods -n httpbin
