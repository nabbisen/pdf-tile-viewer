<script lang="ts">
  import { subscribeLoading } from '../stores/components/loader'

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
      transform: translateY(-0.6rem);
    }
  }
  .circle {
    width: 0.6rem;
    height: 0.6rem;
    margin: 0 0.15rem;
    border-radius: 50%;
    background-color: #2aabb7;
    animation: bounce 1.5s infinite ease-in-out;
  }
  .circle:nth-child(2) {
    width: 0.9rem;
    height: 0.9rem;
    animation-delay: 0.2s;
  }
  .circle:nth-child(3) {
    width: 1.2rem;
    height: 1.2rem;
    animation-delay: 0.4s;
  }
</style>
