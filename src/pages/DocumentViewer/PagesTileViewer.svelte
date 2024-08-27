<script lang="ts">
  import { onDestroy, onMount } from 'svelte'
  import { invoke } from '@tauri-apps/api/core'
  import { type PageViewport, type PDFDocumentProxy } from 'pdfjs-dist'
  import PageViewer from './PageViewer.svelte'
  import ZoomedPageViewer from './ZoomedPageViewer.svelte'
  import { pushToLoadedHistory } from '../../stores/pages/loadedHistory'
  import { getDocumentProxy } from '../../utils/pdf'
  import { debounce } from '../../utils/event'
  import { handleInvokeError } from '../../utils/backend'
  import { returnHome } from '../../utils/route'
  import {
    subscribeBuffer,
    subscribeMatchedPageIndexes,
    subscribeDisplayMatchedPages,
    subscribeZenMode,
    reset,
  } from '../../stores/pages/documentViewer'
  import PagesTileViewerAside from './PagesTileViewerAside.svelte'
  import Tooltip from '../../components/Tooltip.svelte'

  const DEFAULT_SCALE: number = 1.0
  const SCALE_UNIT: number = 0.2
  const MIN_SCALE: number = SCALE_UNIT
  const MAX_SCALE: number = 5.0

  const debounceUpdatePageIndexesRows = debounce(updatePageIndexesRows, 200)
  const debounceHandleWheel = debounce(handleWheel, 120)

  export let filepath: string | undefined

  let initialized: boolean = false

  let buffer: ArrayBuffer | undefined
  let matchedPageIndexes: number[] = []
  let displayMatchedPages: string | undefined
  let zenMode: boolean = false

  let scale: number = DEFAULT_SCALE
  let pdfDocument: PDFDocumentProxy
  let pageViewport: PageViewport
  let pageIndexesRows: number[][] = []
  let pageViewerContainers: HTMLDivElement[] = []
  let zoomedPageIndex: number | undefined
  let pageNumVisible: boolean = false
  let fixPagesPerRow: boolean = false
  let pagesPerRow: number

  $: {
    subscribeBuffer((value) => (buffer = value))
  }

  $: {
    subscribeMatchedPageIndexes((value) => (matchedPageIndexes = value))
  }

  $: {
    subscribeDisplayMatchedPages((value) => (displayMatchedPages = value))
  }

  $: {
    subscribeZenMode((value) => (zenMode = value))
  }

  $: {
    if (buffer) load()
  }

  onMount(() => {
    window.addEventListener('resize', debounceUpdatePageIndexesRows)
    window.addEventListener('wheel', debounceHandleWheel)
  })

  onDestroy(() => {
    window.removeEventListener('resize', debounceUpdatePageIndexesRows)
    window.removeEventListener('wheel', debounceHandleWheel)

    if (pdfDocument) pdfDocument.destroy()
    reset()
  })

  function load() {
    getDocumentProxy(buffer!)
      .then(loadCallback)
      .catch((error: unknown) => {
        handleInvokeError(error)
        returnHome()
      })
  }

  function loadCallback(ret: PDFDocumentProxy) {
    pdfDocument = ret

    updatePageIndexesRows()

    if (!initialized) {
      initialize()
    }
  }

  function initialize() {
    pushToLoadedHistory(filepath!)

    invoke('set_window_title', { filepath: filepath })

    initialized = true
  }

  function handleWheel(event: WheelEvent) {
    if (event.ctrlKey) {
      0 < event.deltaY ? decreaseScale() : increaseScale()
    }
  }

  function handlePageViewport(event: CustomEvent<PageViewport>) {
    const viewport = event.detail
    pageViewport = viewport

    updatePageIndexesRows()
  }

  function updatePageIndexesRows() {
    let ret: number[][] = []

    const rowBreak = pageIndexesPerRow()
    let row: number[] = []
    for (let pageIndex = 0; pageIndex < pdfDocument.numPages; pageIndex++) {
      if (0 < pageIndex && pageIndex % rowBreak === 0) {
        ret.push(row)
        row = []
      }
      row.push(pageIndex)
    }
    if (0 < row.length) {
      ret.push(row)
    }

    pageIndexesRows = ret
  }

  function pageIndexesPerRow(): number {
    if (!pdfDocument) return 0

    if (fixPagesPerRow) return pagesPerRow

    if (!pageViewport) return pdfDocument.numPages
    return Math.floor(window.innerWidth / pageViewport.width)
  }

  function fixPagesPerRowChange(e: CustomEvent<{ fixPagesPerRow: boolean; pagesPerRow: number }>) {
    const { fixPagesPerRow: _fixPagesPerRow, pagesPerRow: _pagesPerRow } = e.detail

    fixPagesPerRow = _fixPagesPerRow
    pagesPerRow = _pagesPerRow

    updatePageIndexesRows()
  }

  function increaseScale() {
    scale = scale + SCALE_UNIT
    if (MAX_SCALE < scale) {
      scale = MAX_SCALE
    }
  }
  function decreaseScale() {
    scale = scale - SCALE_UNIT
    if (scale < MIN_SCALE) {
      scale = MIN_SCALE
    }
  }

  function showZoomedPage(pageIndex: number) {
    // initialize
    zoomedPageIndex = undefined

    zoomedPageIndex = pageIndex
  }

  function scaleChangeHandler(e: CustomEvent<number>) {
    if (!e.detail) return
    scale = Number(e.detail)
  }

  function togglePageNum() {
    pageNumVisible = !pageNumVisible
  }

  let scrollToPageIndex: number | undefined
  let scrollEffectTimer: number | undefined

  function scrollToPage(e: CustomEvent<number>) {
    const scrollToPageNum = e.detail
    if (!scrollToPageNum || !Number.isInteger(scrollToPageNum)) return
    const pageIndex = Number(scrollToPageNum) - 1
    const found = document.querySelector(`.tile[data-page-index="${pageIndex}"]`)
    if (!found) return

    found.scrollIntoView({ behavior: 'smooth' })
    scrollToPageIndex = pageIndex

    if (scrollEffectTimer !== undefined) clearTimeout(scrollEffectTimer)
    scrollEffectTimer = setTimeout(() => {
      scrollToPageIndex = undefined
      scrollEffectTimer = undefined
    }, 5700)
  }
