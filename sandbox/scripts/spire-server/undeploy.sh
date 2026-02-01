#!/usr/bin/env bash
# Removes the SPIRE server and its associated resources from the cluster.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

ROOT_DIR="${ROOT_DIR:-$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)}"
KUBECONFIG_PATH="${KUBECONFIG_PATH:-${ROOT_DIR}/artifacts/kubeconfig}"

export KUBECONFIG="${KUBECONFIG_PATH}"

echo -e "[undeploy] Removing SPIRE server..."
if kubectl get namespace spire-server > /dev/null 2>&1; then
	echo -e "[undeploy] Deleting Deployment..."
	kubectl delete deployment spire-server -n spire-server --ignore-not-found=true
	echo -e "[undeploy] Deleting Service..."
	kubectl delete service spire-server -n spire-server --ignore-not-found=true
	echo -e "[undeploy] Deleting ConfigMap..."
	kubectl delete configmap spire-server-config -n spire-server --ignore-not-found=true
	echo -e "[undeploy] Deleting Secrets..."
	kubectl delete secret spire-server-tls spire-server-ca spire-server-bootstrap -n spire-server --ignore-not-found=true
	echo -e "[undeploy] Deleting ServiceAccount..."
	kubectl delete serviceaccount spire-server -n spire-server --ignore-not-found=true
	echo -e "[undeploy] Deleting ClusterRoleBinding and ClusterRole..."
	kubectl delete clusterrolebinding spire-server-cluster-role-binding --ignore-not-found=true
	kubectl delete clusterrole spire-server-cluster-role --ignore-not-found=true
	echo -e "[undeploy] Deleting namespace..."
	kubectl delete namespace spire-server --ignore-not-found=true
	echo ""
	echo -e "[undeploy] SPIRE server removed successfully!"
else
	echo -e "[undeploy] Namespace 'spire-server' does not exist. Nothing to remove."
fi

