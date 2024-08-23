<!-- ref only component -->

<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { getDocument , GlobalWorkerOptions, type PDFDocumentProxy, type PDFPageProxy } from 'pdfjs-dist';
  import { EventBus, PDFViewer } from 'pdfjs-dist/web/pdf_viewer.mjs';
  import 'pdfjs-dist/web/pdf_viewer.css'

  let buffer: ArrayBuffer
  let pdfDocument: PDFDocumentProxy

  let mainContainer: HTMLDivElement
  let viewerContainer: HTMLDivElement

  GlobalWorkerOptions.workerSrc = new URL('pdfjs-dist/build/pdf.worker.min.mjs', import.meta.url).toString();
  const loadPdfBuffer = async () => {
    // const file = '/home/nabbisen/Downloads/Levy_S-Hackers-Heroes-Computer-Revolution.pdf'
    const file = '/home/nabbisen/Downloads/pdf.js-test.pdf'
    const res: Array<any> = await invoke("read_pdf", { filepath: file })
    buffer = new Uint8Array(res).buffer
  }
  const loadPdfDocument = async () => {
    const CMAP_URL = "pdfjs-dist/cmaps/";
    const CMAP_PACKED = true;

    const ENABLE_XFA = true

    const loadingTask = getDocument({
      data: buffer,
      url: undefined,
      cMapUrl: CMAP_URL,
      cMapPacked: CMAP_PACKED,
      enableXfa: ENABLE_XFA,
    })

    pdfDocument = await loadingTask.promise;
  }

  const onMountHandler = async () => {
    await loadPdfBuffer()
    await loadPdfDocument()

    const pdfViewer = new PDFViewer({
      container: mainContainer,
      viewer: viewerContainer,
      eventBus: new EventBus(),
    })
    pdfViewer.setDocument(pdfDocument)
  }
  onMount(onMountHandler)
</script>

<div>{window.devicePixelRatio}</div>
<div>
  <div bind:this={mainContainer} style="position: absolute; top: 2em; left: 0;">
    <div bind:this={viewerContainer} class="pdfViewer"></div>
  </div>
</div>
