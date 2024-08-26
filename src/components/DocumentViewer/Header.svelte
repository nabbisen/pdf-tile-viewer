<script lang="ts">
  import { invoke } from '@tauri-apps/api/core'
  import Search from './Search.svelte'
  import { filename } from '../../utils/file'
  import { returnHome } from '../../utils/route'
  import { errorToast } from '../../stores/layouts/toast'
  import Tooltip from '../@common/Tooltip.svelte'

  export let filepath: string | undefined

  function filenameClick() {
    invoke('open_file_manager', { filepath: filepath }).catch((e: string) => {
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
  <h2>
    <Tooltip messages="Click to open file manager" position="right">
      <button on:click={filenameClick}>{filename(filepath || '')}</button>
    </Tooltip>
  </h2>

  <nav>
    <Search {filepath} />

    <div class="footer">
      <div class="logo">
        <Tooltip messages="Double click" position="left">
          <button on:dblclick={returnHome}>
            <h1>Home</h1>
          </button>
        </Tooltip>
      </div>
      <div class="scroll-to">
        <button class="top" on:click={scrollToTop}>↑</button>
        <button class="right" on:click={scrollToRight}>→</button>
        <button class="bottom" on:click={scrollToBottom}>↓</button>
        <button class="left" on:click={scrollToLeft}>←</button>
      </div>
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

  nav .footer .logo button {
    width: 3.2rem;
    height: 2.7rem;
    background-color: #efefef;
    font-size: 0.6rem;
    box-shadow: none;
    border: none;
  }
  nav .footer .logo button:hover {
    opacity: 0.87;
  }

  nav .footer .scroll-to {
    position: relative;
    margin-right: 0.7rem;
  }
  nav .footer .scroll-to button {
    position: absolute;
    color: #252525;
    font-size: 0.8rem;
    font-weight: bold;
  }
  nav .footer .scroll-to button:hover {
    color: #bbbbbb;
  }
  nav .footer .scroll-to button:disabled {
    visibility: hidden;
  }
  nav .footer .scroll-to .top {
    right: 0.8rem;
    bottom: 1.5rem;
  }
  nav .footer .scroll-to .right {
    right: 0;
    bottom: 0.75rem;
  }
  nav .footer .scroll-to .bottom {
    right: 0.8rem;
    bottom: 0;
  }
  nav .footer .scroll-to .left {
    right: 1.6rem;
    bottom: 0.75rem;
  }
</style>
