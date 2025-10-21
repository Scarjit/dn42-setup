# Auto-Peer API TODO List

## Design: WireGuard Config as Source of Truth

WireGuard configs will contain custom sections:
- `[Challenge]` - random code, ASN, timestamp, status (pending/validated/deployed)
- `[BGP]` - BGP settings (MPBGP, ExtendedNextHop, Local IP, Neighbor IP)
- Standard `[Interface]` and `[Peer]` sections

Flow: Generate skeleton config with `[Challenge]` → User authenticates → Build full config → Deploy

**Port allocation:** Use port 3XXXX where XXXX = last 4 digits of ASN (e.g. AS4242421234 → port 31234)
**Authentication:** GPG signatures only (SSH cannot sign arbitrary text)

## Core Functionality

- [x] Set up basic Rust web server (axum)
- [x] Parse WireGuard config files (with custom sections)
- [x] Write WireGuard config files (with custom sections)
- [x] Implement DN42 registry sync (git clone/pull with auth)
- [x] Parse AS objects from registry (extract ASN, PGP fingerprints, verify keys)
- [x] Generate random challenge codes
- [x] Verify GPG signatures (pure Rust rpgp library)
- [x] Derive WireGuard port from ASN (port 3XXXX from AS424242XXXX) - implemented in design
- [ ] Allocate tunnel IP addresses (IPv4 and IPv6 link-local)
- [ ] Generate BIRD BGP peer configuration files
- [ ] Deploy WireGuard configuration (wg-quick or systemd)
- [ ] Reload BIRD configuration (birdc configure)
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
