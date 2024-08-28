<script lang="ts">
  import { onMount, createEventDispatcher } from 'svelte'
  import {
    DEFAULT_SCALE,
    MIN_SCALE,
    MAX_SCALE,
    SCALE_UNIT,
    DEFAULT_PAGE_NUM_VISIBLE,
    DEFAULT_FIX_PAGES_PER_ROW,
    DEFAULT_PAGES_PER_ROW,
  } from './consts'
  import {
    loadScale,
    loadPageNumVisible,
    loadFixPagesPerRow,
    loadPagesPerRow,
    setScale,
    setPageNumVisible,
    setFixPagesPerRow,
    setPagesPerRow,
  } from '../../stores/pages/documentViewerSettings'
  import Tooltip from '../../components/Tooltip.svelte'

  export let numPages: number

  const dispatch = createEventDispatcher()

  let scale: number
  let pageNumVisible: boolean
  let fixPagesPerRow: boolean
  let pagesPerRow: number

  let scrollToPageNum: number | undefined

  onMount(loadSettings)

  function scaleOnChange() {
    setScale(scale)
  }

  function pageNumVisibleOnChange() {
    setPageNumVisible(pageNumVisible)
  }

  function fixPagesPerRowOnChange() {
    setFixPagesPerRow(fixPagesPerRow)
  }

  function pagesPerRowOnChange() {
    setPagesPerRow(pagesPerRow)
  }

  function scrollToPage() {
    dispatch('scrollToPage', scrollToPageNum)
  }

  function loadSettings() {
    loadScale(DEFAULT_SCALE).then((ret) => (scale = ret as number))
    loadPageNumVisible(DEFAULT_PAGE_NUM_VISIBLE).then((ret) => (pageNumVisible = ret as boolean))
    loadFixPagesPerRow(DEFAULT_FIX_PAGES_PER_ROW).then((ret) => (fixPagesPerRow = ret as boolean))
    loadPagesPerRow(DEFAULT_PAGES_PER_ROW).then((ret) => (pagesPerRow = ret as number))
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
        on:change={scaleOnChange}
      />
    </div>
  </div>
  <div class="page-operations">
    <div class="page-nums">
      <div class="total"><strong>{numPages}</strong></div>
      <Tooltip messages="Number visible" position="left">
        <label>
          <input
            type="checkbox"
            class="toggle-page-num"
            bind:checked={pageNumVisible}
            on:change={pageNumVisibleOnChange}
          />
          <h4 aria-label="page-num-visible">🔢</h4>
          <span class="button auxiliary">{pageNumVisible ? 'On' : 'Off'}</span>
        </label>
      </Tooltip>
    </div>
    <div class="pages-per-row">
      <Tooltip messages="Pages per row" position="left">
        <h4 aria-label="pages-per-row">📖</h4>
        <label>
          <input type="checkbox" bind:checked={fixPagesPerRow} on:change={fixPagesPerRowOnChange} />
          <span class="button auxiliary">{fixPagesPerRow ? 'Fixed' : 'Auto-wrapped'}</span>
        </label>
      </Tooltip>
      {#if fixPagesPerRow}
        <span>at: </span>
        <input
          type="number"
          min="1"
          step="1"
          disabled={!fixPagesPerRow}
          bind:value={pagesPerRow}
          on:change={pagesPerRowOnChange}
        />
      {/if}
    </div>
    <div class="scroll-to-page">
      <h4 aria-label="jump">📃</h4>
      <span>p.</span>
      <input type="number" bind:value={scrollToPageNum} min="1" max={numPages} step="1" />
      <button class="auxiliary" on:click={scrollToPage}>Go</button>
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
    display: inline-block;
  }

  aside .scale-operations,
  aside .page-operations {
    display: flex;
    align-items: center;
    gap: 2.7rem;
  }
  aside .scale-operations > div,
  aside .page-operations > div {
    padding: 0.2rem 0;
    display: flex;
    flex-direction: row;
    align-items: center;
    gap: 0.3rem;
  }
  aside .page-operations button,
  aside .page-operations .button {
    padding: 0.2rem 0.5rem;
  }
  aside .page-nums .total::after {
    content: ' pages';
  }
  aside .page-nums label,
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

  aside label input[type='checkbox'] {
    display: none;
  }
</style>
