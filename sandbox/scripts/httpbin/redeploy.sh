#!/usr/bin/env bash
# Redeploys httpbin and restarts the deployment to pick up local image updates.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="${ROOT_DIR:-$(cd "${SCRIPT_DIR}/../.." && pwd)}"
KUBECONFIG_PATH="${KUBECONFIG_PATH:-${ROOT_DIR}/artifacts/kubeconfig}"

export KUBECONFIG="${KUBECONFIG_PATH}"

"${SCRIPT_DIR}/deploy.sh"

echo "[redeploy] Restarting httpbin deployment to pick up local images..."
if kubectl rollout restart deployment httpbin -n httpbin 2>/dev/null; then
	if kubectl rollout status deployment httpbin -n httpbin --timeout=120s 2>/dev/null; then
		echo "[redeploy] httpbin rollout complete"
	else
		echo "[redeploy] httpbin rollout may still be in progress"
	fi
else
	echo "[redeploy] Failed to restart httpbin deployment"
fi

kubectl get pods -n httpbin
