#!/bin/bash
# Deploys the SPIRE CSI driver to the Kubernetes cluster.

set -e

# Source color support
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
COLOR_RESET=""
COLOR_BOLD=""
COLOR_RED=""
COLOR_GREEN=""
COLOR_YELLOW=""
COLOR_BLUE=""
COLOR_MAGENTA=""
COLOR_CYAN=""
COLOR_WHITE=""
COLOR_BRIGHT_RED=""
COLOR_BRIGHT_GREEN=""
COLOR_BRIGHT_YELLOW=""
COLOR_BRIGHT_BLUE=""
COLOR_BRIGHT_MAGENTA=""
COLOR_BRIGHT_CYAN=""

ROOT_DIR="${ROOT_DIR:-$(cd "${SCRIPT_DIR}/../.." && pwd)}"
KUBECONFIG_PATH="${KUBECONFIG_PATH:-${ROOT_DIR}/artifacts/kubeconfig}"

export KUBECONFIG="${KUBECONFIG_PATH}"

echo -e "${COLOR_BRIGHT_BLUE}[deploy-csi]${COLOR_RESET} ${COLOR_BOLD}Deploying SPIRE CSI driver...${COLOR_RESET}"
kubectl apply -f "${SCRIPT_DIR}/../../deploy/spire/csi/"
echo -e "${COLOR_GREEN}✓${COLOR_RESET} SPIRE CSI driver deployed"
