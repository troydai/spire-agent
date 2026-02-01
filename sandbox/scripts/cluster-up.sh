#!/usr/bin/env bash
# Sets up a local Kubernetes cluster using kind and configures the kubeconfig for the environment.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"


ROOT_DIR="${ROOT_DIR:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)}"
KIND="${KIND:-kind}"
KIND_CLUSTER_NAME="${KIND_CLUSTER_NAME:-spiffe-helper}"
KIND_CONFIG="${KIND_CONFIG:-${ROOT_DIR}/kind-config.yaml}"
ARTIFACTS_DIR="${ARTIFACTS_DIR:-${ROOT_DIR}/artifacts}"
KUBECONFIG_PATH="${KUBECONFIG_PATH:-${ARTIFACTS_DIR}/kubeconfig}"

# Create artifacts directory
mkdir -p "${ARTIFACTS_DIR}"

echo -e "[cluster-up] Setting up kind cluster..."

# Check if cluster already exists
if ${KIND} get clusters | grep -qx "${KIND_CLUSTER_NAME}"; then
	echo -e "[cluster-up] kind cluster '${KIND_CLUSTER_NAME}' already exists"
else
	echo -e "[cluster-up] Creating kind cluster '${KIND_CLUSTER_NAME}'..."
	KUBECONFIG="${KUBECONFIG_PATH}" ${KIND} create cluster --name "${KIND_CLUSTER_NAME}" --config "${KIND_CONFIG}"
fi

# Get kubeconfig
echo -e "[cluster-up] Writing kubeconfig..."
${KIND} get kubeconfig --name "${KIND_CLUSTER_NAME}" > "${KUBECONFIG_PATH}"
echo -e "✓ Kubeconfig written to ${KUBECONFIG_PATH}"

echo ""
echo -e "[cluster-up] Cluster setup complete!"
