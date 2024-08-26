<script lang="ts">
  import { onMount } from 'svelte'
  import type { PDFDocumentProxy } from 'pdfjs-dist'
  import PageViewer from './PageViewer.svelte'

  export let pageIndex: number | undefined
  export let pdfDocument: PDFDocumentProxy

  let zoomViewScale: number = 3.0
  let zoomViewTransparency: number = 0.0
  let backgroundScrollable: boolean = true

  $: {
    if (pageIndex && !backgroundScrollable) {
      window.document.body.style.overflow = 'hidden'
    } else {
      window.document.body.style.overflow = 'auto'
    }
  }

  onMount(ready)

  function ready() {
    window.addEventListener('keydown', (e: KeyboardEvent) => {
      if (e.key === 'Escape' || e.key === 'Esc') {
        pageIndex = undefined
      }
    })
  }

  function close() {
    pageIndex = undefined
  }
</script>

{#if pageIndex !== undefined}
  <div class="zoomView">
    <div class="viewer-wrapper" style={`opacity: ${1.0 - zoomViewTransparency};`}>
      <PageViewer {pdfDocument} {pageIndex} scale={zoomViewScale} viewerClass="zoom" />
    </div>
    <nav>
      <span class="pageIndex">p.{pageIndex + 1}</span>
      <div class="background-scrollable">
        Background is:
        <button class="auxiliary" on:click={() => (backgroundScrollable = !backgroundScrollable)}>
          {backgroundScrollable ? 'scrollable' : 'fixed'}
        </button>
        <input type="checkbox" bind:checked={backgroundScrollable} />
      </div>
      <label
        >Scale
        <input type="range" step="0.1" min="0.1" max="10.0" bind:value={zoomViewScale} />
      </label>
      <label
        >Transparency
        <input type="range" step="0.1" min="0.0" max="1.0" bind:value={zoomViewTransparency} />
      </label>
      <button class="close auxiliary" on:click={close}>Close</button>
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
    z-index: 20001;
  }
  .zoomView .viewer-wrapper {
    height: calc(100% - 3.2rem);
    display: flex;
    justify-content: center;
    align-items: center;
    background-color: #252525;
  }
  .zoomView nav {
    width: calc(100% - 0.8rem);
    height: 1.75rem;
    padding: 0.8rem 0.4rem 0.3rem;
    display: flex;
    justify-content: space-around;
    align-items: center;
    background-color: #2aabb7;
    color: #ffffff;
  }
  .zoomView nav * {
    font-size: 1rem;
  }
  .background-scrollable {
    width: 14.4em;
    text-align: left;
    cursor: pointer;
  }
  .background-scrollable button + input[type='checkbox'] {
    display: none;
  }
  .zoomView nav input[type='range'] {
    width: 5.7em;
    text-align: right;
  }
  .zoomView nav button.close {
    background-color: #ffffff;
    color: #252525;
  }
</style>
