<script lang="ts">
  import { open } from '@tauri-apps/plugin-dialog'
  import { subscribeLoadedHistory } from '../../stores/pages/loadedHistory'
  import type { LoadedHistoryItem } from '../../types/pages/loadedHistory'
  import { openDocumentViewer } from '../../utils/route'

  let loadedHistory: LoadedHistoryItem[] = []

  $: {
    subscribeLoadedHistory((value) => (loadedHistory = [...value]))
  }

  async function fileSelect() {
    const fileResponse = await open({
      multiple: false, // todo
      directory: false,
      filters: [
        { name: 'PDF', extensions: ['pdf'] },
        { name: 'All files', extensions: ['*'] },
      ],
    })
    if (fileResponse) {
      const filepath = fileResponse.path
      openDocumentViewer(filepath)
    }
  }
</script>

<div class={`placeholder ${0 < loadedHistory.length ? 'shrink' : ''}`}>
  <h2>Drop PDF file</h2>
  <button on:click={fileSelect}>Choose file</button>
</div>

<style>
  .placeholder {
    padding: 16vh 0;
    margin: 2rem;
    text-align: center;
    border: 1px dotted black;
  }
  .placeholder.shrink {
    padding-top: 1.1rem;
    padding-bottom: 1.1rem;
  }
  .placeholder button {
    padding: 0.2rem 0.6rem;
    background-color: #2aabb7;
    color: #ffffff;
    border: 0.03rem #4ccbd6 solid;
    font-size: 1.5rem;
    border-radius: 0.2rem;
  }
  .placeholder button:hover {
    opacity: 0.87;
  }
</style>
