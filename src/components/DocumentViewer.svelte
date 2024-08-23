<script lang="ts">
  import { invoke } from '@tauri-apps/api/core'
  import { getCurrentWebview, type DragDropEvent } from '@tauri-apps/api/webview'
  import { type UnlistenFn, type Event as TauriEvent } from '@tauri-apps/api/event'
  import { onMount, onDestroy } from 'svelte'

  import PagesViewer from './PagesViewer.svelte';

  let buffer: ArrayBuffer | undefined
  let documentTitle: string = ''
  let unlistenDragDrop: UnlistenFn | undefined

  async function listenDragDrop() {
    unlistenDragDrop = await getCurrentWebview().onDragDropEvent((event: TauriEvent<DragDropEvent>) => {
      // console.log(event.payload)
      // todo
      if (event.payload.type === 'drop') {
        const paths = event.payload.paths
        const dropped = paths[0] // todo single file only now

        buffer = undefined
        loadPdfBuffer(dropped)

        const slashSplit = dropped.split('/')
        if (2 <= slashSplit.length) {
          documentTitle = slashSplit[slashSplit.length - 1]
        } else {
          const backslashSplit = dropped.split('\\')
          documentTitle = backslashSplit[backslashSplit.length - 1]
        }
      }
    })
  }

  const loadPdfBuffer = async (filepath: string) => {
    const res: Array<any> = await invoke("read_pdf", { filepath: filepath })
    buffer = new Uint8Array(res).buffer
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
  <button on:click={() => buffer = undefined}>
    <h1>PDF Tile Viewer</h1>
  </button>
  <h2>{documentTitle}</h2>
</header>
<div class="wrapper">
  {#if buffer }
    <PagesViewer buffer={buffer} />
  {:else}
    <div class="placeholder">
      <strong>Drop PDF file</strong>
      <p>
        <b>Number of pages in a row</b> is dynamically calculated
        with <b>app window size</b>
        and <b>zoom scale</b> modified with Ctrl key + mouse wheel.
      </p>
    </div>
  {/if}
</div>

<style>
  .wrapper {
    width: 100%;
    height: 100%;
    padding: 0;
    margin: 0;
  }

  header {
    position: relative;
    z-index: 10000
  }

  button {
    width: fit-content;
    height: fit-content;
    background: none;
    border: none;
    z-index: 10001;
  }
  button:hover {
    background: none;
    border: none;
  }
  header.has-document button {
    position: fixed;
    right: 0.8rem;
    bottom: 0.5rem;
    width: 4em;
    text-align: center;
    background-color: #efefef;
    font-size: 1.0rem;
  }
  header.has-document button:hover {
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

  h2 {
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
    padding: 16% 0;
    margin: 2rem;
    text-align: center;
    border: 1px dotted black;
  }
  .placeholder p {
    max-width: 16.0em;
    margin: 1.1rem auto;
    text-align: left;
    font-size: 0.8rem;
    font-family: serif;
    color: #727272;
  }
</style>
