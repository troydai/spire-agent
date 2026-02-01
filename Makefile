ROOT_DIR := $(abspath $(dir $(lastword $(MAKEFILE_LIST))))
SANDBOX_DIR := $(ROOT_DIR)/sandbox
SCRIPTS_DIR := $(SANDBOX_DIR)/scripts
KIND_CLUSTER_NAME ?= spiffe-helper
KIND ?= kind
KIND_CONFIG := $(SANDBOX_DIR)/kind-config.yaml
ARTIFACTS_DIR := $(SANDBOX_DIR)/artifacts
KUBECONFIG_PATH := $(ARTIFACTS_DIR)/kubeconfig
CERT_DIR := $(ARTIFACTS_DIR)/certs
KUBECTL := KUBECONFIG="$(KUBECONFIG_PATH)" kubectl
DOCKER ?= docker

.PHONY: tools
tools:
	@ROOT_DIR="$(SANDBOX_DIR)" $(SCRIPTS_DIR)/install-tools.sh

.PHONY: certs
certs:
	@CERT_DIR="$(CERT_DIR)" $(SCRIPTS_DIR)/generate-certs.sh

.PHONY: clean
clean:
	@echo "[clean] Removing generated artifacts..."
	@rm -rf $(ARTIFACTS_DIR)
	@echo "[clean] Clean complete."

.PHONY: cluster-up
cluster-up: $(KIND_CONFIG)
	@KIND="$(KIND)" KIND_CLUSTER_NAME="$(KIND_CLUSTER_NAME)" KIND_CONFIG="$(KIND_CONFIG)" ARTIFACTS_DIR="$(ARTIFACTS_DIR)" KUBECONFIG_PATH="$(KUBECONFIG_PATH)" ROOT_DIR="$(SANDBOX_DIR)" $(SCRIPTS_DIR)/cluster-up.sh

.PHONY: cluster-down
cluster-down:
	@if kind get clusters | grep -qx "$(KIND_CLUSTER_NAME)"; then \
		echo "[cluster-down] Deleting kind cluster '$(KIND_CLUSTER_NAME)'"; \
		kind delete cluster --name "$(KIND_CLUSTER_NAME)"; \
	else \
		echo "[cluster-down] kind cluster '$(KIND_CLUSTER_NAME)' already absent"; \
	fi
	@rm -f "$(KUBECONFIG_PATH)"
	@if [ -d "$(ARTIFACTS_DIR)" ] && [ -z "$$(ls -A "$(ARTIFACTS_DIR)" 2>/dev/null)" ]; then rmdir "$(ARTIFACTS_DIR)"; fi

.PHONY: check-cluster
check-cluster:
	@if ! kind get clusters | grep -qx "$(KIND_CLUSTER_NAME)"; then \
		echo "[check-cluster] Error: kind cluster '$(KIND_CLUSTER_NAME)' does not exist. Run 'make cluster-up' first."; \
		exit 1; \
	fi
	@if [ ! -f "$(KUBECONFIG_PATH)" ]; then \
		echo "[check-cluster] Error: kubeconfig not found at $(KUBECONFIG_PATH). Run 'make cluster-up' first."; \
		exit 1; \
	fi
	@if ! $(KUBECTL) cluster-info > /dev/null 2>&1; then \
		echo "[check-cluster] Error: unable to connect to cluster. Run 'make cluster-up' first."; \
		exit 1; \
	fi

.PHONY: check-certs
check-certs:
	@if [ ! -f "$(CERT_DIR)/ca-cert.pem" ] || [ ! -f "$(CERT_DIR)/ca-key.pem" ]; then \
		echo "[check-certs] Error: CA certificate files not found. Expected: $(CERT_DIR)/ca-cert.pem, $(CERT_DIR)/ca-key.pem"; \
		echo "[check-certs] Run 'make certs' first."; \
		exit 1; \
	fi
	@if [ ! -f "$(CERT_DIR)/spire-server-cert.pem" ] || [ ! -f "$(CERT_DIR)/spire-server-key.pem" ]; then \
		echo "[check-certs] Error: SPIRE server certificate files not found. Expected: $(CERT_DIR)/spire-server-cert.pem, $(CERT_DIR)/spire-server-key.pem"; \
		echo "[check-certs] Run 'make certs' first."; \
		exit 1; \
	fi
	@if [ ! -f "$(CERT_DIR)/bootstrap-bundle.pem" ]; then \
		echo "[check-certs] Error: Bootstrap bundle not found. Expected: $(CERT_DIR)/bootstrap-bundle.pem"; \
		echo "[check-certs] Run 'make certs' first."; \
		exit 1; \
	fi

