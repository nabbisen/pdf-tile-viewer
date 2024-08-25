<script lang="ts">
  import { onMount, createEventDispatcher } from 'svelte'
  import { EventBus, PDFPageView, RenderingStates } from 'pdfjs-dist/web/pdf_viewer.mjs'
  import 'pdfjs-dist/web/pdf_viewer.css'
  import { getDocumentProxy } from '../../utils/pdf'

  export let pageIndex: number
  export let pageBuffer: ArrayBuffer
  export let scale: number

  const dispatch = createEventDispatcher()

  let pageViewerContainer: HTMLDivElement
  let pdfPageView: PDFPageView

  const draw = async () => {
    const pdfDocument = await getDocumentProxy(pageBuffer)
    const pdfPage = await pdfDocument.getPage(1)
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

  onMount(draw)

  function dispatchPageViewport() {
    if (pageIndex !== 0 || pdfPageView.renderingState === RenderingStates.RUNNING) return
    dispatch('pageViewport', pdfPageView.viewport)
  }

  $: {
    if (pdfPageView) {
      pdfPageView.update({
        scale: scale,
      })
      dispatchPageViewport()
      pdfPageView.draw()
    }
  }
</script>

<div class="pdfViewer">
  <div bind:this={pageViewerContainer}></div>
</div>
