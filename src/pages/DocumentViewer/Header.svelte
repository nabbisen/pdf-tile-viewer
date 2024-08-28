<script lang="ts">
  import { invoke } from '@tauri-apps/api/core'
  import Search from './Search.svelte'
  import { filename } from '../../utils/file'
  import { returnHome } from '../../utils/route'
  import { errorToast } from '../../stores/components/toast'
  import Tooltip from '../../components/Tooltip.svelte'
  import { setZenMode, subscribeZenMode } from '../../stores/pages/documentViewer'

  export let filepath: string | undefined

  let zenMode: boolean = false

  $: {
    subscribeZenMode((value) => (zenMode = value))
  }

  function filenameClick() {
    invoke('file_manager_open', { filepath: filepath }).catch((e: string) => {
      errorToast(e)
    })
  }

  function scrollToTop() {
    window.scrollTo({
      top: 0,
      behavior: 'smooth',
    })
  }
  function scrollToRight() {
    window.scrollTo({
      left: document.body.scrollWidth,
      behavior: 'smooth',
    })
  }
  function scrollToBottom() {
    window.scrollTo({
      top: document.body.scrollHeight,
      behavior: 'smooth',
    })
  }
  function scrollToLeft() {
    window.scrollTo({
      left: 0,
      behavior: 'smooth',
    })
  }
</script>

<header>
  {#if !zenMode}
    <h2>
      <Tooltip messages="Click to open file manager" position="right">
        <button on:click={filenameClick}>{filename(filepath || '')}</button>
      </Tooltip>
    </h2>
  {/if}

  <nav>
    {#if !zenMode}
      <Search {filepath} />
    {/if}

    <div class="footer">
      <div class="buttons">
        {#if !zenMode}
          <Tooltip messages="Double click" position="left">
            <button class="home" on:dblclick={returnHome}>
              <h1>Home</h1>
            </button>
          </Tooltip>
        {/if}
        <button
          class="zen-mode auxiliary"
          on:click={() => {
            setZenMode(!zenMode)
          }}>Zen{zenMode ? 'On' : 'Off'}</button
        >
      </div>
      {#if !zenMode}
        <div class="scroll-to">
          <button class="top" on:click={scrollToTop}>↑</button>
          <button class="right" on:click={scrollToRight}>→</button>
          <button class="bottom" on:click={scrollToBottom}>↓</button>
          <button class="left" on:click={scrollToLeft}>←</button>
        </div>
      {/if}
    </div>
  </nav>
</header>

<style>
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
    display: flex;
    flex-direction: column;
    justify-content: center;
    align-items: center;
    z-index: 20000;
  }

  nav .footer {
    display: flex;
    flex-direction: row-reverse;
  }

  nav .buttons {
    display: flex;
    flex-direction: column;
  }

  nav button.home,
  nav button.zen-mode {
    width: 3.2rem;
  }

  nav button.home {
    background-color: #efefef;
    font-size: 0.6rem;
    box-shadow: none;
    border: none;
  }
  nav button.home:hover {
    opacity: 0.87;
  }
  nav button.home:active {
    box-shadow: unset;
    transform: unset;
  }

  nav button.zen-mode {
    padding: 0.2rem 0;
    margin-top: 0.4rem;
    font-size: 0.7rem;
  }

  nav .scroll-to {
    position: relative;
    margin-right: 0.7rem;
  }
  nav .scroll-to button {
    position: absolute;
    color: #252525;
    font-size: 0.8rem;
    font-weight: bold;
  }
  nav .scroll-to button:hover {
    color: #bbbbbb;
  }
  nav .scroll-to button:disabled {
    visibility: hidden;
  }
  nav .scroll-to .top {
    right: 0.8rem;
    bottom: 1.5rem;
  }
  nav .scroll-to .right {
    right: 0;
    bottom: 0.75rem;
  }
  nav .scroll-to .bottom {
    right: 0.8rem;
    bottom: 0;
  }
  nav .scroll-to .left {
    right: 1.6rem;
    bottom: 0.75rem;
  }
</style>
