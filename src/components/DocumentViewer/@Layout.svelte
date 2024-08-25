<script lang="ts">
  import { onMount } from 'svelte'
  import { page } from '$app/stores'
  import 'pdfjs-dist/web/pdf_viewer.css'
  import Header from './Header.svelte'
  import PagesTileViewer from './PagesTileViewer.svelte'
  import type { SearchResult } from './@types'
  import { getPageBuffers } from '../../utils/pdf'
  import { successToast } from '../../stores/toast'
  import { pushToLoadedHistory } from '../../stores/loadedHistory'
  import { handleInvokeError } from '../../utils/backend'
  import { returnHome } from '../../utils/route'

  let filepath: string = ''

  onMount(() => {
    filepath = $page.url.searchParams.get('filepath')!
    load(filepath)
  })

  let pageBuffers: ArrayBuffer[] | undefined
  let searchResult: SearchResult | undefined

  const load = (filepath: string) => {
    try {
      getPageBuffers(filepath).then((x) => {
        pageBuffers = x
      })
    } catch (error: unknown) {
      handleInvokeError(error)
      returnHome()
    }

    pushToLoadedHistory(filepath)

    setTimeout(() => {
      successToast('File opened', 2700)
    }, 400)
  }

  const handleSearch = (e: CustomEvent<SearchResult>) => {
    searchResult = e.detail
  }
</script>

<Header {filepath} on:search={handleSearch} />
{#if pageBuffers}
  <PagesTileViewer {pageBuffers} {searchResult} />
{/if}
