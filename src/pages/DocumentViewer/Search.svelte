<script lang="ts">
  import { onDestroy, onMount } from 'svelte'
  import { invoke } from '@tauri-apps/api/core'
  import { handleInvokeError } from '../../utils/backend'
  import {
    setBuffer,
    setMatchedPageIndexes,
    setConfirmedSearchTerm,
    setDisplayMatchedPages,
    reload,
  } from '../../stores/pages/documentViewer'
  import {
    subscribeConfirmedSearchTerm,
    subscribeDisplayMatchedPages,
  } from '../../stores/pages/documentViewer'
  import { infoToast, successToast } from '../../stores/components/toast'
  import { loaderStart, loaderStop } from '../../stores/components/loader'
  import Tooltip from '../../components/Tooltip.svelte'

  export let filepath: string = ''

  const SEARCH_TERM_MIN_LENGTH: number = 3
  const SEQUENTIAL_CONCAT: string = '~'

  let confirmedSearchTerm: string | undefined
  let displayMatchedPages: string | undefined

  let searchTerm: string = ''
  let searchFormVisible: boolean = false

  $: {
    subscribeConfirmedSearchTerm((value) => (confirmedSearchTerm = value))
  }

  $: {
    subscribeDisplayMatchedPages((value) => (displayMatchedPages = value))
  }

  onMount(() => {
    window.addEventListener('keydown', windowOnKeydown)
  })

  onDestroy(() => {
    window.removeEventListener('keydown', windowOnKeydown)
  })

  function windowOnKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape' || e.key === 'Esc') {
      searchFormVisible = false
    }
  }

  function searchPdf() {
    if (searchTerm && searchTerm === confirmedSearchTerm) return

    loaderStart()

    invoke('pdf_search', {
      searchTerm: searchTerm,
      filepath: filepath,
    })
      .then(searchCallback)
      .catch((error: unknown) => {
        handleInvokeError(error)
        return
      })
  }

  function searchCallback(res: any) {
    const buffer = res.buffer as ArrayBuffer
    setBuffer(buffer)

    const matchedPageIndexes = res.page_indexes as number[]
    if (0 < matchedPageIndexes.length) {
      const displayMatchedPages = getDisplayMatchedPages(matchedPageIndexes)
      successToast(`Matched: ${displayMatchedPages}`)
      setDisplayMatchedPages(displayMatchedPages)
    } else {
      infoToast('No matched')
      setDisplayMatchedPages(undefined)
    }
    setMatchedPageIndexes(matchedPageIndexes)

    confirmedSearchTerm = searchTerm
    setConfirmedSearchTerm(confirmedSearchTerm)

    searchFormVisible = false

    loaderStop()
  }

  function toggleSearchForm() {
    searchFormVisible = !searchFormVisible
  }

  const getDisplayMatchedPages = (pageIndexes: number[]): string => {
    const ret = pageIndexes
      .map((pageIndex, arrayIndex, array) => {
        const pageNum = pageIndex + 1
        if (arrayIndex === 0 || arrayIndex === pageIndexes.length - 1) {
          return pageNum.toString()
        }
        const prevPageIndex = array[arrayIndex - 1]
        const nextPageIndex = array[arrayIndex + 1]
        const isInSequentialProc =
          pageIndex === prevPageIndex + 1 && pageIndex + 1 === nextPageIndex
        return isInSequentialProc ? SEQUENTIAL_CONCAT : pageNum.toString()
      })
      .filter((pageStr, arrayIndex, array) => {
        if (arrayIndex === 0 || arrayIndex === pageIndexes.length - 1) {
          return true
        }
        const prevPageStr = array[arrayIndex - 1]
        return `${prevPageStr}${pageStr}` !== SEQUENTIAL_CONCAT.repeat(2)
      })
      .join(', ')
      .replaceAll(`, ${SEQUENTIAL_CONCAT}, `, SEQUENTIAL_CONCAT)
    return `p.${ret}`
  }

  function clear() {
    searchFormVisible = false
    reload(filepath)
  }
</script>

<div class="search-result">
  {#if confirmedSearchTerm && 0 < confirmedSearchTerm.length}
    <div class="confirmed-search-term">
      <h3>Keyword</h3>
      <div>{confirmedSearchTerm}</div>
    </div>
    <div class="search-matched">
      {#if displayMatchedPages}
        <h4>matched</h4>
        <strong>{displayMatchedPages}</strong>
      {:else}
        <div class="no-matched">no matched</div>
      {/if}
    </div>
  {/if}
</div>

<div class="search">
  <Tooltip messages="Search" position="left">
    <button class="toggle" on:click={toggleSearchForm}>🔍</button>
  </Tooltip>
  {#if searchFormVisible}
    <form>
      <input
        type="text"
        bind:value={searchTerm}
        placeholder={`${SEARCH_TERM_MIN_LENGTH} chars or more`}
      />
      <button
        class="search"
        disabled={searchTerm.length < SEARCH_TERM_MIN_LENGTH}
        on:click={searchPdf}>Search</button
      >
      <footer>
        <button class="clear" on:click={clear} disabled={confirmedSearchTerm === undefined}
          >Clear 🈳
        </button>
        <button class="close auxiliary" on:click={toggleSearchForm}>Close</button>
      </footer>
    </form>
  {/if}
</div>

<style>
  .search-result h3,
  .search-result h4 {
    padding: 0;
    margin: 0;
    font-size: 0.6rem;
    font-weight: normal;
  }
  .search-result h3 {
    margin-bottom: 0.4rem;
  }
  .search-result h4 {
    margin-bottom: 0.1rem;
  }
  .search-result h4::after {
    content: ':';
  }

  .search-result .confirmed-search-term,
  .search-result .search-matched {
    margin: 0.6rem 0 0.1rem;
    color: #b7a42a;
    text-align: center;
  }
  .search-result .confirmed-search-term div {
    padding: 0.7rem 0.3rem;
    border: 0.04rem #b7a42a solid;
    border-radius: 0.07rem;
  }
  .search-result .search-matched {
    max-width: 3.2rem;
    line-height: 1.4em;
    text-align: right;
  }
  .search-result .search-matched .no-matched {
    text-align: center;
  }
  .search-result .search-matched strong {
    font-size: 0.57rem;
  }

  .search {
    position: relative;
  }
  .search button {
    padding: 0.3rem 0.7rem;
    background: none;
    border: none;
    border-radius: 0.08rem;
  }
  .search button:hover {
    opacity: 0.87;
  }
  .search button.toggle {
    padding: 1.1rem 0.7rem;
    font-size: 0.8rem;
  }
  .search form {
    position: absolute;
    right: 3.2rem;
    bottom: 1.6em;
    width: 20.2rem;
    padding: 1.4rem 0.8rem 1.1rem;
    display: flex;
    flex-direction: column;
    background-color: #25252587;
  }
  .search form > * {
    margin: 0.5rem 0;
    text-align: center;
  }
  .search form input {
    padding: 0.6rem 0.3rem;
    font-size: 1.33rem;
  }
  .search form button.search {
    padding-top: 0.5rem;
    padding-bottom: 0.5rem;
    background-color: #b7a42a;
    color: #ffffff;
    font-size: 1.2rem;
  }
  .search form footer {
    width: 14.4rem;
    margin: 0 auto;
    display: flex;
    justify-content: space-around;
  }
  .search form footer button {
    width: fit-content;
    margin: 1.1rem auto 0;
    background-color: #ffffff;
    color: #252525;
    font-size: 0.9rem;
  }
</style>
