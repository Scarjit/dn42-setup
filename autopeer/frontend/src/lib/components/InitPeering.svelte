<script lang="ts">
  import { createMutation } from '@tanstack/svelte-query';
  import { Copy, CheckCircle2, AlertCircle } from 'lucide-svelte';
  import { apiClient } from '$lib/api/client';
  import type { InitResponse } from '$lib/api/types';

  // Load state from localStorage on mount
  let asn = $state(typeof window !== 'undefined' ? localStorage.getItem('autopeer_asn') || '' : '');
  let challenge = $state<string | null>(typeof window !== 'undefined' ? localStorage.getItem('autopeer_challenge') : null);
  let pgpFingerprint = $state<string | null>(typeof window !== 'undefined' ? localStorage.getItem('autopeer_pgp_fingerprint') : null);
  let copied = $state(false);
  let copiedCommand = $state(false);
  let copiedExportCommand = $state(false);

  // Save state to localStorage whenever it changes
  $effect(() => {
    if (typeof window !== 'undefined') {
      if (asn) {
        localStorage.setItem('autopeer_asn', asn);
      } else {
        localStorage.removeItem('autopeer_asn');
      }
    }
  });

  $effect(() => {
    if (typeof window !== 'undefined') {
      if (challenge) {
        localStorage.setItem('autopeer_challenge', challenge);
      } else {
        localStorage.removeItem('autopeer_challenge');
      }
    }
  });

  $effect(() => {
    if (typeof window !== 'undefined') {
      if (pgpFingerprint) {
        localStorage.setItem('autopeer_pgp_fingerprint', pgpFingerprint);
      } else {
        localStorage.removeItem('autopeer_pgp_fingerprint');
      }
    }
  });

  // Verification form state
  let signedChallenge = $state('');
  let publicKey = $state('');
  let wgPublicKey = $state('');
  let endpoint = $state('');

  // Validation helpers
  function isValidSignedChallenge(value: string): boolean {
    return value.includes('-----BEGIN PGP SIGNED MESSAGE-----') &&
           value.includes('-----BEGIN PGP SIGNATURE-----') &&
           value.includes('-----END PGP SIGNATURE-----');
  }

  function isValidPublicKey(value: string): boolean {
    return value.includes('-----BEGIN PGP PUBLIC KEY BLOCK-----') &&
           value.includes('-----END PGP PUBLIC KEY BLOCK-----');
  }

  function isValidWgPublicKey(value: string): boolean {
    // WireGuard public keys are 44 characters base64 (typically ending with =)
    return /^[A-Za-z0-9+/]{42,44}={0,2}$/.test(value.trim());
  }

  function isValidEndpoint(value: string): boolean {
    // IPv4:port, [IPv6]:port, or domain:port
    const ipv4Pattern = /^(\d{1,3}\.){3}\d{1,3}:\d{1,5}$/;
    const ipv6Pattern = /^\[([0-9a-fA-F:]+)\]:\d{1,5}$/;
    const domainPattern = /^[a-zA-Z0-9.-]+:\d{1,5}$/;
    return ipv4Pattern.test(value) || ipv6Pattern.test(value) || domainPattern.test(value);
  }

  const initMutation = createMutation(() => ({
    mutationFn: async (asnNumber: number) => {
      return apiClient.initPeering({ asn: asnNumber });
    },
    onSuccess: (data: InitResponse) => {
      challenge = data.challenge;
      pgpFingerprint = data.pgp_fingerprint;
    },
  }));

  const verifyMutation = createMutation(() => ({
    mutationFn: async () => {
      return apiClient.verifyPeering({
        asn: parseInt(asn, 10),
        signed_challenge: signedChallenge,
        public_key: publicKey,
        wg_public_key: wgPublicKey,
        endpoint: endpoint,
      });
    },
  }));

  function handleSubmit(e: Event) {
    e.preventDefault();
    const asnNumber = parseInt(asn, 10);

    if (isNaN(asnNumber) || asnNumber < 4242420000 || asnNumber > 4242423999) {
      return;
    }

    initMutation.mutate(asnNumber);
  }

  function handleVerifySubmit(e: Event) {
    e.preventDefault();
    verifyMutation.mutate();
  }

  async function copyToClipboard() {
    if (!challenge) return;

    try {
      await navigator.clipboard.writeText(challenge);
      copied = true;
      setTimeout(() => {
        copied = false;
      }, 2000);
    } catch (err) {
      console.error('Failed to copy:', err);
    }
  }

  async function copyGpgCommand() {
    if (!challenge) return;

    // If we have a PGP fingerprint, use it with --local-user flag
    const command = pgpFingerprint
      ? `echo "${challenge}" | gpg --local-user ${pgpFingerprint} --clearsign`
      : `echo "${challenge}" | gpg --clearsign`;

    try {
      await navigator.clipboard.writeText(command);
      copiedCommand = true;
      setTimeout(() => {
        copiedCommand = false;
      }, 2000);
    } catch (err) {
      console.error('Failed to copy:', err);
    }
  }

  async function copyGpgExportCommand() {
    const command = pgpFingerprint
      ? `gpg --armor --export ${pgpFingerprint}`
      : 'gpg --armor --export YOUR_KEY_ID';

    try {
      await navigator.clipboard.writeText(command);
      copiedExportCommand = true;
      setTimeout(() => {
        copiedExportCommand = false;
      }, 2000);
    } catch (err) {
      console.error('Failed to copy:', err);
    }
  }
