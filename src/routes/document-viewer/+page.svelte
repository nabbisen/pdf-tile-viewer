<script lang="ts">
  import { onMount } from 'svelte'
  import DocumentViewer from '../../pages/DocumentViewer/@Layout.svelte'
  import { getFilepath } from '../../stores/pages/documentViewer'
  import { errorToast } from '../../stores/components/toast'
  import { returnHome } from '../../utils/route'

  let filepath: string | undefined = $state()
  onMount(() => {
    filepath = getFilepath()
    if (!filepath) {
      errorToast('File path is missing', 1400)
      setTimeout(() => returnHome, 3000)
    }
  })
</script>

{#if filepath}
  <DocumentViewer {filepath} />
{/if}
