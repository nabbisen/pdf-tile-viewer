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
  const MAX_SCALE: number = 10.0

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

  const handleZoomClick = (event: CustomEvent<number>) => {
    zoomedPageIndex = event.detail
  }

  const handleCloseZoomedPage = () => {
    zoomedPageIndex = undefined
  }
</script>

{#if pdfDocument && pdfDocument.numPages}
  {#each pageIndexesRows as pageIndexes}
    <div class="row">
      {#each pageIndexes as pageIndex}
        <div class="col" bind:this={pageViewerContainers[pageIndex]}>
          {#if searchResult && searchResult.matchedPageIndexes.includes(pageIndex)}
            <div class="search-matching">"{searchResult.confirmedSearchTerm}" matched</div>
          {/if}
          <PageViewer
            {pdfDocument}
            {pageIndex}
            {scale}
            on:pageViewport={handlePageViewport}
            on:zoomClick={handleZoomClick}
          />
        </div>
      {/each}
    </div>
  {/each}
{/if}

<ZoomedPageViewer pageIndex={zoomedPageIndex} {pdfDocument} on:close={handleCloseZoomedPage} />

<style>
  .row {
    display: flex;
  }
  .row:not(:first-child) {
    border-bottom: 0.02rem solid #bbbbbb;
  }
  .row:not(:last-child) {
    border-top: 0.02rem solid #bbbbbb;
  }
  .row .col:not(:first-child) {
    border-left: 0.02rem solid #bbbbbb;
  }
  .row .col:not(:last-child) {
    border-left: 0.02rem solid #bbbbbb;
  }

  .search-matching {
    width: 100%;
    background-color: #b7a42a;
    color: #ffffff;
    text-align: center;
    font-size: 0.7rem;
  }
</style>
