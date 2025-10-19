# DN42 Configuration Deployment Makefile
# Deploy configurations to nephthys router

HOST = nephthys
BIRD_DIR = bird
WG_DIR = wireguard
SYSTEMD_DIR = systemd
SYSTEMD_NETWORKD_DIR = systemd-networkd
ALICE_LG_DIR = alice-lg
NGINX_DIR = nginx
DNS_DIR = dns
IPTABLES_DIR = iptables
REMOTE_BIRD_DIR = /etc/bird
REMOTE_WG_DIR = /etc/wireguard
REMOTE_SYSTEMD_DIR = /etc/systemd/system
REMOTE_SYSTEMD_NETWORKD_DIR = /etc/systemd/network
REMOTE_ALICE_LG_DIR = /etc/alice-lg
REMOTE_BIRDWATCHER_DIR = /etc/birdwatcher
REMOTE_NGINX_DIR = /etc/nginx/sites-available
REMOTE_UNBOUND_DIR = /etc/unbound
REMOTE_IPTABLES_DIR = /etc/iptables

.PHONY: help deploy deploy-bird deploy-wireguard deploy-systemd deploy-alice-lg deploy-nginx deploy-dns deploy-iptables status routes clean

help:
	@echo "DN42 Configuration Deployment"
	@echo ""
	@echo "Targets:"
	@echo "  deploy            - Deploy all configurations (bird + wireguard + systemd)"
	@echo "  deploy-bird       - Deploy Bird2 configuration and reconfigure"
	@echo "  deploy-wireguard  - Deploy WireGuard configurations and restart tunnels"
	@echo "  deploy-systemd    - Deploy systemd units and reload"
	@echo "  deploy-alice-lg   - Deploy Alice Looking Glass and Birdwatcher configs"
	@echo "  deploy-nginx      - Deploy nginx configurations and reload"
	@echo "  deploy-dns        - Deploy unbound DNS configuration and restart service"
	@echo "  deploy-iptables   - Deploy iptables rules for DN42 NAT"
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
	@echo "  -> Cleaning old peer configurations"
	@ssh $(HOST) "sudo rm -f $(REMOTE_BIRD_DIR)/peers/*.conf"
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
	@echo "  -> Cleaning old WireGuard configurations"
	@ssh $(HOST) "sudo rm -f $(REMOTE_WG_DIR)/*.conf"
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
	@ssh $(HOST) "sudo birdc show protocols" | grep -E "BGP|^Name" || true
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

deploy-alice-lg: deploy-systemd
	@echo "==> Deploying Alice-LG and Birdwatcher configurations to $(HOST)..."
	@echo "  -> Uploading birdwatcher.conf"
	@scp $(ALICE_LG_DIR)/birdwatcher.conf $(HOST):/tmp/birdwatcher.conf
	@ssh $(HOST) "sudo mv /tmp/birdwatcher.conf $(REMOTE_BIRDWATCHER_DIR)/birdwatcher.conf && sudo chmod 644 $(REMOTE_BIRDWATCHER_DIR)/birdwatcher.conf"
	@echo "  -> Uploading alice.conf"
	@scp $(ALICE_LG_DIR)/alice.conf $(HOST):/tmp/alice.conf
	@ssh $(HOST) "sudo mv /tmp/alice.conf $(REMOTE_ALICE_LG_DIR)/alice.conf && sudo chmod 644 $(REMOTE_ALICE_LG_DIR)/alice.conf"
	@echo "  -> Enabling and starting services"
	@ssh $(HOST) "sudo systemctl daemon-reload"
	@ssh $(HOST) "sudo systemctl enable --now birdwatcher.service"
	@ssh $(HOST) "sudo systemctl enable --now alice-lg.service"
	@echo "==> Alice-LG deployment complete!"
	@echo "  -> Birdwatcher status:"
	@ssh $(HOST) "sudo systemctl status birdwatcher.service --no-pager -l" || true
	@echo "  -> Alice-LG status:"
	@ssh $(HOST) "sudo systemctl status alice-lg.service --no-pager -l" || true

