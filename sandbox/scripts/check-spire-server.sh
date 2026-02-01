#!/usr/bin/env bash
# Checks the status of the SPIRE server, including pod status, service status, and logs.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

ROOT_DIR="${ROOT_DIR:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)}"
KUBECONFIG_PATH="${KUBECONFIG_PATH:-${ROOT_DIR}/artifacts/kubeconfig}"

export KUBECONFIG="${KUBECONFIG_PATH}"

echo -e "[check-spire-server] Checking SPIRE server status..."
echo ""

echo -e "[check-spire-server] === Pod Status ==="
if ! kubectl get pods -n spire-server -l app=spire-server; then
	echo -e "[check-spire-server] Error: SPIRE server namespace or pods not found. Run 'make deploy-spire-server' first."
	exit 1
fi

echo ""
echo -e "[check-spire-server] === Service Status ==="
kubectl get svc -n spire-server spire-server || echo -e "[check-spire-server] Service not found"

echo ""
echo -e "[check-spire-server] === Pod Logs (last 20 lines) ==="
kubectl logs -n spire-server -l app=spire-server --tail=20 || echo -e "[check-spire-server] Unable to fetch logs"

echo ""
echo -e "[check-spire-server] === Health Check ==="
if kubectl get pod -n spire-server -l app=spire-server -o jsonpath='{.items[0].status.conditions[?(@.type=="Ready")].status}' | grep -q "True"; then
	echo -e "✓ SPIRE server pod is Ready"
else
	echo -e "✗ SPIRE server pod is not Ready"
fi

if kubectl get pod -n spire-server -l app=spire-server -o jsonpath='{.items[0].status.containerStatuses[0].ready}' | grep -q "true"; then
	echo -e "✓ SPIRE server container is ready"
else
	echo -e "✗ SPIRE server container is not ready"
fi

echo ""
echo -e "[check-spire-server] To view full logs: kubectl logs -n spire-server -l app=spire-server -f"
echo -e "[check-spire-server] To exec into pod: kubectl exec -it -n spire-server -l app=spire-server -- /bin/sh"

