<script lang="ts">
  import { onMount } from 'svelte'
  import { type PageViewport, type PDFDocumentProxy } from 'pdfjs-dist'
  import PageViewer from './PageViewer.svelte'
  import ZoomedPageViewer from './ZoomedPageViewer.svelte'
  import type { SearchResult } from './@types'
  import { debounce } from '../../utils/event'

  export let pageBuffers: ArrayBuffer[] = []
  export let searchResult: SearchResult | undefined

  const DEFAULT_SCALE: number = 1.0
  const SCALE_UNIT: number = 0.2
  const MIN_SCALE: number = SCALE_UNIT
  const MAX_SCALE: number = 10.0

  let scale: number = DEFAULT_SCALE
  let pageViewport: PageViewport
  let pageIndexesRows: number[][] = []
  let pageViewerContainers: HTMLDivElement[] = []
  let zoomedPageIndex: number | undefined
  let zoomedPageBuffer: ArrayBuffer | undefined

  $: {
    if (0 < pageBuffers.length) {
      updatePageIndexesRows()
    }
  }

  $: {
    if (searchResult) {
      pageBuffers = searchResult.pageBuffers
    }
  }

  onMount(() => {
    window.addEventListener('resize', debounce(updatePageIndexesRows, 200))
    window.addEventListener('wheel', debounce(handleWheel, 120))
  })

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
    for (let pageIndex = 0; pageIndex < pageBuffers.length; pageIndex++) {
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
    if (pageBuffers.length === 0) return 0
    if (!pageViewport) return pageBuffers.length
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
    zoomedPageBuffer = pageBuffers[pageIndex]
  }
</script>

{#if 0 < pageBuffers.length}
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
                  pageBuffer={pageBuffers[pageIndex]}
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

<ZoomedPageViewer pageIndex={zoomedPageIndex} pageBuffer={zoomedPageBuffer} />

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
