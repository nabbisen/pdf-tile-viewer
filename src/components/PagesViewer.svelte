<script lang="ts">
  import { onMount } from "svelte";
  import { getDocument , GlobalWorkerOptions, type PageViewport, type PDFDocumentProxy  } from 'pdfjs-dist';
  import 'pdfjs-dist/web/pdf_viewer.css'
  import PageViewer from './PageViewer.svelte'
  import { debounce } from '../utils/event'

  export let buffer: ArrayBuffer

  // interface pageViewerProps {
  //   pdfDocument: PDFDocumentProxy;
  //   pageNum: number;
  //   scale: number;
  // }

  const DEFAULT_SCALE: number = 1.0
  const SCALE_UNIT: number = 0.2
  const MIN_SCALE: number = SCALE_UNIT
  const MAX_SCALE: number = 10.0

  let scale: number = DEFAULT_SCALE
  let pdfDocument: PDFDocumentProxy
  let pageViewport: PageViewport
  let pageNumsRows: number[][] = []
  let pageViewerContainers: HTMLDivElement[] = []
  let zoomedPageNum: number | undefined
  let zoomViewScale: number = 3.0
  let zoomViewOpacity: number = 1.0

  GlobalWorkerOptions.workerSrc = new URL('pdfjs-dist/build/pdf.worker.min.mjs', import.meta.url).toString();
  
  const loadPdfDocument = async () => {
    const CMAP_URL = "pdfjs-dist/cmaps/";
    const CMAP_PACKED = true;

    const loadingTask = getDocument({
      data: buffer,
      url: undefined,
      cMapUrl: CMAP_URL,
      cMapPacked: CMAP_PACKED,
    })

    pdfDocument = await loadingTask.promise;
  }

  const handleWheel = (event: WheelEvent) => {
    if (event.ctrlKey) {
      (0 < event.deltaY) ? decreaseScale() : increaseScale()
    }
  }

  const onMountHandler = async () => {
    await loadPdfDocument()

    updatePageNumsRows()

    window.addEventListener('resize', debounce(updatePageNumsRows, 200))
    window.addEventListener('wheel', debounce(handleWheel, 120))
  }
  onMount(onMountHandler)

  function handlePageViewport(event: CustomEvent<PageViewport>) {
    const viewport = event.detail
    pageViewport = viewport

    updatePageNumsRows()
  }

  function handleZoomClick(event: CustomEvent<number>) {
    zoomedPageNum = event.detail
  }

  function pagesNumPerRow(): number {
    if (!pdfDocument) return 0
    if (!pageViewport) return pdfDocument.numPages
    return Math.floor(window.innerWidth / pageViewport.width)
  }

  function updatePageNumsRows() {
    let ret: number[][] = []
    
    const rowBreak = pagesNumPerRow()
    let row: number[] = []
    for (let pageNum = 1; pageNum <= pdfDocument.numPages; pageNum++) {
      row.push(pageNum)
      if (pageNum % rowBreak === 0) {
        ret.push(row)
        row = []
      }
    }
    if (0 < row.length) {
      ret.push(row)
    }
    
    pageNumsRows = ret
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
</script>

{#if pdfDocument && pdfDocument.numPages }
  {#each pageNumsRows as pageNums}
    <div class="row">
      {#each pageNums as pageNum}
        <div class="col" bind:this={pageViewerContainers[pageNum]}>
          <PageViewer pdfDocument={pdfDocument} pageNum={pageNum} scale={scale} on:pageViewport={handlePageViewport} on:zoomClick={handleZoomClick} />
        </div>
      {/each}
    </div>
  {/each}
{/if}

{#if zoomedPageNum }
  <div class="zoomView">
    <div class="wrapper" style={`opacity: ${zoomViewOpacity};`}>
      <PageViewer pdfDocument={pdfDocument} pageNum={zoomedPageNum} scale={zoomViewScale} />
    </div>
    <nav>
      <span class="pageNum">p.{zoomedPageNum}</span>
      <label>Scale
        <input type="number" step="0.1" min="0.1" max="10.0" bind:value={zoomViewScale}>
      </label>
      <label>Transparency
        <input type="number" step="0.1" min="0.0" max="1.0" bind:value={zoomViewOpacity}>
      </label>
      <button on:click={() => zoomedPageNum = undefined}>Close</button>
    </nav>
  </div>
{/if}

<!-- todo -->
<!-- {#if zoomedPageViewerProps }
  <PageViewer pdfDocument={zoomedPageViewerProps.pdfDocument} pageNum={zoomedPageViewerProps.pageNum} scale={zoomedPageViewerProps.scale} />
{/if} -->

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
    overflow: scroll;
  }
  .zoomView nav {
    width: 100%;
    height: 1.75rem;
    display: flex;
    justify-content: space-around;
    align-items: center;
    background-color: #2aabb7;
    color: #ffffff;
  }
  .zoomView nav * {
    font-size: 1.0rem;
  }
  .zoomView nav input {
    width: 3.2em;
    text-align: right;
  }
</style>