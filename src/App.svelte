<script lang="ts">
  import { GlobalWorkerOptions } from 'pdfjs-dist'

  // import Header from './Header.svelte'
  import DragDrop from './components/DragDrop.svelte'
  // import Footer from './Footer.svelte'
  import Toast from './components/Toast.svelte'
  import Loader from './components/Loader.svelte'

  import './app.css'

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
