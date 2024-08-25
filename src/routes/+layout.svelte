<script lang="ts">
  // import Header from '../layouts/Header.svelte'
  import DragDrop from '../layouts/DragDrop.svelte'
  // import Footer from '../layouts/Footer.svelte'
  import Toast from '../layouts/Toast.svelte'

  import '../app.css'
</script>

<!-- <header><Header /></header> -->
<main><slot /></main>
<DragDrop />
<!-- <footer><Footer /></footer> -->

<Toast />
