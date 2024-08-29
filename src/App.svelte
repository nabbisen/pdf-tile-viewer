<script lang="ts">
  import { onDestroy, onMount } from 'svelte'
  import { GlobalWorkerOptions } from 'pdfjs-dist'
  import { setWindowHeight, setWindowWidth } from './stores/settings/app_window'
  import { debounce } from './utils/event'

  // import Header from './Header.svelte'
  import DragDrop from './components/DragDrop.svelte'
  // import Footer from './Footer.svelte'
  import Toast from './components/Toast.svelte'
  import Loader from './components/Loader.svelte'

  import './app.css'

  const debounceStoreWindowSize = debounce(storeWindowSize, 100)

  onMount(appOnMount)
  onDestroy(appOnDestroy)

  function appOnMount() {
    window.addEventListener('resize', debounceStoreWindowSize)
  }

  function appOnDestroy() {
    window.removeEventListener('resize', debounceStoreWindowSize)
  }

  function storeWindowSize(_e: UIEvent) {
    setWindowWidth(window.outerWidth)
    setWindowHeight(window.outerHeight)
  }

  // initialize pdf.js worker
  GlobalWorkerOptions.workerSrc = new URL(
    'pdfjs-dist/build/pdf.worker.min.mjs',
    import.meta.url
  ).toString()
</script>

<!-- <header><Header /></header> -->
<main><slot /></main>
<DragDrop />
<!-- <footer><Footer /></footer> -->

<Toast />
<Loader />
