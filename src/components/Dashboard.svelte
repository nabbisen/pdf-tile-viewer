<script lang="ts">
  import { createEventDispatcher } from 'svelte'
  import { open } from '@tauri-apps/plugin-dialog'
  import { subscribeLoadedHistory } from '../stores/loadedHistory'
  import type { LoadedHistoryItem } from '../types/loadedHistory'

  const dispatch = createEventDispatcher()

  let loadedHistory: LoadedHistoryItem[] = []
  $: {
    subscribeLoadedHistory((value) => (loadedHistory = [...value]))
  }

  const selectFile = async () => {
    const fileResponse = await open({
      multiple: false, // todo
      directory: false,
      filters: [
        { name: 'PDF', extensions: ['pdf'] },
        { name: 'All files', extensions: ['*'] },
      ],
    })
    if (fileResponse) {
      const filepath = fileResponse.path
      dispatch('fileSelect', filepath)
    }
  }
</script>

<header>
  <div class="logo">
    <h1>PDF Tile Viewer</h1>
  </div>
</header>

<div class={`placeholder ${0 < loadedHistory.length ? 'shrink' : ''}`}>
  <h2>Drop PDF file</h2>
  <p>
    <b>Number of pages in a row</b> is dynamically calculated with <b>app window size</b>
    and <b>zoom scale</b> modified with Ctrl key + mouse wheel.
  </p>
  <button on:click={selectFile}>Choose file</button>
</div>

<h3>History to re-open</h3>
<ul class="loaded-history">
  {#each loadedHistory as x}
    <li>
      <time>{x.timestamp.toTimeString().split(' ')[0]}</time>
      <button
        on:click={() => {
          dispatch('fileSelect', x.filepath)
        }}
      >
        <h4>{x.filename}</h4>
        <div class="filepath">{x.filepath}</div>
      </button>
    </li>
  {/each}
</ul>

<style>
  header {
    position: relative;
    z-index: 10000;
  }

  h1 {
    font-size: 1.8rem;
    padding: 0;
    margin: 0;
    color: #555555;
  }
  h1::after {
    content: '🦈';
    display: inline-block;
    padding: 0 0.7rem;
  }

  .placeholder {
    padding: 16vh 0;
    margin: 2rem;
    text-align: center;
    border: 1px dotted black;
  }
  .placeholder.shrink {
    padding-top: 1.1rem;
    padding-bottom: 1.1rem;
  }
  .placeholder p {
    max-width: 16em;
    margin: 1.1rem auto;
    text-align: left;
    font-size: 0.8rem;
    font-family: serif;
    color: #727272;
  }
  .placeholder button {
    padding: 0.2rem 0.6rem;
    background-color: #2aabb7;
    color: #ffffff;
    border: 0.03rem #4ccbd6 solid;
    font-size: 1.5rem;
    border-radius: 0.2rem;
  }
  .placeholder button:hover {
    opacity: 0.87;
  }

  h3 {
    display: none;
    color: #878787;
    text-align: center;
  }
  .has-history ~ h3 {
    display: block;
  }

  .loaded-history {
    max-width: 40rem;
    margin: 0.5rem auto;
  }
  .loaded-history {
    list-style-type: none;
  }
  .loaded-history time {
    margin-right: 0.8rem;
    color: #878787;
    font-size: 0.8rem;
  }
  .loaded-history button {
    position: relative;
    max-width: calc(100% - 9rem);
    background: none;
    border: none;
    font-size: 1.4rem;
    text-align: left;
    cursor: pointer;
  }
  .loaded-history h4 {
    padding: 0;
    margin: 0;
  }
  .loaded-history button:hover h4 {
    color: #2aabb7;
  }
  .loaded-history h4::before {
    width: 0;
  }
  .loaded-history button:hover h4::before {
    content: '';
    position: absolute;
    left: 0;
    bottom: 0;
    width: 100%;
    height: 0.04rem;
    background-color: #4ccbd6;
    transition: width 1s ease;
  }
  .loaded-history .filepath {
    max-width: 100%;
    font-size: 0.8rem;
    white-space: nowrap;
    overflow-x: hidden;
    text-overflow: ellipsis;
    unicode-bidi: plaintext;
    direction: rtl;
  }
</style>
