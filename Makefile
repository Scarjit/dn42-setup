# DN42 Configuration Deployment Makefile
# Deploy configurations to nephthys router

HOST = nephthys
BIRD_DIR = bird
WG_DIR = wireguard
SYSTEMD_DIR = systemd
SYSTEMD_NETWORKD_DIR = systemd-networkd
REMOTE_BIRD_DIR = /etc/bird
REMOTE_WG_DIR = /etc/wireguard
REMOTE_SYSTEMD_DIR = /etc/systemd/system
REMOTE_SYSTEMD_NETWORKD_DIR = /etc/systemd/network

.PHONY: help deploy deploy-bird deploy-wireguard deploy-systemd status routes clean

help:
	@echo "DN42 Configuration Deployment"
	@echo ""
	@echo "Targets:"
	@echo "  deploy            - Deploy all configurations (bird + wireguard + systemd)"
	@echo "  deploy-bird       - Deploy Bird2 configuration and reconfigure"
	@echo "  deploy-wireguard  - Deploy WireGuard configurations and restart tunnels"
	@echo "  deploy-systemd    - Deploy systemd units and reload"
	@echo "  status            - Show Bird BGP and WireGuard status"
	@echo "  status-bird       - Show Bird BGP status only"
	@echo "  status-wg         - Show WireGuard status only"
	@echo "  routes            - Show all BGP routes (IPv4 and IPv6)"
	@echo "  routes PROTOCOL=X - Show routes from specific protocol (e.g., lenny_v6)"
	@echo ""

deploy: deploy-bird deploy-wireguard deploy-systemd
	@echo "==> All configurations deployed successfully!"

deploy-bird:
	@echo "==> Deploying Bird2 configuration to $(HOST)..."
	@echo "  -> Uploading bird.conf"
	@scp $(BIRD_DIR)/bird.conf $(HOST):/tmp/bird.conf
	@ssh $(HOST) "sudo mv /tmp/bird.conf $(REMOTE_BIRD_DIR)/bird.conf && sudo chmod 644 $(REMOTE_BIRD_DIR)/bird.conf"
	@echo "  -> Creating peers directory"
	@ssh $(HOST) "sudo mkdir -p $(REMOTE_BIRD_DIR)/peers"
	@echo "  -> Uploading peer configurations"
	@for peer in $(BIRD_DIR)/peers/*.conf; do \
		if [ -f "$$peer" ]; then \
			filename=$$(basename "$$peer"); \
			echo "     - $$filename"; \
			scp "$$peer" $(HOST):/tmp/$$filename; \
			ssh $(HOST) "sudo mv /tmp/$$filename $(REMOTE_BIRD_DIR)/peers/$$filename && sudo chmod 644 $(REMOTE_BIRD_DIR)/peers/$$filename"; \
		fi \
	done
	@echo "==> Testing Bird2 configuration..."
	@if ssh $(HOST) "sudo birdc configure check" > /dev/null 2>&1; then \
		echo "  -> Configuration is valid"; \
		echo "==> Applying Bird2 configuration..."; \
		ssh $(HOST) "sudo birdc configure"; \
		echo "==> Bird2 deployment complete!"; \
	else \
		echo "  -> ERROR: Configuration test failed!"; \
		echo "  -> Changes were uploaded but NOT applied"; \
		exit 1; \
	fi

deploy-wireguard:
	@echo "==> Deploying WireGuard configurations to $(HOST)..."
	@for conf in $(WG_DIR)/*.conf; do \
		if [ -f "$$conf" ]; then \
			filename=$$(basename "$$conf"); \
			echo "  -> Uploading $$filename"; \
			scp "$$conf" $(HOST):/tmp/$$filename; \
			ssh $(HOST) "sudo mv /tmp/$$filename $(REMOTE_WG_DIR)/$$filename && sudo chmod 600 $(REMOTE_WG_DIR)/$$filename"; \
		fi \
	done
	@echo "==> Restarting WireGuard tunnels..."
	@for conf in $(WG_DIR)/*.conf; do \
		if [ -f "$$conf" ]; then \
			interface=$$(basename "$$conf" .conf); \
			echo "  -> Restarting $$interface"; \
			ssh $(HOST) "sudo wg-quick down $$interface 2>/dev/null || true && sudo wg-quick up $$interface"; \
		fi \
	done
	@echo "==> WireGuard deployment complete!"

deploy-systemd:
	@echo "==> Deploying systemd units to $(HOST)..."
	@for unit in $(SYSTEMD_DIR)/*; do \
		if [ -f "$$unit" ]; then \
			filename=$$(basename "$$unit"); \
			echo "  -> Uploading $$filename"; \
			scp "$$unit" $(HOST):/tmp/$$filename; \
			ssh $(HOST) "sudo mv /tmp/$$filename $(REMOTE_SYSTEMD_DIR)/$$filename && sudo chmod 644 $(REMOTE_SYSTEMD_DIR)/$$filename"; \
		fi \
	done
	@if [ -d "$(SYSTEMD_NETWORKD_DIR)" ]; then \
		echo "  -> Uploading systemd-networkd configs"; \
		for netfile in $(SYSTEMD_NETWORKD_DIR)/*; do \
			if [ -f "$$netfile" ]; then \
				filename=$$(basename "$$netfile"); \
				echo "     - $$filename"; \
				scp "$$netfile" $(HOST):/tmp/$$filename; \
				ssh $(HOST) "sudo mv /tmp/$$filename $(REMOTE_SYSTEMD_NETWORKD_DIR)/$$filename && sudo chmod 644 $(REMOTE_SYSTEMD_NETWORKD_DIR)/$$filename"; \
			fi \
		done; \
	fi
	@echo "  -> Reloading systemd daemon"
	@ssh $(HOST) "sudo systemctl daemon-reload"
	@echo "  -> Restarting systemd-networkd"
	@ssh $(HOST) "sudo systemctl restart systemd-networkd"
	@echo "  -> Enabling and starting timers"
	@ssh $(HOST) "sudo systemctl enable --now dn42-roa.timer"
	@echo "==> Systemd deployment complete!"

status: status-bird status-wg

status-bird:
	@echo "==> Bird2 BGP Status:"
	@ssh $(HOST) "sudo birdc show protocols" | grep -E "^BIRD|^name|bgp" || true
	@echo ""

status-wg:
	@echo "==> WireGuard Status:"
	@ssh $(HOST) "sudo wg show" || true
	@echo ""

routes:
	@if [ -n "$(PROTOCOL)" ]; then \
		echo "==> Routes from protocol: $(PROTOCOL)"; \
		ssh $(HOST) "sudo birdc show route protocol $(PROTOCOL)" || true; \
	else \
		echo "==> IPv4 Routes:"; \
		ssh $(HOST) "sudo birdc show route" || true; \
		echo ""; \
		echo "==> IPv6 Routes:"; \
		ssh $(HOST) "sudo birdc show route for ::/0" || true; \
	fi
	@echo ""
