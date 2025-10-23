# AutoPeer Frontend

A simple, clean web interface for the DN42 AutoPeer API.

## Stack

### Core
- **Svelte 5** - Reactive UI framework with runes
- **TypeScript** - Type safety for API interactions
- **Vite** - Fast development and build tooling
- **TailwindCSS** - Utility-first styling

### Libraries
- **Zod** - Runtime validation for API requests/responses
- **@tanstack/svelte-query** - API state management and caching
- **shadcn-svelte** - Headless, accessible component primitives
- **lucide-svelte** - Clean icon library

## Goals

Build a straightforward interface for DN42 network operators to automatically establish peering sessions.

### User Flow

1. **Initialize Peering**
   - User enters their ASN
   - API generates a challenge code only
   - User receives challenge to sign

2. **Verify Ownership**
   - User signs the challenge with their DN42 registry GPG key
   - User provides signed challenge + their WireGuard public key + endpoint
   - API validates signature against DN42 registry
   - API generates WireGuard keypair for the peer (after verification)
   - User receives JWT token + our WireGuard public key + complete config

3. **Deploy Configuration**
   - Authenticated user deploys the peering
   - WireGuard tunnel and BGP session are configured

4. **Manage Peering**
   - Update endpoint or other settings
   - View current configuration
   - Delete peering

## Philosophy

**Keep It Simple, Stupid (KISS)**

- Single-page application (no complex routing)
- Manual form handling (no form library overhead)
- Clean, minimal UI focused on the peering workflow
- Copy-paste friendly (configs, tokens, challenges)
- Clear error messages and validation feedback

## Development

```bash
npm install
npm run dev
```

## Build

```bash
npm run build
```

The built assets will be in `dist/` and can be served statically.
