<script lang="ts">
  import { subscribeLoadedHistory } from '../../stores/pages/loadedHistory'
  import type { LoadedHistoryItem } from '../../types/pages/loadedHistory'
  import { openDocumentViewer } from '../../utils/route'

  let loadedHistory: LoadedHistoryItem[] = []

  $: {
    subscribeLoadedHistory((value) => (loadedHistory = [...value]))
  }

  function historyItemOnClick(filepath: string) {
    openDocumentViewer(filepath)
  }

  function displayLoadingHistory(): LoadedHistoryItem[] {
    return loadedHistory.sort((a, b) => b.timestamp.getTime() - a.timestamp.getTime())
  }
</script>

{#if 0 < loadedHistory.length}
  <h3>History to re-open</h3>
  <ul class="loaded-history">
    {#each displayLoadingHistory() as x}
      <li>
        <time>{x.timestamp.toTimeString().split(' ')[0]}</time>
        <button on:click={() => historyItemOnClick(x.filepath)}>
          <h4>{x.filename}</h4>
          <div class="filepath">{x.filepath}</div>
        </button>
      </li>
    {/each}
  </ul>
{/if}

<style>
  h3 {
    color: #878787;
    text-align: center;
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
