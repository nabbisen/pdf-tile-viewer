<script lang="ts">
  import { invoke } from '@tauri-apps/api/core'
  import { getCurrentWebview, type DragDropEvent } from '@tauri-apps/api/webview'
  import { type UnlistenFn, type Event as TauriEvent } from '@tauri-apps/api/event'
  import { open } from '@tauri-apps/plugin-dialog'
  import { onMount, onDestroy } from 'svelte'

  import DocumentViewer from './DocumentViewer.svelte'
  import { errorToast } from '../stores/toast'

  let buffer: ArrayBuffer | undefined
  let filename: string = ''
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
    const filepath = paths[0]
    loadPdfBuffer(filepath)
  }

  const loadPdfBuffer = async (filepath: string) => {
    unloadPdfBuffer()

    try {
      const res: Array<any> = await invoke('read_pdf', { filepath: filepath })
      buffer = new Uint8Array(res).buffer
    } catch (error: unknown) {
      handleLoadPdfBufferError(error)
      return
    }

    updateFilename(filepath)
    pushToLoadedHistory(filepath)
  }

  const handleLoadPdfBufferError = (error: unknown) => {
    const messages: string =
      typeof error === 'string'
        ? (error as string)
        : error instanceof Error
          ? error.message
          : 'unknown error'
    errorToast(messages, 10000)
  }

  const unloadPdfBuffer = () => {
    buffer = undefined
    filename = ''
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
      const filepath = fileResponse.path
      loadPdfBuffer(filepath)
    }
  }

  const updateFilename = (filepath: string) => {
    const slashSplit = filepath.split('/')
    if (2 <= slashSplit.length) {
      filename = slashSplit[slashSplit.length - 1]
    } else {
      const backslashSplit = filepath.split('\\')
      filename = backslashSplit[backslashSplit.length - 1]
    }
  }

  const pushToLoadedHistory = (filepath: string) => {
    const existingItemIndex = loadedHistory.findIndex((x) => x.filepath === filepath)
    if (existingItemIndex !== -1) {
      loadedHistory.splice(existingItemIndex, 1)
    }
    const loadedHistoryItem = <LoadedHistoryItem>{
      filename,
      filepath,
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

<header class={buffer ? 'has-document' : ''}>
  <div class="logo">
    <button on:click={unloadPdfBuffer}>
      <h1>PDF Tile Viewer</h1>
    </button>
  </div>
  <h2>{filename}</h2>
</header>
<div>
  {#if buffer}
    <DocumentViewer {buffer} />
  {:else}
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
          <button on:click={() => loadPdfBuffer(x.filepath)}>
            <h4>{x.filename}</h4>
            <div class="filepath">{x.filepath}</div>
          </button>
        </li>
      {/each}
    </ul>
  {/if}
</div>

<style>
  header {
    position: relative;
    z-index: 10000;
  }

  .logo button {
    width: fit-content;
    height: fit-content;
    background: none;
    border: none;
    z-index: 10001;
  }
  .logo button:hover {
    background: none;
    border: none;
  }
  header.has-document .logo button {
    position: fixed;
    right: 0.8rem;
    bottom: 0.5rem;
    width: 4em;
    text-align: center;
    background-color: #efefef;
    font-size: 1rem;
  }
  header.has-document .logo button:hover {
    opacity: 0.93;
  }

  h1 {
    font-size: 1.8rem;
    padding: 0;
    margin: 0;
    color: #555555;
  }
  header.has-document,
  header.has-document h1 {
    font-size: 0.8rem;
  }
  header.has-document h1 {
    color: #878787;
  }
  h1::after {
    content: '🦈';
    display: inline-block;
    padding: 0 0.7rem;
  }

  header h2 {
    position: fixed;
    top: 0;
    right: 0.4rem;
    transform: rotate(90deg) translate(100%, 0);
    transform-origin: top right;
    white-space: nowrap;
    font-size: 0.8rem;
    color: #878787;
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
