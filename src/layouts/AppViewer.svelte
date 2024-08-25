<script lang="ts">
  import { getCurrentWebview, type DragDropEvent } from '@tauri-apps/api/webview'
  import { type UnlistenFn, type Event as TauriEvent } from '@tauri-apps/api/event'
  import { onMount, onDestroy } from 'svelte'

  import Dashboard from '../components/Dashboard.svelte'
  import DocumentViewer from '../components/DocumentViewer/Layout.svelte'

  let filepath: string | undefined
  let unlistenDragDrop: UnlistenFn | undefined

  async function listenDragDrop() {
    unlistenDragDrop = await getCurrentWebview().onDragDropEvent(
      (event: TauriEvent<DragDropEvent>) => {
        if (event.payload.type === 'drop') {
          handleDrop(event.payload.paths)
        }
      }
    )
  }

  const handleDrop = (paths: string[]) => {
    // todo: single file only now
    filepath = paths[0]
  }

  async function ready() {
    await listenDragDrop()
  }
  onMount(ready)
  onDestroy(() => {
    unlistenDragDrop && unlistenDragDrop()
  })

  const onFileSelect = (e: CustomEvent<string>) => {
    filepath = e.detail
  }
</script>

{#if filepath}
  <DocumentViewer
    {filepath}
    on:closeDocument={() => {
      filepath = undefined
    }}
  />
{:else}
  <Dashboard on:fileSelect={onFileSelect} />
{/if}
