# DNSSEC Keys

This directory contains DNSSEC signing keys for DN42 domains.

## Keys

- `Kscarjit.dn42.+013+24332.*` - DNSSEC keys for scarjit.dn42
- `Klinnenberg.dn42.+013+45642.*` - DNSSEC keys for linnenberg.dn42

## Security

- `.private` files are encrypted with git-crypt
- `.key` files contain public keys and are not encrypted
- Algorithm: ECDSAP256SHA256 (13)

## DS Records

### scarjit.dn42
```
ds-rdata: 24332 13 2 FC25104C8A83F9F3ECEC01DE378211B773C4719EACD01C5B8CE9CF18BCE7974C
```

### linnenberg.dn42
```
ds-rdata: 45642 13 2 9724C6F4DA22D13B61BB44EB8E09468ED4BA34019711C9A4BB9C65E246FB1CF2
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
