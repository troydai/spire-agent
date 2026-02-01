#!/usr/bin/env bash
# Removes the httpbin service and its associated resources from the cluster.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

ROOT_DIR="${ROOT_DIR:-$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)}"
KUBECONFIG_PATH="${KUBECONFIG_PATH:-${ROOT_DIR}/artifacts/kubeconfig}"

export KUBECONFIG="${KUBECONFIG_PATH}"

echo -e "[undeploy] Removing httpbin service..."
if kubectl get namespace httpbin > /dev/null 2>&1; then
	echo -e "[undeploy] Deleting Deployment..."
	kubectl delete deployment httpbin -n httpbin --ignore-not-found=true
	echo -e "[undeploy] Deleting Service..."
	kubectl delete service httpbin -n httpbin --ignore-not-found=true
	echo -e "[undeploy] Deleting ConfigMap..."
	kubectl delete configmap spiffe-helper-config -n httpbin --ignore-not-found=true
	echo -e "[undeploy] Deleting ServiceAccount..."
	kubectl delete serviceaccount httpbin -n httpbin --ignore-not-found=true
	echo -e "[undeploy] Deleting namespace..."
	kubectl delete namespace httpbin --ignore-not-found=true
	echo ""
	echo -e "[undeploy] httpbin removed successfully!"
else
	echo -e "[undeploy] Namespace 'httpbin' does not exist. Nothing to remove."
fi
