<script lang="ts">
  import { subscribeLoading } from '../stores/layouts/loader'

  let loading: boolean = false

  $: {
    subscribeLoading((value) => (loading = value))
  }
</script>

<div class="loader" class:loading>
  <div class="circle"></div>
  <div class="circle"></div>
  <div class="circle"></div>
</div>

<style>
  .loader {
    position: fixed;
    left: calc((100vw - 3.2rem) / 2);
    top: calc((100vh - 3.2rem) / 2);
    width: 6.4rem;
    height: 6.4rem;
    display: none;
    justify-content: center;
    align-items: center;
    opacity: 0.33;
    transition: opacity 0.8s ease;
    z-index: 30000;
  }
  .loader.loading {
    display: flex;
  }

  @keyframes bounce {
    0%,
    100% {
      transform: translateY(0);
    }
    50% {
      transform: translateY(-20px);
    }
  }
  .circle {
    width: 20px;
    height: 20px;
    margin: 0 5px;
    border-radius: 50%;
    background-color: #2aabb7;
    animation: bounce 1.5s infinite ease-in-out;
  }
  .circle:nth-child(2) {
    width: 30px;
    height: 30px;
    animation-delay: 0.2s;
  }
  .circle:nth-child(3) {
    width: 40px;
    height: 40px;
    animation-delay: 0.4s;
  }
</style>
