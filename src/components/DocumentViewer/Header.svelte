<script lang="ts">
  import { invoke } from '@tauri-apps/api/core'
  import Search from './Search.svelte'
  import { filename } from '../../utils/file'
  import { returnHome } from '../../utils/route'
  import {
    subscribeConfirmedSearchTerm,
    subscribeDisplayMatchedPages,
  } from '../../stores/components/documentViewer'
  import { errorToast } from '../../stores/layouts/toast'

  export let filepath: string | undefined

  let confirmedSearchTerm: string | undefined
  let displayMatchedPages: string | undefined

  $: {
    subscribeConfirmedSearchTerm((value) => (confirmedSearchTerm = value))
  }

  $: {
    subscribeDisplayMatchedPages((value) => (displayMatchedPages = value))
  }

  function filenameClick() {
    invoke('open_file_manager', { filepath: filepath }).catch((e: string) => {
      errorToast(e)
    })
  }
</script>

<header>
  <h2>
    <button on:click={filenameClick}>{filename(filepath || '')}</button>
  </h2>

  <nav>
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
    <Search {filepath} />

    <div class="logo">
      <button on:click={returnHome}>
        <h1>Home</h1>
      </button>
    </div>
  </nav>
</header>

<style>
  header,
  h1 {
    font-size: 0.7rem;
  }

  h1 {
    color: #878787;
  }
  h1::after {
    content: '🏠';
    display: inline-block;
    padding: 0 0.7rem;
  }

  nav {
    position: fixed;
    right: 0.8rem;
    bottom: 0.5rem;
    width: 4.4em;
    display: flex;
    flex-direction: column;
    justify-content: center;
    align-items: center;
    z-index: 20000;
  }

  .logo button {
    background-color: #efefef;
    font-size: 0.6rem;
    box-shadow: none;
    border: none;
    text-align: center;
  }
  .logo button:hover {
    opacity: 0.87;
  }

  h2 {
    position: fixed;
    top: 0;
    right: 0.4rem;
    transform: rotate(90deg) translate(100%, 0);
    transform-origin: top right;
    white-space: nowrap;
  }
  h2 button {
    color: #5d9ae5;
    font-size: 0.8rem;
    font-weight: bold;
  }

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
    line-height: 1.4em;
    text-align: right;
  }
  .search-result .search-matched .no-matched {
    text-align: center;
  }
</style>
