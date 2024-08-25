<script lang="ts">
  import { onMount } from 'svelte'
  import { type PageViewport, type PDFDocumentProxy } from 'pdfjs-dist'
  import PageViewer from './PageViewer.svelte'
  import ZoomedPageViewer from './ZoomedPageViewer.svelte'
  import type { SearchResult } from './@types'
  import { successToast } from '../../stores/toast'
  import { pushToLoadedHistory } from '../../stores/loadedHistory'
  import { getDocumentProxy } from '../../utils/pdf'
  import { debounce } from '../../utils/event'
  import { handleInvokeError } from '../../utils/backend'
  import { returnHome } from '../../utils/route'

  export let filepath: string = ''
  export let buffer: ArrayBuffer
  export let searchResult: SearchResult | undefined

  const DEFAULT_SCALE: number = 1.0
  const SCALE_UNIT: number = 0.2
  const MIN_SCALE: number = SCALE_UNIT
  const MAX_SCALE: number = 5.0

  let scale: number = DEFAULT_SCALE
  let pdfDocument: PDFDocumentProxy
  let pageViewport: PageViewport
  let pageIndexesRows: number[][] = []
  let pageViewerContainers: HTMLDivElement[] = []
  let zoomedPageIndex: number | undefined

  $: {
    if (searchResult) {
      buffer = searchResult.buffer
    }
  }

  onMount(() => {
    try {
      load(buffer)
    } catch (error: unknown) {
      handleInvokeError(error)
      returnHome()
    }

    pushToLoadedHistory(filepath)

    window.addEventListener('resize', debounce(updatePageIndexesRows, 200))
    window.addEventListener('wheel', debounce(handleWheel, 120))

    setTimeout(() => {
      successToast('File opened', 2700)
    }, 400)
  })

  const load = async (buffer: ArrayBuffer) => {
    pdfDocument = await getDocumentProxy(buffer)
    updatePageIndexesRows()
  }

  const handleWheel = (event: WheelEvent) => {
    if (event.ctrlKey) {
      0 < event.deltaY ? decreaseScale() : increaseScale()
    }
  }

  const handlePageViewport = (event: CustomEvent<PageViewport>) => {
    const viewport = event.detail
    pageViewport = viewport

    updatePageIndexesRows()
  }

  const updatePageIndexesRows = () => {
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

  const pageIndexesPerRow = (): number => {
    if (!pdfDocument) return 0
    if (!pageViewport) return pdfDocument.numPages
    return Math.floor(window.innerWidth / pageViewport.width)
  }

  const increaseScale = () => {
    scale = scale + SCALE_UNIT
    if (MAX_SCALE < scale) {
      scale = MAX_SCALE
    }
  }
  const decreaseScale = () => {
    scale = scale - SCALE_UNIT
    if (scale < MIN_SCALE) {
      scale = MIN_SCALE
    }
  }

  const showZoomedPage = (pageIndex: number) => {
    // initialize
    zoomedPageIndex = undefined

    zoomedPageIndex = pageIndex
  }
</script>

<nav class="scale">
  <h4>Scale</h4>
  <input type="range" min={MIN_SCALE} max={MAX_SCALE} step={SCALE_UNIT} bind:value={scale} />
</nav>

<div class="document">
  {#if pdfDocument && pdfDocument.numPages}
    {#each pageIndexesRows as pageIndexes}
      <div class="row">
        {#each pageIndexes as pageIndex}
          <div class="col" bind:this={pageViewerContainers[pageIndex]}>
            <div class="tile">
              {#if searchResult && searchResult.matchedPageIndexes.includes(pageIndex)}
                <div class="search-matched">Search matched</div>
              {/if}
              <div class="page">
                <article>
                  <PageViewer
                    {pdfDocument}
                    {pageIndex}
                    {scale}
                    on:pageViewport={handlePageViewport}
                  />
                </article>
                <nav>
                  <div></div>
                  <div>{pageIndex + 1}</div>
                  <button class="zoom" on:click={() => showZoomedPage(pageIndex)} aria-label="zoom"
                    >🧐</button
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

<style>
  nav.scale {
    position: fixed;
    bottom: 1.1rem;
    left: 0.7rem;
    z-index: 1;
    display: flex;
    align-items: flex-end;
  }
  nav.scale h4 {
    font-size: 0.7rem;
    color: #878787;
    padding: 0;
    margin: 0;
  }
  nav.scale input {
    width: 5.7rem;
    margin-left: 0.6rem;
  }

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

  .tile .page {
    position: relative;
  }
  .tile .page article:hover {
    transform: scale(1.02) translateX(-1%) translateY(-1%);
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
  .tile .page:hover nav {
    display: flex;
  }
  .tile .page nav > * {
    font-size: 0.8rem;
    color: #878787;
    text-align: center;
  }
  .tile nav button.zoom {
    padding: 0 0.4rem;
    font-size: 1rem;
  }

  .search-matched {
    width: 100%;
    padding: 0.1rem 0.5rem;
    background-color: #b7a42a;
    color: #ffffff;
    text-align: center;
    font-size: 0.7rem;
    border-radius: 0.05rem;
  }
</style>
