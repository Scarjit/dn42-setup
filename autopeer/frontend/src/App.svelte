<script lang="ts">
  import { QueryClient, QueryClientProvider } from '@tanstack/svelte-query';
  import InitPeering from './lib/components/InitPeering.svelte';
  import Titlebar from './lib/components/Titlebar.svelte';

  const queryClient = new QueryClient({
    defaultOptions: {
      queries: {
        staleTime: 1000 * 60 * 5, // 5 minutes
        retry: 1,
      },
    },
  });

  let asn = $state<string | null>(null);
  let isLoggedIn = $state(false);

  // Check login status from localStorage
  $effect(() => {
    if (typeof window !== 'undefined') {
      const token = localStorage.getItem('autopeer_token');
      const storedAsn = localStorage.getItem('autopeer_asn');
      isLoggedIn = !!token;
      asn = storedAsn;
    }
  });

  function handleLogout() {
    if (typeof window !== 'undefined') {
      localStorage.removeItem('autopeer_token');
      localStorage.removeItem('autopeer_asn');
      localStorage.removeItem('autopeer_challenge');
      localStorage.removeItem('autopeer_pgp_fingerprint');
      isLoggedIn = false;
      asn = null;
      // Reload page to reset state
      window.location.reload();
    }
  }
</script>

<QueryClientProvider client={queryClient}>
  <div class="min-h-screen bg-gray-100">
    <Titlebar asn={asn || undefined} isLoggedIn={isLoggedIn} onLogout={handleLogout} />
    <InitPeering />
  </div>
</QueryClientProvider>
