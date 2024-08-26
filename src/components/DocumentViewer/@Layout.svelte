<script lang="ts">
  import { onMount } from 'svelte'
  import { page } from '$app/stores'
  import 'pdfjs-dist/web/pdf_viewer.css'
  import Header from './Header.svelte'
  import PagesTileViewer from './PagesTileViewer.svelte'
  import type { SearchResult } from './@types'
  import { getDocumentBuffer } from '../../utils/pdf'
  import { handleInvokeError } from '../../utils/backend'
  import { returnHome } from '../../utils/route'

  let filepath: string = ''

  onMount(() => {
    filepath = $page.url.searchParams.get('filepath')!
    load(filepath)
  })

  let buffer: ArrayBuffer | undefined
  let searchResult: SearchResult | undefined

  const load = (filepath: string) => {
    getDocumentBuffer(filepath)
      .then((x) => {
        buffer = x
      })
      .catch((error: unknown) => {
        handleInvokeError(error)
        returnHome()
      })
  }

  const handleSearch = (e: CustomEvent<SearchResult>) => {
    searchResult = e.detail
  }
</script>

<Header {filepath} on:search={handleSearch} />
{#if buffer}
  <PagesTileViewer {filepath} {buffer} {searchResult} />
{/if}
