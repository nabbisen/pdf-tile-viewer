<script lang="ts">
  import 'pdfjs-dist/web/pdf_viewer.css'
  import Header from './Header.svelte'
  import PagesTileViewer from './PagesTileViewer.svelte'
  import { getDocumentBuffer } from '../../utils/pdf'
  import { handleInvokeError } from '../../utils/backend'
  import { returnHome } from '../../utils/route'
  import { subscribeFilepath, setBuffer } from '../../stores/pages/documentViewer'
  import MouseDragMove from './MouseDragMove.svelte'

  let filepath: string | undefined

  $: {
    subscribeFilepath((value) => (filepath = value))
  }

  $: {
    if (filepath) load()
  }

  function load() {
    getDocumentBuffer(filepath!)
      .then((res) => {
        const buffer = new Uint8Array(res).buffer as ArrayBuffer
        setBuffer(buffer)
      })
      .catch((error: unknown) => {
        handleInvokeError(error)
        returnHome()
      })
  }
</script>

<Header {filepath} />
<PagesTileViewer {filepath} />

<MouseDragMove />
