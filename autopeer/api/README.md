# AutoPeer API

Automated DN42 peering with GPG authentication and JWT tokens.

## Endpoints

- `POST /peering/init` - Start peering, get challenge
- `POST /peering/verify` - Submit GPG-signed challenge, get JWT
- `POST /peering/deploy` - Deploy WireGuard + BIRD configs
- `GET /peering/config?token=...` - Get current config
- `PATCH /peering/update` - Update endpoint and re-deploy
- `DELETE /peering?token=...` - Remove peering

## Workflow

1. Call `/init` with ASN → get challenge
2. Sign challenge with PGP key from DN42 registry
3. Submit to `/verify` → get JWT token
4. Call `/deploy` with JWT → peering active
5. Update/delete as needed using JWT

## Config

Required env vars:
```bash
JWT_SECRET=secret
DN42_GIT_USERNAME=user
DN42_GIT_TOKEN=token
```

Optional (with defaults):
```bash
MY_ASN=4242420257
BIND_ADDRESS=127.0.0.1:3000
RUST_LOG=info
```

## Features

- GPG auth via DN42 registry
- Auto WireGuard + BIRD config
- IPv6 link-local from ASNs: `fe80::{peer}:{my}:{0/1}`
- WireGuard port from ASN: `30000 + (asn % 10000)`
- Zero database, configs are source of truth
