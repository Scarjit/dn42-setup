#!/bin/bash
# WireGuard Connection Monitor for DN42

INTERFACE="wg-lenny"
PEER_IP="fdba:4d7b:fb4c::1"
LOG_FILE="/var/log/dn42-wg-monitor.log"

echo "=== WireGuard Monitor - $(date) ===" | tee -a "$LOG_FILE"

# Check WireGuard status
echo "## WireGuard Status:" | tee -a "$LOG_FILE"
wg show "$INTERFACE" | tee -a "$LOG_FILE"

# Check handshake age
HANDSHAKE=$(wg show "$INTERFACE" latest-handshakes | awk '{print $2}')
NOW=$(date +%s)
AGE=$((NOW - HANDSHAKE))
echo "Handshake age: ${AGE}s" | tee -a "$LOG_FILE"

if [ $AGE -gt 180 ]; then
    echo "WARNING: Handshake is older than 3 minutes!" | tee -a "$LOG_FILE"
fi

# Test connectivity
echo "## Connectivity Test:" | tee -a "$LOG_FILE"
if ping6 -c 3 -W 2 "$PEER_IP" > /dev/null 2>&1; then
    echo "✓ Peer $PEER_IP is reachable" | tee -a "$LOG_FILE"
else
    echo "✗ Peer $PEER_IP is NOT reachable" | tee -a "$LOG_FILE"
fi

# Check BGP status
echo "## BGP Status:" | tee -a "$LOG_FILE"
birdc show protocols lenny_v6 | grep -E "lenny_v6|BGP state" | tee -a "$LOG_FILE"

echo "" | tee -a "$LOG_FILE"
