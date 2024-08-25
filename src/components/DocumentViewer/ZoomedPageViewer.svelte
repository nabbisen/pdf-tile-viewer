<script lang="ts">
  import { onMount } from 'svelte'
  import type { PDFDocumentProxy } from 'pdfjs-dist'
  import PageViewer from './PageViewer.svelte'

  export let pageIndex: number | undefined
  export let pdfDocument: PDFDocumentProxy

  let zoomViewScale: number = 3.0
  let zoomViewOpacity: number = 1.0
  let windowScrollable: boolean = true

  $: {
    if (pageIndex && !windowScrollable) {
      window.document.body.style.overflow = 'hidden'
    } else {
      window.document.body.style.overflow = 'auto'
    }
  }

  onMount(() => {
    window.addEventListener('keydown', (e: KeyboardEvent) => {
      if (e.key === 'Escape' || e.key === 'Esc') {
        pageIndex = undefined
      }
    })
  })

  const close = () => {
    pageIndex = undefined
  }
</script>

{#if pageIndex !== undefined}
  <div class="zoomView">
    <div class="wrapper" style={`opacity: ${zoomViewOpacity};`}>
      <PageViewer {pdfDocument} {pageIndex} scale={zoomViewScale} />
    </div>
    <nav>
      <span class="pageIndex">p.{pageIndex + 1}</span>
      <label class="window-scrollable"
        >Window <u>{windowScrollable ? 'scrollable' : 'fixed'}</u>
        <input id="toggle-window-scrollable" type="checkbox" bind:checked={windowScrollable} />
      </label>
      <label
        >Scale
        <input type="number" step="0.1" min="0.1" max="10.0" bind:value={zoomViewScale} />
      </label>
      <label
        >Transparency
        <input type="number" step="0.1" min="0.0" max="1.0" bind:value={zoomViewOpacity} />
      </label>
      <button class="close" on:click={close}>Close</button>
    </nav>
  </div>
{/if}

<style>
  .zoomView {
    position: fixed;
    left: 10vw;
    top: 5vh;
    width: 80vw;
    height: 90vh;
    display: flex;
    flex-direction: column;
    border: 0.27rem #2aabb7 solid;
  }
  .zoomView .wrapper {
    width: 100%;
    display: flex;
    justify-content: center;
    align-items: center;
    background-color: #252525;
    overflow: scroll;
  }
  .zoomView nav {
    width: 100%;
    height: 1.75rem;
    padding: 0.3rem 0.4rem 0.1rem;
    display: flex;
    justify-content: space-around;
    align-items: center;
    background-color: #2aabb7;
    color: #ffffff;
  }
  .zoomView nav * {
    font-size: 1rem;
  }
  .window-scrollable {
    width: 9.7em;
    text-align: left;
    cursor: pointer;
  }
  .window-scrollable input {
    display: none;
  }
  .zoomView nav input[type='number'] {
    width: 3.2em;
    text-align: right;
  }
  .zoomView nav button.close {
    background-color: #ffffff;
    color: #252525;
  }
</style>
