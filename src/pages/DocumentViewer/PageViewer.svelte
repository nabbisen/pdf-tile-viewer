<script lang="ts">
  import { onDestroy, onMount, createEventDispatcher } from 'svelte'
  import { type PDFDocumentProxy, type PDFPageProxy } from 'pdfjs-dist'
  import { EventBus, PDFPageView, RenderingStates } from 'pdfjs-dist/web/pdf_viewer.mjs'
  import 'pdfjs-dist/web/pdf_viewer.css'
  import type { PageViewerClass } from '../../types/pages/documentViewer'

  export let pdfDocument: PDFDocumentProxy
  export let pageIndex: number
  export let scale: number
  export let viewerClass: PageViewerClass

  const dispatch = createEventDispatcher()

  let initialized: boolean = false

  let pageViewerContainer: HTMLDivElement
  let pdfPageView: PDFPageView

  $: {
    if (pdfPageView) initialized = true
  }

  $: {
    if (initialized) scaleOnChange(scale)
  }

  $: {
    if (initialized) pageIndexOnChange(pageIndex)
  }

  onMount(loadPage)

  onDestroy(unloadPage)

  async function loadPage() {
    const pdfPage = await pdfDocument.getPage(pageIndex + 1)

    pdfPageView = new PDFPageView({
      id: pageIndex,
      container: pageViewerContainer,
      defaultViewport: pdfPage.getViewport(),
      scale: scale,
      eventBus: new EventBus(),
    })
    pdfPageView.setPdfPage(pdfPage)
    dispatchPageViewport()

    pdfPageView.draw()
  }

  function unloadPage() {
    if (!pdfPageView) return
    pdfPageView.destroy()
  }

  function pageIndexOnChange(pageIndex: number) {
    pdfDocument.getPage(pageIndex + 1).then((pdfPage: PDFPageProxy) => {
      pdfPageView.setPdfPage(pdfPage)
      dispatchPageViewport()

      pdfPageView.draw()
    })
  }

  function scaleOnChange(scale: number) {
    pdfPageView.update({
      scale: scale,
    })
    dispatchPageViewport()

    pdfPageView.draw()
  }

  function dispatchPageViewport() {
    if (pageIndex !== 0 || pdfPageView.renderingState === RenderingStates.RUNNING) return

    dispatch('pageViewport', pdfPageView.viewport)
  }
</script>

<div class={`pdfViewer viewer ${viewerClass}`}>
  <div bind:this={pageViewerContainer}></div>
</div>

<style>
  .viewer.zoom {
    overflow: scroll;
    height: 100%;
    width: 100%;
  }
</style>
