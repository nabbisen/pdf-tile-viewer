<script lang="ts">
  // import Header from './Header.svelte'
  import DragDrop from './DragDrop.svelte'
  // import Footer from './Footer.svelte'
  import Toast from './Toast.svelte'

  import '../app.css'
</script>

<!-- <header><Header /></header> -->
<main><slot /></main>
<DragDrop />
<!-- <footer><Footer /></footer> -->

<Toast />
