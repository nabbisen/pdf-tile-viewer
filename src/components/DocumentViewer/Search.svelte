<script lang="ts">
  import { createEventDispatcher } from 'svelte'
  import { invoke } from '@tauri-apps/api/core'
  import { infoToast, successToast } from '../../stores/toast'
  import { handleInvokeError } from '../../utils/backend'
  import type { SearchResult } from './@types'

  export let filepath: string = ''

  const dispatch = createEventDispatcher()

  const SEARCH_TERM_MIN_LENGTH: number = 3
  const SEQUENTIAL_CONCAT: string = '~'
  let searchTerm: string = ''
  let isSearchFormVisible: boolean = false

  const searchPdf = async () => {
    let res: any
    invoke('search_pdf', {
      searchTerm: searchTerm,
      filepath: filepath,
    })
      .then((ret: unknown) => {
        res = ret
      })
      .catch((error: unknown) => {
        handleInvokeError(error)
        return
      })
    const buffer = res.buffer as ArrayBuffer

    const matchedPageIndexes = res.page_indexes as number[]
    if (0 < matchedPageIndexes.length) {
      successToast(`Matches: p.${displayMatchedPages(matchedPageIndexes)}`)
    } else {
      infoToast('No matches')
    }
    const confirmedSearchTerm = searchTerm

    const searchResult: SearchResult = {
      buffer,
      matchedPageIndexes,
      confirmedSearchTerm,
    }
    dispatch('search', searchResult)

    isSearchFormVisible = false
  }

  const displayMatchedPages = (pageIndexes: number[]): string => {
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
    return ret
  }

  const toggleSearchForm = () => {
    isSearchFormVisible = !isSearchFormVisible
  }
</script>

<div class="search">
  <button class="toggle" on:click={toggleSearchForm}>🔍</button>
  {#if isSearchFormVisible}
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
      <button class="close" on:click={toggleSearchForm}>Close</button>
    </form>
  {/if}
</div>

<style>
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
  .search form button.search:disabled {
    background-color: #eaeaea;
    color: #bbbbbb;
  }
  .search form button.close {
    width: fit-content;
    margin: 1.1rem auto 0;
    background-color: #ffffff;
    color: #252525;
    font-size: 0.9rem;
  }
</style>
