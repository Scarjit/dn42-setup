# AutoPeer API

Automated DN42 peering system with GPG-based authentication and JWT tokens.

## Features

- **GPG Authentication**: Verify peer identity using PGP signatures against DN42 registry
- **Automated Configuration**: Generate WireGuard and BIRD BGP configs automatically
- **JWT Tokens**: Secure API access after initial authentication
- **IPv6 Link-Local**: Automatic address allocation from ASNs (`fe80::{peer}:{my}:{0/1}`)
- **Port Derivation**: WireGuard ports derived from ASN (port 3XXXX)
- **Zero Database**: WireGuard configs are the source of truth

## API Endpoints

### POST /peering/init
Initialize a new peering session.

**Request:**
```json
{
  "asn": 4242421234
}
```

**Response:**
```json
{
  "challenge": "AUTOPEER-4242421234-a1b2c3d4...",
  "peer_public_key": "base64key==",
  "wireguard_config": "[Interface]\n..."
}
```

### POST /peering/verify
Verify GPG-signed challenge and receive JWT token.

**Request:**
```json
{
  "asn": 4242421234,
  "signed_challenge": "-----BEGIN PGP SIGNED MESSAGE-----\n...",
  "public_key": "-----BEGIN PGP PUBLIC KEY BLOCK-----\n...",
  "wg_public_key": "base64key==",
  "endpoint": "192.0.2.1:31234"
}
```

**Response:**
```json
{
  "token": "eyJhbGciOiJIUzI1NiIs...",
  "wireguard_config": "[Interface]\n[Peer]\n[BGP]\n..."
}
```

### POST /peering/deploy
Deploy verified configuration (requires JWT).

**Request:**
```json
{
  "asn": 4242421234,
  "token": "eyJhbGciOiJIUzI1NiIs..."
}
```

**Response:**
```json
{
  "status": "deployed",
  "interface": "wg-as4242421234"
}
```

### GET /peering/config?token=...
Retrieve current configuration (ASN extracted from JWT).

**Response:**
```json
{
  "wireguard_config": "[Interface]\n[Peer]\n[BGP]\n..."
}
```

### PATCH /peering/update
Update and re-deploy configuration (requires JWT).

**Request:**
```json
{
  "token": "eyJhbGciOiJIUzI1NiIs...",
  "endpoint": "198.51.100.1:31234"
}
```

**Response:**
```json
{
  "status": "updated"
}
```

### DELETE /peering?token=...
Remove peering configuration (ASN extracted from JWT).

**Response:**
```json
{
  "status": "deleted"
}
```

## Configuration

Environment variables (with defaults):

```bash
# Required
JWT_SECRET=your-secret-key
DN42_GIT_USERNAME=your-username
DN42_GIT_TOKEN=your-token

# Optional
MY_ASN=4242420257
BIND_ADDRESS=127.0.0.1:3000
DATA_PENDING_DIR=./data/pending
DATA_VERIFIED_DIR=./data/verified
DN42_REGISTRY_URL=https://git.dn42.dev/dn42/registry
DN42_REGISTRY_PATH=./data/dn42-registry
RUST_LOG=info
```

## Workflow

1. **Initialize**: Peer calls `/peering/init` with their ASN
2. **Sign Challenge**: Peer signs the challenge with their PGP key from DN42 registry
3. **Verify**: Peer submits signed challenge to `/peering/verify`, receives JWT token
4. **Deploy**: Peer calls `/peering/deploy` with JWT to activate the peering
5. **Manage**: Peer can update endpoint or delete peering using JWT

## Technical Details

### WireGuard Configuration Format

Configs include custom sections for automation:

```ini
[Interface]
Address = fe80::1234:257:0/64
PrivateKey = ...
ListenPort = 31234

[Peer]
PublicKey = ...
Endpoint = 192.0.2.1:31234
AllowedIPs = 0.0.0.0/0, ::/0
PersistentKeepalive = 25

[BGP]
MPBGP = true
ExtendedNextHop = true
Local = fe80::1234:257:0
Neighbor = fe80::1234:257:1
```

### Authentication Flow

1. GPG signature verification against DN42 registry
2. JWT token issued (7-day expiration)
3. All subsequent operations authenticated via JWT
4. ASN embedded in JWT claims

### IP Allocation

IPv6 link-local addresses derived from ASNs:
- Format: `fe80::{peer_last4}:{my_last4}:{0/1}/64`
- Example: AS4242421234 ↔ AS4242420257 → `fe80::1234:257:0` and `fe80::1234:257:1`

### Port Allocation

WireGuard ports derived from ASN:
- Formula: `30000 + (asn % 10000)`
- Example: AS4242421234 → port 31234

## Deployment Locations

- WireGuard: `/etc/wireguard/{interface}.conf`
- BIRD: `/etc/bird/peers/autopeer_as{asn}.conf`
- Configs: `./data/{pending,verified}/{interface}.conf`

## Development

```bash
# Run tests
cargo test

# Run server
cargo run

# With logging
RUST_LOG=debug cargo run
```

## License

Internal project for DN42 network automation.
