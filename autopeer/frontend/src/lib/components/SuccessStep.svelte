<script lang="ts">
  import { CheckCircle2 } from 'lucide-svelte';
  import type { DeploymentInfo } from '$lib/api/types';

  interface Props {
    asn: string;
    deployment: DeploymentInfo;
  }

  let { asn, deployment }: Props = $props();
</script>

<div class="bg-white rounded-lg shadow-md p-6">
  <div class="flex items-center gap-2 mb-4">
    <CheckCircle2 class="w-6 h-6 text-green-600" />
    <h2 class="text-xl font-semibold">Peering Verified Successfully!</h2>
  </div>

  <p class="text-gray-600 mb-6">
    Your peering has been verified and deployed. Below are the configuration details for your records.
  </p>

  <div class="bg-gray-50 rounded-md p-6 mb-4">
    <h3 class="text-lg font-medium text-gray-900 mb-4">Deployment Information</h3>

    <div class="space-y-3">
      <div class="grid grid-cols-2 gap-4">
        <div>
          <p class="text-sm font-medium text-gray-500">WireGuard Interface</p>
          <p class="text-sm text-gray-900 font-mono">{deployment.interface_address}</p>
        </div>
        <div>
          <p class="text-sm font-medium text-gray-500">Listen Port</p>
          <p class="text-sm text-gray-900 font-mono">{deployment.listen_port}</p>
        </div>
      </div>

      <div>
        <p class="text-sm font-medium text-gray-500">Our Public Key</p>
        <p class="text-sm text-gray-900 font-mono break-all">{deployment.our_public_key}</p>
      </div>

      <div>
        <p class="text-sm font-medium text-gray-500">Our Endpoint</p>
        <p class="text-sm text-gray-900 font-mono">{deployment.our_endpoint}</p>
      </div>

      <div class="border-t pt-3">
        <p class="text-sm font-medium text-gray-700 mb-2">BGP Configuration</p>
        <div class="grid grid-cols-3 gap-4">
          <div>
            <p class="text-xs font-medium text-gray-500">Neighbor</p>
            <p class="text-sm text-gray-900 font-mono">{deployment.bgp_neighbor}</p>
          </div>
          <div>
            <p class="text-xs font-medium text-gray-500">Local ASN</p>
            <p class="text-sm text-gray-900 font-mono">{deployment.bgp_local_as}</p>
          </div>
          <div>
            <p class="text-xs font-medium text-gray-500">Remote ASN</p>
            <p class="text-sm text-gray-900 font-mono">{deployment.bgp_remote_as}</p>
          </div>
        </div>
      </div>
    </div>
  </div>

  <div class="p-4 bg-blue-50 border border-blue-200 rounded-md">
    <p class="text-sm text-blue-900 font-medium mb-2">Your WireGuard Configuration:</p>
    <ol class="text-sm text-blue-800 space-y-1 list-decimal list-inside">
      <li>Create your WireGuard interface configuration</li>
      <li>Add our endpoint and public key as a [Peer] section</li>
      <li>Configure BGP with neighbor {deployment.bgp_neighbor}</li>
      <li>Your peering is now active!</li>
    </ol>
  </div>
</div>