.PHONY: deploy-spire-server
deploy-spire-server: cluster-up certs
	@ROOT_DIR="$(SANDBOX_DIR)" KUBECONFIG_PATH="$(KUBECONFIG_PATH)" CERT_DIR="$(CERT_DIR)" $(SCRIPTS_DIR)/spire-server/deploy.sh

.PHONY: undeploy-spire-server
undeploy-spire-server:
	@ROOT_DIR="$(SANDBOX_DIR)" KUBECONFIG_PATH="$(KUBECONFIG_PATH)" $(SCRIPTS_DIR)/spire-server/undeploy.sh

.PHONY: check-spire-server
check-spire-server: check-cluster
	@ROOT_DIR="$(SANDBOX_DIR)" KUBECONFIG_PATH="$(KUBECONFIG_PATH)" $(SCRIPTS_DIR)/check-spire-server.sh

.PHONY: deploy-spire-agent
deploy-spire-agent: certs
	@ROOT_DIR="$(SANDBOX_DIR)" KUBECONFIG_PATH="$(KUBECONFIG_PATH)" CERT_DIR="$(CERT_DIR)" $(SCRIPTS_DIR)/spire-agent/deploy.sh

.PHONY: undeploy-spire-agent
undeploy-spire-agent:
	@ROOT_DIR="$(SANDBOX_DIR)" KUBECONFIG_PATH="$(KUBECONFIG_PATH)" $(SCRIPTS_DIR)/spire-agent/undeploy.sh

.PHONY: deploy-registration
deploy-registration: check-cluster
	@ROOT_DIR="$(SANDBOX_DIR)" KUBECONFIG_PATH="$(KUBECONFIG_PATH)" $(SCRIPTS_DIR)/registration/deploy.sh

.PHONY: undeploy-registration
undeploy-registration:
	@ROOT_DIR="$(SANDBOX_DIR)" KUBECONFIG_PATH="$(KUBECONFIG_PATH)" $(SCRIPTS_DIR)/registration/undeploy.sh

.PHONY: deploy-spire-csi
deploy-spire-csi: check-cluster
	@ROOT_DIR="$(SANDBOX_DIR)" KUBECONFIG_PATH="$(KUBECONFIG_PATH)" $(SCRIPTS_DIR)/spire-csi/deploy.sh

.PHONY: undeploy-spire-csi
undeploy-spire-csi:
	@ROOT_DIR="$(SANDBOX_DIR)" KUBECONFIG_PATH="$(KUBECONFIG_PATH)" $(SCRIPTS_DIR)/spire-csi/undeploy.sh

.PHONY: deploy-httpbin
deploy-httpbin: check-cluster
	@ROOT_DIR="$(SANDBOX_DIR)" KUBECONFIG_PATH="$(KUBECONFIG_PATH)" $(SCRIPTS_DIR)/httpbin/deploy.sh

.PHONY: build-spiffe-debug
build-spiffe-debug: check-cluster
	@ROOT_DIR="$(ROOT_DIR)" SANDBOX_DIR="$(SANDBOX_DIR)" KIND_CLUSTER_NAME="$(KIND_CLUSTER_NAME)" KIND="$(KIND)" DOCKER="$(DOCKER)" $(SCRIPTS_DIR)/spiffe-debugger/build.sh

.PHONY: redeploy-httpbin
redeploy-httpbin: build-spiffe-debug check-cluster
	@ROOT_DIR="$(SANDBOX_DIR)" KUBECONFIG_PATH="$(KUBECONFIG_PATH)" $(SCRIPTS_DIR)/httpbin/redeploy.sh

.PHONY: undeploy-httpbin
undeploy-httpbin:
	@ROOT_DIR="$(SANDBOX_DIR)" KUBECONFIG_PATH="$(KUBECONFIG_PATH)" $(SCRIPTS_DIR)/httpbin/undeploy.sh

.PHONY: env-up
env-up: tools certs cluster-up deploy-spire-server deploy-spire-agent deploy-spire-csi deploy-registration deploy-httpbin
	@echo "[env-up] Environment setup complete!"

.PHONY: env-down
env-down: undeploy-httpbin undeploy-registration undeploy-spire-csi undeploy-spire-agent undeploy-spire-server cluster-down clean
	@echo "[env-down] Environment teardown complete!"
