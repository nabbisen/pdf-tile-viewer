<script lang="ts">
  import { onMount, createEventDispatcher } from "svelte";
  import { type PDFDocumentProxy } from 'pdfjs-dist';
  import { EventBus, PDFPageView, RenderingStates } from 'pdfjs-dist/web/pdf_viewer.mjs';
  import 'pdfjs-dist/web/pdf_viewer.css'

  export let pdfDocument: PDFDocumentProxy;
  export let pageNum: number;
  export let scale: number;
  
  const dispatch = createEventDispatcher()

  let pageViewerContainer: HTMLDivElement
  let pdfPageView: PDFPageView

  const onMountHandler = async () => {
    const pdfPage = await pdfDocument.getPage(pageNum)
    pdfPageView = new PDFPageView({
      id: pageNum,
      container: pageViewerContainer,
      defaultViewport: pdfPage.getViewport(),
      eventBus: new EventBus(),
    })
    pdfPageView.setPdfPage(pdfPage)
    pdfPageView.draw()

    dispatchPageViewport()
  }
  onMount(onMountHandler)

  function dispatchPageViewport() {
    if (pageNum !== 1 || pdfPageView.renderingState === RenderingStates.RUNNING) return
    dispatch('pageViewport', pdfPageView.viewport)
  }

  function handleClick(event: MouseEvent) {
    if (!event.ctrlKey) return
    dispatch('ctrlClick', { pdfDocument, pageNum, scale })
    event.preventDefault()
  }

  $: {
    if (pdfPageView) {
      pdfPageView.update({
        scale: scale
      })
      dispatchPageViewport()
      pdfPageView.draw()
    }
  }
</script>

<div class="container main" data-page-num={pageNum}>
  <div class="pdfViewer">
    <div bind:this={pageViewerContainer}></div>
  </div>
  <!-- <input type="checkbox" on:click={handleClick}> -->
</div>

<style>
  .container.main {
    position: relative;
  }
  .container.main:hover {
  /* .container.main:has(input:checked) { */
    transform: scale(1.02) translateX(-1%) translateY(-1%);
  }
  .container.main:hover::before {
    content: attr(data-page-num);
    position: absolute;
    left: 1.1rem;
    top: 1.1rem;
    color: #bbbbbb;
    font-size: 0.8rem;
    font-weight: bold;
    z-index: 1;
  }
  /* input {
    position: absolute;
    left: 0;
    top: 0;
    width: 100%;
    height: 100%;
    opacity: 0.0;
  }
  input:checked {
    background-color: #ffffef;
    opacity: 0.06;
  } */
</style>