</script>

<div class="max-w-2xl mx-auto p-6">
  <div class="mb-8">
    <h1 class="text-3xl font-bold mb-2">DN42 AutoPeer</h1>
    <p class="text-gray-600">Automatic peering setup for DN42 network</p>
  </div>

  {#if !challenge}
    <div class="bg-white rounded-lg shadow-md p-6">
      <h2 class="text-xl font-semibold mb-4">Step 1: Initialize Peering</h2>
      <p class="text-gray-600 mb-6">
        Enter your ASN to begin the peering process. We'll generate a challenge code that you'll need to sign with your DN42 registry GPG key.
      </p>

      <form onsubmit={handleSubmit} class="space-y-4">
        <div>
          <label for="asn" class="block text-sm font-medium text-gray-700 mb-2">
            Your ASN
          </label>
          <input
            type="number"
            id="asn"
            bind:value={asn}
            placeholder="4242420257"
            min="4242420000"
            max="4242423999"
            required
            disabled={initMutation.isPending}
            class="w-full px-4 py-2 border border-gray-300 rounded-md focus:ring-2 focus:ring-blue-500 focus:border-transparent disabled:bg-gray-100 disabled:cursor-not-allowed"
          />
          <p class="text-xs text-gray-500 mt-1">
            Valid ASN range: 4242420000 - 4242423999
          </p>
        </div>

        {#if initMutation.isError}
          <div class="flex items-start gap-2 p-4 bg-red-50 border border-red-200 rounded-md">
            <AlertCircle class="w-5 h-5 text-red-600 flex-shrink-0 mt-0.5" />
            <div class="text-sm text-red-800">
              <p class="font-medium">Failed to initialize peering</p>
              <p class="mt-1">{initMutation.error?.message || 'Unknown error'}</p>
            </div>
          </div>
        {/if}

        <button
          type="submit"
          disabled={initMutation.isPending || !asn}
          class="w-full px-4 py-2 bg-blue-600 text-white font-medium rounded-md hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 disabled:bg-gray-400 disabled:cursor-not-allowed transition-colors"
        >
          {initMutation.isPending ? 'Generating Challenge...' : 'Generate Challenge'}
        </button>
      </form>
    </div>
  {:else}
    <div class="bg-white rounded-lg shadow-md p-6">
      <div class="flex items-center gap-2 mb-4">
        <CheckCircle2 class="w-6 h-6 text-green-600" />
        <h2 class="text-xl font-semibold">Challenge Generated</h2>
      </div>

      <p class="text-gray-600 mb-4">
        Sign this challenge with your DN42 registry GPG key to prove ownership of ASN {asn}.
      </p>

      <div class="bg-gray-50 rounded-md p-4 mb-4">
        <div class="flex items-start justify-between gap-4">
          <code class="text-sm font-mono break-all flex-1">{challenge}</code>
          <button
            onclick={copyToClipboard}
            class="flex-shrink-0 p-2 hover:bg-gray-200 rounded-md transition-colors"
            title="Copy to clipboard"
          >
            {#if copied}
              <CheckCircle2 class="w-5 h-5 text-green-600" />
            {:else}
              <Copy class="w-5 h-5 text-gray-600" />
            {/if}
          </button>
        </div>
      </div>

      <div class="bg-blue-50 border border-blue-200 rounded-md p-4 mb-4">
        <p class="text-sm text-blue-900 font-medium mb-2">How to sign the challenge:</p>
        <ol class="text-sm text-blue-800 space-y-1 list-decimal list-inside">
          <li>Copy the challenge code or command below</li>
          <li>Run the GPG command in your terminal</li>
          <li>Copy the signed output (everything including -----BEGIN/END PGP SIGNATURE-----)</li>
          <li>Continue to the next step with your signed challenge</li>
        </ol>
      </div>

      <div class="mb-6">
        <button
          onclick={copyGpgCommand}
          class="w-full flex items-center justify-between gap-4 px-4 py-3 bg-gray-50 hover:bg-gray-100 border border-gray-300 rounded-md transition-colors"
        >
          <code class="text-sm font-mono text-left break-all">
            {#if pgpFingerprint}
              echo "{challenge}" | gpg --local-user {pgpFingerprint} --clearsign
            {:else}
              echo "{challenge}" | gpg --clearsign
            {/if}
          </code>
          {#if copiedCommand}
            <CheckCircle2 class="w-5 h-5 text-green-600 flex-shrink-0" />
          {:else}
            <Copy class="w-5 h-5 text-gray-600 flex-shrink-0" />
          {/if}
        </button>
        <p class="text-xs text-gray-500 mt-2">
          Click to copy the full command. Paste and run it in your terminal to sign the challenge with your DN42 GPG key.
        </p>
      </div>

      <div class="flex justify-start">
        <button
          onclick={() => {
            challenge = null;
            pgpFingerprint = null;
            asn = '';
            // Clear localStorage
            if (typeof window !== 'undefined') {
              localStorage.removeItem('autopeer_asn');
              localStorage.removeItem('autopeer_challenge');
              localStorage.removeItem('autopeer_pgp_fingerprint');
            }
          }}
          class="px-4 py-2 bg-gray-200 text-gray-800 font-medium rounded-md hover:bg-gray-300 focus:outline-none focus:ring-2 focus:ring-gray-500 focus:ring-offset-2 transition-colors"
        >
          Start Over
        </button>
      </div>
    </div>

    <!-- Step 2: Verification Form (Always Visible) -->
    <div class="bg-white rounded-lg shadow-md p-6 mt-6">
      <h2 class="text-xl font-semibold mb-4">Step 2: Verify Ownership</h2>

      <form onsubmit={handleVerifySubmit} class="space-y-4">
        <div>
          <label for="signedChallenge" class="block text-sm font-medium text-gray-700 mb-2">
            Signed Challenge
          </label>
          <textarea
            id="signedChallenge"
            bind:value={signedChallenge}
            placeholder="-----BEGIN PGP SIGNED MESSAGE-----&#10;Hash: SHA512&#10;&#10;AUTOPEER-...&#10;-----BEGIN PGP SIGNATURE-----&#10;...&#10;-----END PGP SIGNATURE-----"
            rows="8"
            required
            disabled={verifyMutation.isPending || !challenge}
            class="w-full px-4 py-2 border rounded-md font-mono text-sm focus:ring-2 focus:ring-blue-500 focus:border-transparent disabled:bg-gray-100 disabled:cursor-not-allowed {signedChallenge && isValidSignedChallenge(signedChallenge) ? 'border-green-500' : 'border-gray-300'}"
          />
          <p class="text-xs text-gray-500 mt-1">
            Paste the complete output from the GPG command (including BEGIN/END lines)
          </p>
        </div>

        <div>
          <label for="publicKey" class="block text-sm font-medium text-gray-700 mb-2">
            Your GPG Public Key
          </label>
          <textarea
            id="publicKey"
            bind:value={publicKey}
            placeholder="-----BEGIN PGP PUBLIC KEY BLOCK-----&#10;...&#10;-----END PGP PUBLIC KEY BLOCK-----"
            rows="6"
            required
            disabled={verifyMutation.isPending || !challenge}
            class="w-full px-4 py-2 border rounded-md font-mono text-sm focus:ring-2 focus:ring-blue-500 focus:border-transparent disabled:bg-gray-100 disabled:cursor-not-allowed {publicKey && isValidPublicKey(publicKey) ? 'border-green-500' : 'border-gray-300'}"
          />
          <button
            type="button"
            onclick={copyGpgExportCommand}
            class="mt-2 w-full flex items-center justify-between gap-4 px-4 py-2 bg-gray-50 hover:bg-gray-100 border border-gray-300 rounded-md transition-colors text-left"
          >
            <code class="text-sm font-mono break-all">
              gpg --armor --export {pgpFingerprint || 'YOUR_KEY_ID'}
            </code>
            {#if copiedExportCommand}
              <CheckCircle2 class="w-5 h-5 text-green-600 flex-shrink-0" />
            {:else}
              <Copy class="w-5 h-5 text-gray-600 flex-shrink-0" />
            {/if}
          </button>
          <p class="text-xs text-gray-500 mt-1">
            Click to copy the export command to get your public key
          </p>
        </div>

        <div>
          <label for="wgPublicKey" class="block text-sm font-medium text-gray-700 mb-2">
            Your WireGuard Public Key
          </label>
          <input
            type="text"
            id="wgPublicKey"
            bind:value={wgPublicKey}
            placeholder="your_base64_public_key="
            required
            disabled={verifyMutation.isPending || !challenge}
            class="w-full px-4 py-2 border rounded-md font-mono text-sm focus:ring-2 focus:ring-blue-500 focus:border-transparent disabled:bg-gray-100 disabled:cursor-not-allowed {wgPublicKey && isValidWgPublicKey(wgPublicKey) ? 'border-green-500' : 'border-gray-300'}"
          />
        </div>

        <div>
          <label for="endpoint" class="block text-sm font-medium text-gray-700 mb-2">
            Your WireGuard Endpoint
          </label>
          <input
            type="text"
            id="endpoint"
            bind:value={endpoint}
            placeholder="IPv4:port, [IPv6]:port, or domain:port (e.g., 1.2.3.4:51820, [2001:db8::1]:51820, or peer.example.com:51820)"
            required
            disabled={verifyMutation.isPending || !challenge}
            class="w-full px-4 py-2 border rounded-md focus:ring-2 focus:ring-blue-500 focus:border-transparent disabled:bg-gray-100 disabled:cursor-not-allowed {endpoint && isValidEndpoint(endpoint) ? 'border-green-500' : 'border-gray-300'}"
          />
        </div>

        {#if verifyMutation.isError}
          <div class="flex items-start gap-2 p-4 bg-red-50 border border-red-200 rounded-md">
            <AlertCircle class="w-5 h-5 text-red-600 flex-shrink-0 mt-0.5" />
            <div class="text-sm text-red-800">
              <p class="font-medium">Verification failed</p>
              <p class="mt-1">{verifyMutation.error?.message || 'Unknown error'}</p>
            </div>
          </div>
        {/if}

        <button
          type="submit"
          disabled={verifyMutation.isPending || !challenge || !signedChallenge || !publicKey || !wgPublicKey || !endpoint}
          class="w-full px-4 py-2 bg-blue-600 text-white font-medium rounded-md hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 disabled:bg-gray-400 disabled:cursor-not-allowed transition-colors"
        >
          {verifyMutation.isPending ? 'Verifying...' : 'Verify & Complete'}
        </button>
      </form>
    </div>
  {/if}
</div>
