# DNSSEC Keys

This directory contains DNSSEC signing keys for DN42 domains and reverse zones.

## Keys

### Forward DNS
- `Kscarjit.dn42.+013+24332.*` - DNSSEC keys for scarjit.dn42
- `Klinnenberg.dn42.+013+45642.*` - DNSSEC keys for linnenberg.dn42

### Reverse DNS
- `K13.20.172.in-addr.arpa.+013+25379.*` - DNSSEC keys for 172.20.13.0/27 reverse zone
- `K0.0.0.4.6.6.3.0.3.d.f.ip6.arpa.+013+44028.*` - DNSSEC keys for fd30:366:4000::/48 reverse zone

## Security

- `.private` files are encrypted with git-crypt
- `.key` files contain public keys and are not encrypted
- Algorithm: ECDSAP256SHA256 (13)

## DS Records

### Forward DNS

#### scarjit.dn42
```
ds-rdata: 24332 13 2 FC25104C8A83F9F3ECEC01DE378211B773C4719EACD01C5B8CE9CF18BCE7974C
```

#### linnenberg.dn42
```
ds-rdata: 45642 13 2 9724C6F4DA22D13B61BB44EB8E09468ED4BA34019711C9A4BB9C65E246FB1CF2
```

### Reverse DNS

#### 13.20.172.in-addr.arpa (172.20.13.0/27)
```
ds-rdata: 25379 13 2 580F675D915B246E96AB083E843056162DC82C106717D58A19562B0EEE69C6E8
```

#### 0.0.0.4.6.6.3.0.3.d.f.ip6.arpa (fd30:366:4000::/48)
```
ds-rdata: 44028 13 2 17C5C76DAFB8C55C29E7236A2150169681A51A463E8D695B2383BB3DF9DC5F72
```

## Usage

These keys are currently used for DS record delegation in the DN42 registry.
They will be needed if/when we implement full DNSSEC signing of zones.

## Key Rotation

DNSSEC keys should be rotated periodically. To generate new keys:

```bash
ssh nephthys "cd /tmp && dnssec-keygen -a ECDSAP256SHA256 -n ZONE <domain>"
ssh nephthys "cd /tmp && dnssec-dsfromkey <keyfile>"
```

Then update the DS records in the registry.