</script>

<div class="document">
  {#if pdfDocument && pdfDocument.numPages}
    {#each pageIndexesRows as pageIndexes}
      <div class="row">
        {#each pageIndexes as pageIndex}
          <div class="col" bind:this={pageViewerContainers[pageIndex]}>
            <div
              class="tile"
              data-page-index={pageIndex}
              class:scrolled-into-view={scrollToPageIndex === pageIndex}
              class:search-matched={!zenMode && matchedPageIndexes.includes(pageIndex)}
            >
              <div class="page">
                <article>
                  <PageViewer
                    {pdfDocument}
                    {pageIndex}
                    {scale}
                    viewerClass="tile"
                    on:pageViewport={handlePageViewport}
                  />
                </article>
                <nav>
                  <!-- left part (empty currently) -->
                  <div></div>
                  <div class="pageNum" class:visible={pageNumVisible}>{pageIndex + 1}</div>
                  <Tooltip messages="Zoom up" position="bottom">
                    <button
                      class="zoom"
                      on:click={() => showZoomedPage(pageIndex)}
                      aria-label="zoom"
                    ></button></Tooltip
                  >
                </nav>
              </div>
            </div>
          </div>
        {/each}
      </div>
    {/each}
  {/if}
</div>

<ZoomedPageViewer pageIndex={zoomedPageIndex} {pdfDocument} />

{#if pdfDocument && !zenMode}
  <PagesTileViewerAside
    {MIN_SCALE}
    {MAX_SCALE}
    {SCALE_UNIT}
    {scale}
    numPages={pdfDocument.numPages}
    {pageNumVisible}
    on:scaleChange={scaleChangeHandler}
    on:togglePageNum={togglePageNum}
    on:scrollToPage={scrollToPage}
    on:fixPagesPerRowChange={fixPagesPerRowChange}
  />
{/if}

<style>
  .document {
    padding-bottom: 2.7rem;
  }

  .row {
    display: flex;
  }
  .row:not(:first-child) {
    border-bottom: 0.02rem solid #bbbbbb;
  }
  .row:not(:last-child) {
    border-top: 0.02rem solid #bbbbbb;
  }
  .row .col:not(:first-child) article {
    border-left: 0.02rem solid #bbbbbb;
  }
  .row .col:not(:last-child) article {
    border-left: 0.02rem solid #bbbbbb;
  }
  .col:last-child {
    padding-right: 4.4rem;
  }

  .tile.scrolled-into-view,
  .tile.search-matched {
    position: relative;
  }
  .tile.scrolled-into-view::before,
  .tile.search-matched::before {
    content: '';
    position: absolute;
    left: 5%;
    bottom: 0;
    width: 90%;
    height: 0.2rem;
  }
  .tile.scrolled-into-view::before {
    background-color: #5d9ae5;
  }
  .tile.search-matched::before {
    background-color: #b7a42a;
  }
  .tile.scrolled-into-view.search-matched::before {
    background: linear-gradient(
      to right,
      #5d9ae5 0%,
      #5d9ae5 27%,
      #b7a42a 27%,
      #b7a42a 73%,
      #5d9ae5 73%,
      #5d9ae5 100%
    );
  }
  .tile.search-matched::after {
    content: 'Search matched';
    position: absolute;
    left: 0;
    bottom: 0.18rem;
    width: 100%;
    color: #b7a42a;
    text-align: center;
    font-size: 0.7rem;
    z-index: 1;
  }

  .tile .page {
    position: relative;
  }
  .tile .page article:hover {
    transform: scale(1.003) translateX(-0.09%) translateY(-0.09%);
  }
  .tile .page nav {
    position: absolute;
    bottom: 0.7rem;
    left: 10%;
    width: 80%;
    height: 1.1rem;
    padding: 0.3rem 0.7rem;
    display: none;
    justify-content: space-between;
    align-items: flex-end;
  }
  .tile .page:hover nav,
  .tile .page nav:has(.pageNum.visible) {
    display: flex;
  }
  .tile .page nav > * {
    font-size: 0.8rem;
    color: #878787;
    text-align: center;
  }
  .tile .page nav button.zoom {
    padding: 0 0.4rem;
    font-size: 1rem;
  }
  .tile .page nav button.zoom::before {
    content: '🧐';
  }
  .tile .page:not(:hover) nav .pageNum.visible ~ * button.zoom::before {
    content: '';
  }
</style>
