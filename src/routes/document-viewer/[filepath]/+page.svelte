<script lang="ts">
  import { page } from '$app/state'

  import DocumentViewer from '../../../pages/DocumentViewer/@Layout.svelte'

  const filepath: string = decodeURIComponent(page.params.filepath)
</script>

<DocumentViewer {filepath} />
