<script lang="ts">
  import Tooltip from '../../components/Tooltip.svelte'
  import { successToast, errorToast } from '../../stores/components/toast'

  const SHARE_THIS_APP_MESSAGES: string =
    'PDF Tile Viewer : https://github.com/nabbisen/pdf-tile-viewer/releases'

  async function shareThisAppOnClick() {
    try {
      await navigator.clipboard.writeText(SHARE_THIS_APP_MESSAGES)
      successToast('App info copied to clipboard.', 2100)
    } catch (err) {
      errorToast('Failed to copy to clipboard.', 1400)
    }
  }
</script>

<header>
  <div class="logo">
    <img src="/logo.png" alt="logo" />
    <h1><span>PDF</span> <span>Tile</span> <span>Viewer</span></h1>
  </div>
  <div class="share-this-app">
    <Tooltip messages="App info is copied to clipboard" position="right">
      <button on:click={shareThisAppOnClick}>
        <div>Share</div>
        <span>this app</span>
      </button>
    </Tooltip>
  </div>
</header>

<style>
  header {
    position: relative;
    z-index: 10000;
    display: flex;
  }

  @keyframes logo-bg {
    from {
      width: 4.4rem;
    }
    to {
      width: calc(100% - 3.6rem);
    }
  }
  .logo {
    position: relative;
    width: fit-content;
    display: flex;
  }
  .logo::before {
    content: '';
    position: absolute;
    width: 4.4rem;
    left: 4.1rem;
    bottom: 0.5rem;
    height: 0.8rem;
    background-color: #fff49132;
    animation: logo-bg 4.4s ease forwards;
  }
  img {
    height: 3.2rem;
    margin: 0.2rem 0.6rem;
  }
  img:hover {
    transform: translate(0.05rem, 0.05rem);
    transition: transform 0.2s ease;
  }
  h1 {
    font-size: 1.8rem;
    padding: 0;
    margin: 0 0 0 0.6rem;
    display: flex;
    align-items: center;
  }
  h1 span:not(:first-child) {
    margin-left: 0.3rem;
  }
  h1 span {
    color: #636363;
  }
  h1 span:nth-child(2) {
    color: #878787;
  }
  h1 span:nth-child(3) {
    color: #939393;
  }

  .share-this-app {
    margin-left: 1.5rem;
    margin-top: 0.3rem;
  }
  .share-this-app button {
    display: flex;
    flex-direction: column;
    align-items: center;
  }
  .share-this-app button div {
    width: 2.2rem;
    height: 2.2rem;
    background-color: hsl(185, 63%, 93%);
    color: hsl(185, 87%, 33%);
    border-radius: 50%;
    font-size: 0.7rem;
    display: flex;
    justify-content: center;
    align-items: center;
  }
  .share-this-app button span {
    color: #878787;
    font-size: 0.57rem;
  }
</style>
