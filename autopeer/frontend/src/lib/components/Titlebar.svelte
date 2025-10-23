<script lang="ts">
  import { LogOut } from 'lucide-svelte';

  interface Props {
    asn?: string;
    isLoggedIn?: boolean;
    onLogout?: () => void;
  }

  let { asn, isLoggedIn = false, onLogout }: Props = $props();
</script>

<div class="bg-white border-b border-gray-200 shadow-sm">
  <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
    <div class="flex justify-between items-center h-16">
      <div>
        <h1 class="text-2xl font-bold text-gray-900">DN42 AutoPeer</h1>
        <p class="text-sm text-gray-600">Automatic peering setup for DN42 network</p>
      </div>

      {#if isLoggedIn && asn}
        <div class="flex items-center gap-4">
          <div class="text-right">
            <p class="text-sm font-medium text-gray-700">ASN</p>
            <p class="text-lg font-mono font-semibold text-gray-900">{asn}</p>
          </div>
          <button
            onclick={onLogout}
            class="flex items-center gap-2 px-4 py-2 bg-red-600 text-white font-medium rounded-md hover:bg-red-700 focus:outline-none focus:ring-2 focus:ring-red-500 focus:ring-offset-2 transition-colors"
            title="Logout"
          >
            <LogOut class="w-4 h-4" />
            <span>Logout</span>
          </button>
        </div>
      {/if}
    </div>
  </div>
</div>
