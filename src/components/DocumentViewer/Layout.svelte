<script lang="ts">
  import { onMount } from 'svelte'
  import { goto } from '$app/navigation'
  import { page } from '$app/stores'
  import { invoke } from '@tauri-apps/api/core'
  import {
    getDocument,
    GlobalWorkerOptions,
    type PageViewport,
    type PDFDocumentProxy,
  } from 'pdfjs-dist'
  import 'pdfjs-dist/web/pdf_viewer.css'
  import PageViewer from './PageViewer.svelte'
  import { infoToast, successToast } from '../../stores/toast'
  import { debounce } from '../../utils/event'
  import { handleInvokeError } from '../../utils/backend'
  import { filename } from '../../utils/file'
  import { pushToLoadedHistory } from '../../stores/loadedHistory'

  let filepath: string
  $: {
    filepath = decodeURIComponent($page.url.searchParams.get('filepath')!)
  }

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
  let zoomViewScale: number = 3.0
  let zoomViewOpacity: number = 1.0

  GlobalWorkerOptions.workerSrc = new URL(
    'pdfjs-dist/build/pdf.worker.min.mjs',
    import.meta.url
  ).toString()

  $: {
    if (zoomedPageIndex) {
      window.document.body.style.overflow = 'hidden'
    } else {
      window.document.body.style.overflow = 'auto'
    }
  }

  const loadPdfBuffer = async () => {
    let res: Array<any>
    try {
      res = await invoke('read_pdf', { filepath: filepath })
    } catch (error: unknown) {
      handleInvokeError(error)
      returnHome()
      return
    }
    const buffer = new Uint8Array(res).buffer
    try {
      await loadPdfDocument(buffer)
    } catch (error: unknown) {
      handleInvokeError(error)
      returnHome()
      return
    }

    pushToLoadedHistory(filepath)
  }

  const returnHome = () => {
    goto('/dashboard')
  }

  const loadPdfDocument = async (buffer: ArrayBuffer) => {
    const CMAP_URL = 'pdfjs-dist/cmaps/'
    const CMAP_PACKED = true

    const loadingTask = getDocument({
      data: buffer,
      url: undefined,
      cMapUrl: CMAP_URL,
      cMapPacked: CMAP_PACKED,
    })

    pdfDocument = await loadingTask.promise
  }

  const handleWheel = (event: WheelEvent) => {
    if (event.ctrlKey) {
      0 < event.deltaY ? decreaseScale() : increaseScale()
    }
  }

  const onMountHandler = async () => {
    await loadPdfBuffer()

    updatePageIndexesRows()

    window.addEventListener('resize', debounce(updatePageIndexesRows, 200))
    window.addEventListener('wheel', debounce(handleWheel, 120))

    window.addEventListener('keydown', (e: KeyboardEvent) => {
      if (e.key === 'Escape' || e.key === 'Esc') {
        zoomedPageIndex = undefined
      }
    })

    setTimeout(() => {
      successToast('File opened', 2700)
    }, 400)
  }
  onMount(onMountHandler)

  function handlePageViewport(event: CustomEvent<PageViewport>) {
    const viewport = event.detail
    pageViewport = viewport

    updatePageIndexesRows()
  }

  function handleZoomClick(event: CustomEvent<number>) {
    zoomedPageIndex = event.detail
  }

  function pageIndexesPerRow(): number {
    if (!pdfDocument) return 0
    if (!pageViewport) return pdfDocument.numPages
    return Math.floor(window.innerWidth / pageViewport.width)
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

  const SEARCH_TERM_MIN_LENGTH: number = 3
  const SEQUENTIAL_CONCAT: string = '~'
  let matchingPageIndexes: number[] = []
  let searchTerm: string = ''
  let confirmedSearchTerm: string = ''
  let isSearchFormVisible: boolean = false

  const searchPdf = async () => {
    confirmedSearchTerm = ''

    let res: any
    try {
      res = await invoke('search_pdf', {
        searchTerm: searchTerm,
        filepath: filepath,
      })
    } catch (error: unknown) {
      handleInvokeError(error)
      return
    }
    const buffer = res.buffer as ArrayBuffer
    await loadPdfDocument(buffer)

    matchingPageIndexes = res.page_indexes as number[]
    if (0 < matchingPageIndexes.length) {
      successToast(`Matches: p.${displayMatchedPages(matchingPageIndexes)}`)
    } else {
      infoToast('No matches')
    }

    confirmedSearchTerm = searchTerm

    isSearchFormVisible = false
  }

  const displayMatchedPages = (pageIndexes: number[]): string => {
    const ret = pageIndexes
      .map((pageIndex, arrayIndex, array) => {
        const pageNum = pageIndex + 1
        if (arrayIndex === 0 || arrayIndex === pageIndexes.length - 1) {
          return pageNum.toString()
        }
        const prevPageIndex = array[arrayIndex - 1]
        const nextPageIndex = array[arrayIndex + 1]
        const isInSequentialProc =
          pageIndex === prevPageIndex + 1 && pageIndex + 1 === nextPageIndex
        return isInSequentialProc ? SEQUENTIAL_CONCAT : pageNum.toString()
      })
      .filter((pageStr, arrayIndex, array) => {
        if (arrayIndex === 0 || arrayIndex === matchingPageIndexes.length - 1) {
          return true
        }
        const prevPageStr = array[arrayIndex - 1]
        return `${prevPageStr}${pageStr}` !== SEQUENTIAL_CONCAT.repeat(2)
      })
      .join(', ')
      .replaceAll(`, ${SEQUENTIAL_CONCAT}, `, SEQUENTIAL_CONCAT)
    return ret
  }

  const toggleSearchForm = () => {
    isSearchFormVisible = !isSearchFormVisible
  }
</script>

<header>
  <h2>{filename(filepath)}</h2>

  <nav>
    <div class="search">
      <button class="toggle" on:click={toggleSearchForm}>🔍</button>
      {#if isSearchFormVisible}
        <form>
          <input
            type="text"
            bind:value={searchTerm}
            placeholder={`${SEARCH_TERM_MIN_LENGTH} chars or more`}
          />
          <button
            class="search"
            disabled={searchTerm.length < SEARCH_TERM_MIN_LENGTH}
            on:click={searchPdf}>Search</button
          >
          <button class="close" on:click={toggleSearchForm}>Close</button>
        </form>
      {/if}
    </div>

    <div class="logo">
      <button on:click={returnHome}>
        <h1>Home</h1>
      </button>
    </div>
  </nav>
</header>

{#if pdfDocument && pdfDocument.numPages}
  {#each pageIndexesRows as pageIndexes}
    <div class="row">
      {#each pageIndexes as pageIndex}
        <div class="col" bind:this={pageViewerContainers[pageIndex]}>
          {#if matchingPageIndexes.includes(pageIndex)}
            <div class="search-matching">"{confirmedSearchTerm}" matched</div>
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

{#if zoomedPageIndex !== undefined}
  <div class="zoomView">
    <div class="wrapper" style={`opacity: ${zoomViewOpacity};`}>
      <PageViewer {pdfDocument} pageIndex={zoomedPageIndex} scale={zoomViewScale} />
    </div>
    <nav>
      <span class="pageIndex">p.{zoomedPageIndex + 1}</span>
      <label
        >Scale
        <input type="number" step="0.1" min="0.1" max="10.0" bind:value={zoomViewScale} />
      </label>
      <label
        >Transparency
        <input type="number" step="0.1" min="0.0" max="1.0" bind:value={zoomViewOpacity} />
      </label>
      <button class="close" on:click={() => (zoomedPageIndex = undefined)}>Close</button>
    </nav>
  </div>
{/if}

<style>
  header,
  h1 {
    font-size: 0.7rem;
  }
  h1 {
    color: #878787;
  }
  h1::after {
    content: '🏠';
    display: inline-block;
    padding: 0 0.7rem;
  }
  header nav {
    position: fixed;
    right: 0.8rem;
    bottom: 0.5rem;
    width: 4.4em;
    display: flex;
    flex-direction: column;
    justify-content: center;
    align-items: center;
    z-index: 20000;
  }
  .logo button {
    background-color: #efefef;
    font-size: 0.6rem;
    box-shadow: none;
    border: none;
    text-align: center;
    cursor: pointer;
  }
  .logo button:hover {
    opacity: 0.87;
  }

  .search {
    position: relative;
  }
  .search button {
    padding: 0.3rem 0.7rem;
    background: none;
    border: none;
    border-radius: 0.08rem;
  }
  .search button:hover {
    opacity: 0.87;
  }
  .search button.toggle {
    padding: 1.1rem 0.7rem;
    font-size: 0.8rem;
  }
  .search form {
    position: absolute;
    right: 3.2rem;
    bottom: 1.6em;
    width: 20.2rem;
    padding: 1.4rem 0.8rem 1.1rem;
    display: flex;
    flex-direction: column;
    background-color: #25252587;
  }
  .search form > * {
    margin: 0.5rem 0;
    text-align: center;
  }
  .search form input {
    padding: 0.6rem 0.3rem;
    font-size: 1.33rem;
  }
  .search form button.search {
    padding-top: 0.5rem;
    padding-bottom: 0.5rem;
    background-color: #b7a42a;
    color: #ffffff;
    font-size: 1.2rem;
  }
  .search form button.search:disabled {
    background-color: #eaeaea;
    color: #bbbbbb;
  }
  .search form button.close {
    width: fit-content;
    margin: 1.1rem auto 0;
    background-color: #ffffff;
    color: #252525;
    font-size: 0.9rem;
  }

  .search-matching {
    width: 100%;
    background-color: #b7a42a;
    color: #ffffff;
    text-align: center;
    font-size: 0.7rem;
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
  .zoomView nav input {
    width: 3.2em;
    text-align: right;
  }
  .zoomView nav button.close {
    background-color: #ffffff;
    color: #252525;
  }
</style>
