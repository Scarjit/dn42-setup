<script lang="ts">
  import { createMutation, createQuery } from '@tanstack/svelte-query';
  import { apiClient } from '../api/client';
  import type { DeploymentInfo } from '../api/types';
  import { Check, Loader2, WifiIcon, Globe } from 'lucide-svelte';
  import LogoutButton from './LogoutButton.svelte';

  interface Props {
    asn: number;
    onDeploymentSuccess?: (deployment: DeploymentInfo) => void;
    onLogout?: () => void;
  }

  let { asn, onDeploymentSuccess, onLogout }: Props = $props();

  let wgPublicKey = $state('');
  let endpoint = $state('');
  let isEditing = $state(false);

  // Query existing deployment status
  const statusQuery = createQuery(() => ({
    queryKey: ['deployment-status', asn],
    queryFn: async () => {
      const token = localStorage.getItem('autopeer_token');
      if (!token) throw new Error('Not authenticated');
      return await apiClient.getStatus(token);
    },
    retry: false, // Don't retry on 404 - that's expected for new deployments
  }));

  // Deploy mutation
  const deployMutation = createMutation(() => ({
    mutationFn: async () => {
      const token = localStorage.getItem('autopeer_token');
      if (!token) throw new Error('Not authenticated');
      return await apiClient.deployPeering(token, {
        wg_public_key: wgPublicKey.trim(),
        endpoint: endpoint.trim(),
      });
    },
    onSuccess: (response) => {
      isEditing = false;
      onDeploymentSuccess?.(response.deployment);
      statusQuery.refetch();
    },
  }));

  // Activate mutation
  const activateMutation = createMutation(() => ({
    mutationFn: async () => {
      const token = localStorage.getItem('autopeer_token');
      if (!token) throw new Error('Not authenticated');
      return await apiClient.activatePeering(token);
    },
    onSuccess: () => {
      statusQuery.refetch();
    },
  }));

  // Deactivate mutation
  const deactivateMutation = createMutation(() => ({
    mutationFn: async () => {
      const token = localStorage.getItem('autopeer_token');
      if (!token) throw new Error('Not authenticated');
      return await apiClient.deactivatePeering(token);
    },
    onSuccess: () => {
      statusQuery.refetch();
    },
  }));

  // Set form values when editing existing deployment
  $effect(() => {
    if (statusQuery.data && isEditing) {
      // When editing, we can't retrieve the peer's original values
      // User needs to re-enter them
      wgPublicKey = '';
      endpoint = '';
    }
  });

  function handleSubmit(e: Event) {
    e.preventDefault();
    deployMutation.mutate();
  }

  function handleEdit() {
    isEditing = true;
    wgPublicKey = '';
    endpoint = '';
  }

  function handleCancelEdit() {
    isEditing = false;
    wgPublicKey = '';
    endpoint = '';
  }

  function handleLogout() {
    localStorage.removeItem('autopeer_token');
    onLogout?.();
  }

  // Validate WireGuard public key format (base64, 44 characters)
  const isValidWgKey = $derived(
    /^[A-Za-z0-9+/]{42}[A-Za-z0-9+/=]{2}$/.test(wgPublicKey.trim())
  );

  // Validate endpoint format (IPv4:port, [IPv6]:port, or domain:port)
  const isValidEndpoint = $derived(
    /^(\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}:\d+|\[[0-9a-fA-F:]+\]:\d+|[a-zA-Z0-9.-]+:\d+)$/.test(
      endpoint.trim()
    )
  );

  const canSubmit = $derived(
    wgPublicKey.trim() && endpoint.trim() && isValidWgKey && isValidEndpoint && !deployMutation.isPending
  );
</script>

