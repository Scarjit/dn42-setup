# DN42 Network - SCARJIT-MNT

## Network Information

**ASN**: AS4242420257
**Maintainer**: SCARJIT-MNT
**Contact**: ferdinand@linnenberg.dev

### IPv4 Allocations
- `172.20.13.0/27` - Main network

### IPv6 Allocations
- `fd30:366:4000::/48` - Main network

### Router IPs
- **IPv4**: 172.20.13.1
- **IPv6**: fd30:366:4000::1

## Peering with me

### Prerequisites
- WireGuard tunnel support
- BGP daemon (Bird2, FRRouting, etc.)
- Valid DN42 registry objects (mntner, person, AS, inetnum/inet6num)
- ROA objects registered in the DN42 registry for your prefixes

### My Endpoint
- **Public IPv6**: `2a0a:a543:d3f3:80:be24:11ff:fe5d:a0dc`
- **WireGuard Public Key**: `BsU2BEUpd6aqPqCAyU5kFakuOPRi8i5Ou6v6WJKXEXY=`
- **WireGuard Port**: 51820

### Peering Steps

1. **Contact me** at ferdinand@linnenberg.dev or via DN42 IRC/Matrix
2. **Exchange information**:
   - Your ASN
   - Your WireGuard public key
   - Your WireGuard endpoint (IP:port)
   - Tunnel IPs you'd like to use (link-local or private)
3. **Configure WireGuard tunnel**
4. **Set up BGP session** using tunnel IPs
5. **Test connectivity** and route exchange

### Peering Policy
- Open peering policy - happy to peer with anyone in DN42
- ROA validation enabled
- IPv4 and IPv6 BGP sessions supported
- Multi-protocol BGP over IPv6 supported

## Infrastructure

**Router**: nephthys
**Platform**: Proxmox VM running Ubuntu 25.10
**Location**: Germany
**BGP Daemon**: Bird2 2.17.1

### Features
- **ROA validation enabled** - Only accepts routes with valid ROA entries
- Automatic ROA table updates (every 15 minutes via systemd timer)
- Route filtering with strict validation
- IPv4 and IPv6 support with MP-BGP
- WireGuard connection monitoring

## Repository Structure

```
dn42-setup/
├── bird/
│   ├── bird.conf              # Main Bird2 configuration with ROA validation
│   └── peers/
│       └── lenny.conf         # BGP peer configurations
├── wireguard/
│   ├── wg-lenny.conf          # WireGuard tunnel configs (encrypted with git-crypt)
│   └── README.md              # Encryption instructions
├── systemd/
│   ├── dn42-roa.service       # ROA update service
│   ├── dn42-roa.timer         # ROA update timer (runs every 15m)
│   ├── dn42-wg-monitor.service # WireGuard monitor service
│   └── dn42-wg-monitor.timer  # Monitor timer (runs every 5m)
├── systemd-networkd/
│   ├── dn42-dummy.netdev      # Dummy interface for DN42 IPs
│   └── dn42-dummy.network     # IP address assignment
├── scripts/
│   └── wg-monitor.sh          # WireGuard connection monitoring script
├── registry/                  # DN42 registry objects (separate git repo, not tracked)
├── Makefile                   # Deployment automation
└── README.md                  # This file
```

### Deployment

Run `make help` to see available deployment commands.

### Git-Crypt

WireGuard configuration files containing private keys are encrypted using git-crypt. To decrypt after cloning:

```bash
git-crypt unlock
```

You'll need the authorized GPG private key to decrypt the files.

## Resources

- **Registry**: https://git.dn42.dev/dn42/registry
- **DN42 Wiki**: https://dn42.dev
- **My Registry Objects**: Search for SCARJIT-MNT in the registry
