<script lang="ts">
  import { QueryClient, QueryClientProvider } from '@tanstack/svelte-query';
  import InitPeering from './lib/components/InitPeering.svelte';

  const queryClient = new QueryClient({
    defaultOptions: {
      queries: {
        staleTime: 1000 * 60 * 5, // 5 minutes
        retry: 1,
      },
    },
  });
</script>

<QueryClientProvider client={queryClient}>
  <div class="min-h-screen bg-gray-100">
    <InitPeering />
  </div>
</QueryClientProvider>
