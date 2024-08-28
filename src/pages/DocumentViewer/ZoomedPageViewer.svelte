<script lang="ts">
  import { onDestroy, onMount } from 'svelte'
  import type { PDFDocumentProxy } from 'pdfjs-dist'
  import PageViewer from './PageViewer.svelte'
  import { setZoomedPageIndex, subscribeZoomedPageIndex } from '../../stores/pages/documentViewer'
  import {
    setZoomViewBackgroundLocked,
    loadZoomViewBackgroundLocked,
    setZoomViewScale,
    loadZoomViewScale,
  } from '../../stores/settings/documentViewer/zoomedPageViewer'
  import { DEFAULT_ZOOM_VIEW_BACKGROUND_LOCKED, DEFAULT_ZOOM_VIEW_SCALE } from './consts'

  export let pdfDocument: PDFDocumentProxy

  let zoomViewBackgroundLocked: boolean
  let zoomViewScale: number

  let pageIndex: number

  let zoomViewTransparency: number = 0.0

  $: {
    subscribeZoomedPageIndex((value) => (pageIndex = value!))
  }

  $: {
    if (zoomViewBackgroundLocked) {
      lockBackground()
    } else {
      unlockBackground()
    }
  }

  onMount(() => {
    loadSettings()

    window.addEventListener('keydown', windowOnKeydown)
  })

  onDestroy(() => {
    unlockBackground()

    window.removeEventListener('keydown', windowOnKeydown)
  })

  function windowOnKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape' || e.key === 'Esc') {
      setZoomedPageIndex(undefined)
    }
  }

  function loadSettings() {
    loadZoomViewBackgroundLocked(DEFAULT_ZOOM_VIEW_BACKGROUND_LOCKED).then(
      (ret) => (zoomViewBackgroundLocked = ret as boolean)
    )
    loadZoomViewScale(DEFAULT_ZOOM_VIEW_SCALE).then((ret) => (zoomViewScale = ret as number))
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

  function lockBackgroundOnClick() {
    zoomViewBackgroundLocked = !zoomViewBackgroundLocked
    setZoomViewBackgroundLocked(zoomViewBackgroundLocked)
  }

  function zoomViewScaleOnChange() {
    setZoomViewScale(zoomViewScale)
  }

  function close() {
    setZoomedPageIndex(undefined)
  }

  function lockBackground() {
    window.document.body.style.overflow = 'hidden'
  }

  function unlockBackground() {
    window.document.body.style.overflow = 'auto'
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
    <div class="lock-background">
      Background:
      <label class="button auxiliary">
        {zoomViewBackgroundLocked ? 'Locked' : 'Free'}
        <input type="checkbox" bind:checked={zoomViewBackgroundLocked} />
      </label>
    </div>
    <label
      >Scale
      <input
        type="range"
        step="0.1"
        min="0.1"
        max="10.0"
        bind:value={zoomViewScale}
        on:change={zoomViewScaleOnChange}
      />
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

  nav label input[type='checkbox'] {
    display: none;
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

  nav .lock-background {
    width: 14.4em;
    text-align: left;
    cursor: pointer;
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
