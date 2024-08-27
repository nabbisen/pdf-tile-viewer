<script lang="ts">
  import { createEventDispatcher } from 'svelte'

  export let MIN_SCALE: number
  export let MAX_SCALE: number
  export let SCALE_UNIT: number
  export let scale: number
  export let numPages: number
  export let pageNumVisible: boolean

  const dispatch = createEventDispatcher()

  let scrollToPageNum: number | undefined
  let fixPagesPerRow: boolean
  let pagesPerRow: number = 5

  function scaleChange() {
    dispatch('scaleChange', scale)
  }

  function togglePageNum() {
    dispatch('togglePageNum')
  }

  function scrollToPage() {
    dispatch('scrollToPage', scrollToPageNum)
  }

  function fixPagesPerRowChange() {
    dispatch('fixPagesPerRowChange', { fixPagesPerRow, pagesPerRow })
  }
</script>

<aside>
  <div class="scale-operations">
    <div class="scale">
      <h4>Scale</h4>
      <input
        type="range"
        min={MIN_SCALE}
        max={MAX_SCALE}
        step={SCALE_UNIT}
        bind:value={scale}
        on:change={scaleChange}
      />
    </div>
  </div>
  <div class="page-operations">
    <div class="page-nums">
      <div class="total"><strong>{numPages}</strong></div>
      <button class="toggle-page-num" on:click={togglePageNum}
        >🔢 <span class="button auxiliary">{pageNumVisible ? 'on' : 'off'}</span></button
      >
    </div>
    <div class="scroll-to-page">
      <h4>Jump</h4>
      <span>p.</span>
      <input type="number" bind:value={scrollToPageNum} min="1" max={numPages} step="1" />
      <button class="auxiliary" on:click={scrollToPage}>Go</button>
    </div>
    <div class="pages-per-row">
      <h4>Pages per row</h4>
      <span>is: </span>
      <button
        class="auxiliary"
        on:click={() => {
          fixPagesPerRow = !fixPagesPerRow
          fixPagesPerRowChange()
        }}
      >
        {fixPagesPerRow ? 'fixed' : 'auto-determined'}
      </button>
      <input type="checkbox" bind:checked={fixPagesPerRow} on:change={fixPagesPerRowChange} />
      {#if fixPagesPerRow}
        <span>at: </span>
        <input
          type="number"
          min="1"
          step="1"
          disabled={!fixPagesPerRow}
          bind:value={pagesPerRow}
          on:change={fixPagesPerRowChange}
        />
      {/if}
    </div>
  </div>
</aside>

<style>
  aside {
    position: fixed;
    bottom: 0.4rem;
    left: 0.7rem;
    z-index: 1;
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 0.1rem;
  }
  aside * {
    font-size: 0.7rem;
    color: #878787;
  }

  aside h4 {
    padding: 0;
    margin: 0;
  }

  aside .scale-operations,
  aside .scale-operations > div,
  aside .page-operations,
  aside .page-operations > div {
    display: flex;
    align-items: center;
    gap: 1.2rem;
  }
  aside .scale-operations > div,
  aside .page-operations > div {
    padding: 0.2rem 0;
    display: flex;
    flex-direction: row;
    align-items: center;
    gap: 0.3rem;
  }
  aside .page-nums .total::after {
    content: ' pages';
  }
  aside .page-nums button.toggle-page-num,
  aside input[type='number'] {
    font-size: 0.8rem;
  }

  aside .scale input[type='range'] {
    width: 5.7rem;
    margin: 0 0.6rem;
  }

  aside input[type='number'] {
    width: 3.6em;
    color: #444444;
    text-align: right;
  }
  aside .toggle-page-num span {
    padding-left: 0.3rem;
    padding-right: 0.3rem;
  }

  aside .pages-per-row button + input[type='checkbox'] {
    display: none;
  }
</style>
