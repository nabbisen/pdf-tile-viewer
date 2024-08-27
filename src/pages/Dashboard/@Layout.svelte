<script lang="ts">
  import { onMount } from 'svelte'
  import { invoke } from '@tauri-apps/api/core'
  import Header from './Header.svelte'
  import LoadedHistory from './LoadedHistory.svelte'
  import FileSelect from './FileSelect.svelte'

  onMount(() => {
    invoke('reset_window_title')
  })
</script>

<Header />
<FileSelect />
<LoadedHistory />
