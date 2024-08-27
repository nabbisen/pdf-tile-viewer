<script lang="ts">
  import { onDestroy, onMount } from 'svelte'
  import type { PDFDocumentProxy } from 'pdfjs-dist'
  import PageViewer from './PageViewer.svelte'
  import { subscribeZoomedPageIndex, setZoomedPageIndex } from '../../stores/pages/documentViewer'

  export let pdfDocument: PDFDocumentProxy

  let pageIndex: number

  let zoomViewScale: number = 2.7
  let zoomViewTransparency: number = 0.0
  let backgroundScrollable: boolean = true

  $: {
    subscribeZoomedPageIndex((value) => (pageIndex = value!))
  }

  $: {
    if (pageIndex && !backgroundScrollable) {
      window.document.body.style.overflow = 'hidden'
    } else {
      window.document.body.style.overflow = 'auto'
    }
  }

  onMount(() => {
    window.addEventListener('keydown', windowOnKeydown)
  })

  onDestroy(() => {
    window.removeEventListener('keydown', windowOnKeydown)
  })

  function windowOnKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape' || e.key === 'Esc') {
      setZoomedPageIndex(undefined)
    }
  }

  function prevPage() {
    const _pageIndex = pageIndex - 1
    if (_pageIndex < 0) return
    setZoomedPageIndex(_pageIndex)
  }

  function nextPage() {
    const _pageIndex = pageIndex + 1
    if (pdfDocument.numPages < _pageIndex) return
    setZoomedPageIndex(_pageIndex)
  }

  function close() {
    setZoomedPageIndex(undefined)
  }
</script>

<div class="zoom-view">
  <div class="viewer-wrapper" style={`opacity: ${1.0 - zoomViewTransparency};`}>
    <PageViewer {pdfDocument} {pageIndex} scale={zoomViewScale} viewerClass="zoom" />
  </div>
  <nav>
    <div class="page">
      <button class="prev auxiliary" on:click={prevPage} disabled={pageIndex === 0}>←</button>
      <span class="page-index">p.{pageIndex + 1}</span>
      <button
        class="next auxiliary"
        on:click={nextPage}
        disabled={pageIndex === pdfDocument.numPages - 1}>→</button
      >
    </div>
    <div class="background-scrollable">
      Background:
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

<style>
  .zoom-view {
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

  .viewer-wrapper {
    height: calc(100% - 3.2rem);
    display: flex;
    justify-content: center;
    align-items: center;
    background-color: #252525;
  }

  nav {
    width: calc(100% - 0.8rem);
    height: 1.75rem;
    padding: 0.7rem 0.4rem 0.4rem;
    display: flex;
    justify-content: space-around;
    align-items: center;
    background-color: #2aabb7;
    color: #ffffff;
  }

  nav * {
    font-size: 1rem;
  }

  nav .page {
    margin-right: 0.4rem;
  }
  nav .page-index {
    margin: 0 0.3rem;
  }
  nav .page button.prev,
  nav .page button.next {
    padding: 0.1rem 0.3rem;
    font-size: 0.8rem;
    border-radius: 50%;
  }

  nav .background-scrollable {
    width: 14.4em;
    text-align: left;
    cursor: pointer;
  }
  nav .background-scrollable button + input[type='checkbox'] {
    display: none;
  }
  nav .zoomView nav input[type='range'] {
    width: 5.7em;
    text-align: right;
  }
  nav .zoomView nav button.close {
    background-color: #ffffff;
    color: #252525;
  }
</style>
