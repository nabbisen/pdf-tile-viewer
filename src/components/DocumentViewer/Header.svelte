<script lang="ts">
  import { createEventDispatcher } from 'svelte'
  import Search from './Search.svelte'
  import { filename } from '../../utils/file'
  import { returnHome } from '../../utils/route'
  import type { SearchResult } from './@types'

  export let filepath: string = ''

  const dispatch = createEventDispatcher()

  const handleSearch = (e: CustomEvent<SearchResult>) => {
    const searchResult = e.detail
    dispatch('search', searchResult)
  }
</script>

<header>
  <h2>{filename(filepath)}</h2>

  <nav>
    <Search {filepath} on:search={handleSearch} />

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
    cursor: pointer;
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
    font-size: 0.8rem;
    color: #878787;
  }
</style>