deploy-nginx:
	@echo "==> Deploying nginx configurations to $(HOST)..."
	@for conf in $(NGINX_DIR)/*.conf; do \
		if [ -f "$$conf" ]; then \
			filename=$$(basename "$$conf"); \
			echo "  -> Uploading $$filename"; \
			scp "$$conf" $(HOST):/tmp/$$filename; \
			ssh $(HOST) "sudo mv /tmp/$$filename $(REMOTE_NGINX_DIR)/$$filename && sudo chmod 644 $(REMOTE_NGINX_DIR)/$$filename"; \
			sitename=$$(basename "$$conf" .conf); \
			echo "  -> Enabling site: $$sitename"; \
			ssh $(HOST) "sudo ln -sf $(REMOTE_NGINX_DIR)/$$filename /etc/nginx/sites-enabled/$$filename 2>/dev/null || true"; \
		fi \
	done
	@echo "  -> Testing nginx configuration"
	@if ssh $(HOST) "sudo nginx -t" > /dev/null 2>&1; then \
		echo "  -> Configuration is valid"; \
		echo "  -> Reloading nginx"; \
		ssh $(HOST) "sudo systemctl reload nginx"; \
		echo "==> Nginx deployment complete!"; \
	else \
		echo "  -> ERROR: Nginx configuration test failed!"; \
		echo "  -> Changes were uploaded but NOT applied"; \
		exit 1; \
	fi

deploy-dns:
	@echo "==> Deploying unbound DNS configuration to $(HOST)..."
	@echo "  -> Uploading unbound.conf"
	@scp $(DNS_DIR)/unbound.conf $(HOST):/tmp/unbound.conf
	@ssh $(HOST) "sudo mv /tmp/unbound.conf $(REMOTE_UNBOUND_DIR)/unbound.conf && sudo chmod 644 $(REMOTE_UNBOUND_DIR)/unbound.conf"
	@echo "  -> Testing unbound configuration"
	@if ssh $(HOST) "sudo unbound-checkconf $(REMOTE_UNBOUND_DIR)/unbound.conf" > /dev/null 2>&1; then \
		echo "  -> Configuration is valid"; \
		echo "  -> Restarting unbound service"; \
		ssh $(HOST) "sudo systemctl restart unbound"; \
		echo "  -> Enabling unbound service"; \
		ssh $(HOST) "sudo systemctl enable unbound"; \
		echo "==> DNS deployment complete!"; \
		echo "  -> Unbound status:"; \
		ssh $(HOST) "sudo systemctl status unbound --no-pager -l" || true; \
	else \
		echo "  -> ERROR: Unbound configuration test failed!"; \
		echo "  -> Changes were uploaded but NOT applied"; \
		exit 1; \
	fi

deploy-iptables: deploy-systemd
	@echo "==> Deploying iptables rules for DN42 NAT to $(HOST)..."
	@echo "  -> Creating iptables directory"
	@ssh $(HOST) "sudo mkdir -p $(REMOTE_IPTABLES_DIR)"
	@echo "  -> Uploading iptables.rules"
	@scp $(IPTABLES_DIR)/iptables.rules $(HOST):/tmp/iptables.rules
	@ssh $(HOST) "sudo mv /tmp/iptables.rules $(REMOTE_IPTABLES_DIR)/iptables.rules && sudo chmod 644 $(REMOTE_IPTABLES_DIR)/iptables.rules"
	@echo "  -> Uploading ip6tables.rules"
	@scp $(IPTABLES_DIR)/ip6tables.rules $(HOST):/tmp/ip6tables.rules
	@ssh $(HOST) "sudo mv /tmp/ip6tables.rules $(REMOTE_IPTABLES_DIR)/ip6tables.rules && sudo chmod 644 $(REMOTE_IPTABLES_DIR)/ip6tables.rules"
	@echo "  -> Applying iptables rules"
	@ssh $(HOST) "sudo iptables-restore < $(REMOTE_IPTABLES_DIR)/iptables.rules"
	@ssh $(HOST) "sudo ip6tables-restore < $(REMOTE_IPTABLES_DIR)/ip6tables.rules"
	@echo "  -> Enabling iptables-restore service"
	@ssh $(HOST) "sudo systemctl enable --now iptables-restore.service"
	@echo "==> Iptables deployment complete!"
	@echo "  -> Service status:"
	@ssh $(HOST) "sudo systemctl status iptables-restore.service --no-pager -l" || true
