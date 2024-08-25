<script lang="ts">
  import { getCurrentWebview, type DragDropEvent } from '@tauri-apps/api/webview'
  import { type UnlistenFn, type Event as TauriEvent } from '@tauri-apps/api/event'
  import { open } from '@tauri-apps/plugin-dialog'
  import { onMount, onDestroy } from 'svelte'

  import DocumentViewer from './DocumentViewer.svelte'
  import { filename } from '../utils/file'

  let filepath: string | undefined
  let unlistenDragDrop: UnlistenFn | undefined
  interface LoadedHistoryItem {
    filename: string
    filepath: string
    timestamp: Date
  }
  let loadedHistory: LoadedHistoryItem[] = []

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

  const chooseFile = async () => {
    const fileResponse = await open({
      multiple: false, // todo
      directory: false,
      filters: [
        { name: 'PDF', extensions: ['pdf'] },
        { name: 'All files', extensions: ['*'] },
      ],
    })
    if (fileResponse) {
      filepath = fileResponse.path
    }
  }

  const pushToLoadedHistory = (e: CustomEvent<string>) => {
    const successFilepath = e.detail

    const existingItemIndex = loadedHistory.findIndex((x) => x.filepath === successFilepath)
    if (existingItemIndex !== -1) {
      loadedHistory.splice(existingItemIndex, 1)
    }
    const loadedHistoryItem = <LoadedHistoryItem>{
      filename: filename(successFilepath),
      filepath: successFilepath,
      timestamp: new Date(),
    }
    loadedHistory.push(loadedHistoryItem)
  }

  async function ready() {
    await listenDragDrop()
  }
  onMount(ready)
  onDestroy(() => {
    unlistenDragDrop && unlistenDragDrop()
  })
</script>

{#if filepath}
  <DocumentViewer
    {filepath}
    on:loadSuccess={pushToLoadedHistory}
    on:closeDocument={() => {
      filepath = undefined
    }}
  />
{:else}
  <header>
    <div class="logo">
      <h1>PDF Tile Viewer</h1>
    </div>
  </header>

  <span class={0 < loadedHistory.length ? 'has-history' : ''}></span>

  <div class="placeholder">
    <h2>Drop PDF file</h2>
    <p>
      <b>Number of pages in a row</b> is dynamically calculated with <b>app window size</b>
      and <b>zoom scale</b> modified with Ctrl key + mouse wheel.
    </p>
    <button on:click={chooseFile}>Choose file</button>
  </div>

  <h3>History to re-open</h3>
  <ul class="loaded-history">
    {#each loadedHistory as x}
      <li>
        <time>{x.timestamp.toTimeString().split(' ')[0]}</time>
        <button
          on:click={() => {
            filepath = x.filepath
          }}
        >
          <h4>{x.filename}</h4>
          <div class="filepath">{x.filepath}</div>
        </button>
      </li>
    {/each}
  </ul>
{/if}

<style>
  header {
    position: relative;
    z-index: 10000;
  }

  h1 {
    font-size: 1.8rem;
    padding: 0;
    margin: 0;
    color: #555555;
  }
  h1::after {
    content: '🦈';
    display: inline-block;
    padding: 0 0.7rem;
  }

  .placeholder {
    padding: 16vh 0;
    margin: 2rem;
    text-align: center;
    border: 1px dotted black;
  }
  .has-history ~ .placeholder {
    padding-top: 1.1rem;
    padding-bottom: 1.1rem;
  }
  .placeholder p {
    max-width: 16em;
    margin: 1.1rem auto;
    text-align: left;
    font-size: 0.8rem;
    font-family: serif;
    color: #727272;
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

  h3 {
    display: none;
    color: #878787;
    text-align: center;
  }
  .has-history ~ h3 {
    display: block;
  }

  .loaded-history {
    max-width: 40rem;
    margin: 0.5rem auto;
  }
  .loaded-history {
    list-style-type: none;
  }
  .loaded-history time {
    margin-right: 0.8rem;
    color: #878787;
    font-size: 0.8rem;
  }
  .loaded-history button {
    position: relative;
    max-width: calc(100% - 9rem);
    background: none;
    border: none;
    font-size: 1.4rem;
    text-align: left;
    cursor: pointer;
  }
  .loaded-history h4 {
    padding: 0;
    margin: 0;
  }
  .loaded-history button:hover h4 {
    color: #2aabb7;
  }
  .loaded-history h4::before {
    width: 0;
  }
  .loaded-history button:hover h4::before {
    content: '';
    position: absolute;
    left: 0;
    bottom: 0;
    width: 100%;
    height: 0.04rem;
    background-color: #4ccbd6;
    transition: width 1s ease;
  }
  .loaded-history .filepath {
    max-width: 100%;
    font-size: 0.8rem;
    white-space: nowrap;
    overflow-x: hidden;
    text-overflow: ellipsis;
    unicode-bidi: plaintext;
    direction: rtl;
  }
</style>
