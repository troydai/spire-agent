#!/usr/bin/env bash
# Removes the SPIRE agent and its associated resources from the cluster.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

ROOT_DIR="${ROOT_DIR:-$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)}"
KUBECONFIG_PATH="${KUBECONFIG_PATH:-${ROOT_DIR}/artifacts/kubeconfig}"

export KUBECONFIG="${KUBECONFIG_PATH}"

echo -e "[undeploy-spire-agent] Removing SPIRE agent..."

if [ ! -f "${KUBECONFIG_PATH}" ]; then
	echo -e "[undeploy-spire-agent] Kubeconfig not found. Skipping undeploy."
	exit 0
fi

echo -e "[undeploy-spire-agent] Deleting DaemonSet..."
kubectl delete daemonset spire-agent -n spire-agent --ignore-not-found=true
echo -e "[undeploy-spire-agent] Deleting ConfigMap..."
kubectl delete configmap spire-agent-config -n spire-agent --ignore-not-found=true
echo -e "[undeploy-spire-agent] Deleting ClusterRoleBinding and ClusterRole..."
kubectl delete clusterrolebinding spire-agent-cluster-role-binding --ignore-not-found=true
kubectl delete clusterrole spire-agent-cluster-role --ignore-not-found=true
echo -e "[undeploy-spire-agent] Deleting ServiceAccount..."
kubectl delete serviceaccount spire-agent -n spire-agent --ignore-not-found=true
echo -e "[undeploy-spire-agent] Deleting Secret..."
kubectl delete secret spire-bundle -n spire-agent --ignore-not-found=true
echo -e "[undeploy-spire-agent] Deleting namespace..."
kubectl delete namespace spire-agent --ignore-not-found=true

echo ""
echo -e "[undeploy-spire-agent] SPIRE agent removed."