<div class="bg-white rounded-lg shadow-md p-6">
  {#if statusQuery.isLoading}
    <div class="flex items-center justify-center py-12">
      <Loader2 class="w-8 h-8 text-blue-600 animate-spin" />
      <span class="ml-3 text-gray-600">Checking deployment status...</span>
    </div>
  {:else if statusQuery.data && !isEditing}
    <!-- Existing Deployment Display -->
    <div class="space-y-4">
      <div class="flex items-center justify-between">
        <h2 class="text-2xl font-bold text-gray-900 flex items-center gap-2">
          {#if statusQuery.data.is_active}
            <Check class="w-6 h-6 text-green-600" />
            Peering Active
          {:else}
            <Check class="w-6 h-6 text-yellow-600" />
            Peering Configured (Inactive)
          {/if}
        </h2>
        <div class="flex gap-3">
          <button
            onclick={handleEdit}
            class="px-4 py-2 bg-blue-600 text-white font-medium rounded-md hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 transition-colors"
          >
            Update Configuration
          </button>
          {#if onLogout}
            <LogoutButton onLogout={handleLogout} />
          {/if}
        </div>
      </div>

      <div class="bg-gray-50 rounded-lg p-4 space-y-3">
        <div class="grid grid-cols-2 gap-4">
          <div>
            <p class="text-sm font-medium text-gray-700">Interface Address</p>
            <p class="text-sm font-mono text-gray-900">{statusQuery.data.interface_address}</p>
          </div>
          <div>
            <p class="text-sm font-medium text-gray-700">Listen Port</p>
            <p class="text-sm font-mono text-gray-900">{statusQuery.data.listen_port}</p>
          </div>
          <div>
            <p class="text-sm font-medium text-gray-700">Our Public Key</p>
            <p class="text-xs font-mono text-gray-900 break-all">{statusQuery.data.our_public_key}</p>
          </div>
          <div>
            <p class="text-sm font-medium text-gray-700">Our Endpoint</p>
            <p class="text-sm font-mono text-gray-900">{statusQuery.data.our_endpoint}</p>
          </div>
        </div>

        <div class="border-t border-gray-200 pt-3 mt-3">
          <p class="text-sm font-semibold text-gray-700 mb-2">BGP Configuration</p>
          <div class="grid grid-cols-2 gap-4">
            <div>
              <p class="text-sm font-medium text-gray-600">BGP Neighbor</p>
              <p class="text-sm font-mono text-gray-900">{statusQuery.data.bgp_neighbor}</p>
            </div>
            <div>
              <p class="text-sm font-medium text-gray-600">Local AS</p>
              <p class="text-sm font-mono text-gray-900">{statusQuery.data.bgp_local_as}</p>
            </div>
            <div>
              <p class="text-sm font-medium text-gray-600">Remote AS</p>
              <p class="text-sm font-mono text-gray-900">{statusQuery.data.bgp_remote_as}</p>
            </div>
          </div>
        </div>
      </div>

      <div class="bg-blue-50 border-l-4 border-blue-400 p-4">
        <p class="text-sm text-blue-700">
          <strong>Note:</strong> Use the information above to configure your WireGuard client and BGP daemon.
        </p>
      </div>

      <!-- Activate/Deactivate Controls -->
      <div class="border-t border-gray-200 pt-4 space-y-3">
        <h3 class="text-lg font-semibold text-gray-900">Peering Control</h3>

        {#if activateMutation.isError}
          <div class="bg-red-50 border-l-4 border-red-400 p-4">
            <p class="text-sm text-red-700">
              <strong>Activation Error:</strong> {activateMutation.error instanceof Error
                ? activateMutation.error.message
                : 'Failed to activate peering'}
            </p>
          </div>
        {/if}

        {#if deactivateMutation.isError}
          <div class="bg-red-50 border-l-4 border-red-400 p-4">
            <p class="text-sm text-red-700">
              <strong>Deactivation Error:</strong> {deactivateMutation.error instanceof Error
                ? deactivateMutation.error.message
                : 'Failed to deactivate peering'}
            </p>
          </div>
        {/if}

        {#if activateMutation.isSuccess}
          <div class="bg-green-50 border-l-4 border-green-400 p-4">
            <p class="text-sm text-green-700">
              <strong>Success:</strong> Peering has been activated and is now running on the router.
            </p>
          </div>
        {/if}

        {#if deactivateMutation.isSuccess}
          <div class="bg-yellow-50 border-l-4 border-yellow-400 p-4">
            <p class="text-sm text-yellow-700">
              <strong>Success:</strong> Peering has been deactivated. Configuration is preserved and can be reactivated anytime.
            </p>
          </div>
        {/if}

        <div class="flex gap-3">
          {#if statusQuery.data.is_active}
            <button
              onclick={() => deactivateMutation.mutate()}
              disabled={deactivateMutation.isPending}
              class="flex-1 px-4 py-2 bg-orange-600 text-white font-medium rounded-md hover:bg-orange-700 focus:outline-none focus:ring-2 focus:ring-orange-500 focus:ring-offset-2 disabled:opacity-50 disabled:cursor-not-allowed transition-colors flex items-center justify-center gap-2"
            >
              {#if deactivateMutation.isPending}
                <Loader2 class="w-4 h-4 animate-spin" />
                <span>Deactivating...</span>
              {:else}
                <span>Deactivate Peering</span>
              {/if}
            </button>
          {:else}
            <button
              onclick={() => activateMutation.mutate()}
              disabled={activateMutation.isPending}
              class="flex-1 px-4 py-2 bg-green-600 text-white font-medium rounded-md hover:bg-green-700 focus:outline-none focus:ring-2 focus:ring-green-500 focus:ring-offset-2 disabled:opacity-50 disabled:cursor-not-allowed transition-colors flex items-center justify-center gap-2"
            >
              {#if activateMutation.isPending}
                <Loader2 class="w-4 h-4 animate-spin" />
                <span>Activating...</span>
              {:else}
                <span>Activate Peering</span>
              {/if}
            </button>
          {/if}
        </div>

        <p class="text-xs text-gray-500">
          {#if statusQuery.data.is_active}
            <strong>Deactivate:</strong> Remove configuration from router but keep it saved for later reactivation.
          {:else}
            <strong>Activate:</strong> Deploy WireGuard and BGP configuration to the router to establish the peering connection.
          {/if}
        </p>
      </div>
    </div>
  {:else}
    <!-- Deployment Form -->
    <div class="space-y-4">
      <h2 class="text-2xl font-bold text-gray-900">
        {isEditing ? 'Update' : 'Create'} Deployment
      </h2>

      <p class="text-gray-600">
        {#if isEditing}
          Update your WireGuard configuration details below. This will redeploy your peering connection.
        {:else}
          Provide your WireGuard public key and endpoint to create the peering connection.
        {/if}
      </p>

      <form onsubmit={handleSubmit} class="space-y-4">
        <!-- WireGuard Public Key -->
        <div>
          <label for="wg-pubkey" class="block text-sm font-medium text-gray-700 mb-1">
            <div class="flex items-center gap-2">
              <WifiIcon class="w-4 h-4" />
              Your WireGuard Public Key
            </div>
          </label>
          <input
            id="wg-pubkey"
            type="text"
            bind:value={wgPublicKey}
            placeholder="e.g., AbCdEfGhIjKlMnOpQrStUvWxYz0123456789+/="
            class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent font-mono text-sm"
            class:border-red-500={wgPublicKey && !isValidWgKey}
            class:border-green-500={wgPublicKey && isValidWgKey}
          />
          {#if wgPublicKey && !isValidWgKey}
            <p class="text-xs text-red-600 mt-1">
              Invalid WireGuard public key format (must be 44-character base64)
            </p>
          {/if}
        </div>

        <!-- Endpoint -->
        <div>
          <label for="endpoint" class="block text-sm font-medium text-gray-700 mb-1">
            <div class="flex items-center gap-2">
              <Globe class="w-4 h-4" />
              Your Endpoint (IP:port or domain:port)
            </div>
          </label>
          <input
            id="endpoint"
            type="text"
            bind:value={endpoint}
            placeholder="e.g., 1.2.3.4:51820 or [2001:db8::1]:51820 or example.com:51820"
            class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent font-mono text-sm"
            class:border-red-500={endpoint && !isValidEndpoint}
            class:border-green-500={endpoint && isValidEndpoint}
          />
          {#if endpoint && !isValidEndpoint}
            <p class="text-xs text-red-600 mt-1">
              Invalid endpoint format (use IPv4:port, [IPv6]:port, or domain:port)
            </p>
          {/if}
        </div>

        {#if deployMutation.isError}
          <div class="bg-red-50 border-l-4 border-red-400 p-4">
            <p class="text-sm text-red-700">
              <strong>Error:</strong> {deployMutation.error instanceof Error
                ? deployMutation.error.message
                : 'Failed to deploy peering'}
            </p>
          </div>
        {/if}

        <div class="flex gap-3">
          {#if isEditing}
            <button
              type="button"
              onclick={handleCancelEdit}
              class="flex-1 px-4 py-2 border border-gray-300 text-gray-700 font-medium rounded-md hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-gray-500 focus:ring-offset-2 transition-colors"
            >
              Cancel
            </button>
          {/if}
          <button
            type="submit"
            disabled={!canSubmit}
            class="flex-1 px-4 py-2 bg-blue-600 text-white font-medium rounded-md hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 disabled:opacity-50 disabled:cursor-not-allowed transition-colors flex items-center justify-center gap-2"
          >
            {#if deployMutation.isPending}
              <Loader2 class="w-4 h-4 animate-spin" />
              <span>Deploying...</span>
            {:else}
              <span>{isEditing ? 'Update' : 'Deploy'} Peering</span>
            {/if}
          </button>
        </div>
      </form>

      <div class="bg-yellow-50 border-l-4 border-yellow-400 p-4">
        <p class="text-sm text-yellow-700">
          <strong>Tip:</strong> Generate your WireGuard keypair with: <code
            class="bg-yellow-100 px-1 py-0.5 rounded">wg genkey | tee privatekey | wg pubkey</code
          >
        </p>
      </div>
    </div>
  {/if}
</div>
