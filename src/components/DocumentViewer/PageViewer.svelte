<script lang="ts">
  import { onMount, createEventDispatcher } from 'svelte'
  import { type PDFDocumentProxy } from 'pdfjs-dist'
  import { EventBus, PDFPageView, RenderingStates } from 'pdfjs-dist/web/pdf_viewer.mjs'
  import 'pdfjs-dist/web/pdf_viewer.css'

  export let pdfDocument: PDFDocumentProxy
  export let pageIndex: number
  export let scale: number

  const dispatch = createEventDispatcher()

  let pageViewerContainer: HTMLDivElement
  let pdfPageView: PDFPageView

  $: {
    if (pdfPageView) {
      pdfPageView.update({
        scale: scale,
      })
      dispatchPageViewport()
      pdfPageView.draw()
    }
  }

  $: {
    if (pdfDocument) {
      draw()
    }
  }

  onMount(draw)

  async function draw() {
    const pdfPage = await pdfDocument.getPage(pageIndex + 1)

    if (!pdfPageView) {
      pdfPageView = new PDFPageView({
        id: pageIndex,
        container: pageViewerContainer,
        defaultViewport: pdfPage.getViewport(),
        eventBus: new EventBus(),
      })
    } else {
      pdfPageView.destroy()
    }
    pdfPageView.setPdfPage(pdfPage)
    pdfPageView.draw()

    dispatchPageViewport()
  }

  function dispatchPageViewport() {
    if (pageIndex !== 0 || pdfPageView.renderingState === RenderingStates.RUNNING) return
    dispatch('pageViewport', pdfPageView.viewport)
  }
</script>

<div class="pdfViewer">
  <div bind:this={pageViewerContainer}></div>
</div>
