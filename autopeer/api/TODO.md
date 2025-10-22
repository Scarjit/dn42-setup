# Auto-Peer API TODO List

## Design: WireGuard Config as Source of Truth

WireGuard configs will contain custom sections:
- `[Challenge]` - random code and ASN for authentication
- `[BGP]` - BGP settings (MPBGP, ExtendedNextHop, Local IP, Neighbor IP)
- Standard `[Interface]` and `[Peer]` sections

Flow: Generate skeleton config with `[Challenge]` → User authenticates with GPG signature → Issue JWT token for ASN → Use JWT to deploy config

**Port allocation:** Use port 3XXXX where XXXX = last 4 digits of ASN (e.g. AS4242421234 → port 31234)
**Authentication:** GPG signatures for initial auth, then JWT tokens for subsequent API calls

## Core Functionality

- [x] Set up basic Rust web server (axum)
- [x] Parse WireGuard config files (with custom sections)
- [x] Write WireGuard config files (with custom sections)
- [x] Implement DN42 registry sync (git clone/pull with auth)
- [x] Parse AS objects from registry (extract ASN, PGP fingerprints, verify keys)
- [x] Generate random challenge codes
- [x] Verify GPG signatures (pure Rust rpgp library)
- [x] Derive WireGuard port from ASN (port 3XXXX from AS424242XXXX)
- [x] Allocate tunnel IP addresses (IPv6 link-local from ASN: fe80::{peer}:{my}:{0/1})
- [x] Generate BIRD BGP peer configuration files
- [x] Implement JWT token generation and verification for authenticated ASNs
- [x] Generate WireGuard keypairs for each config
- [x] Deploy WireGuard configuration (wg-quick up/down, write to /etc/wireguard/)
- [x] Deploy BIRD configuration (write to /etc/bird/peers/, birdc configure)
- [ ] Create API endpoints:
  - POST /peering/init (create skeleton config with challenge)
  - POST /peering/verify (submit signature, build full config)
  - POST /peering/deploy (deploy validated config)
  - GET /peering/config/:id (retrieve their peer config)
- [ ] Add input validation and sanitization
- [ ] Add logging and error handling
- [ ] Create config file for API settings (IP pools, config paths, etc)

## Optional/Future

- [ ] Rate limiting
- [ ] Peer removal/deprovisioning endpoint
- [ ] Admin API for manual overrides
- [ ] Metrics and monitoring
