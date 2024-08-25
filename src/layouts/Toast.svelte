<script lang="ts">
  import { subscribeToast } from '../stores/toast'
  import type { ToastContent } from '../types/toast'

  let contents: ToastContent[] = []
  $: {
    subscribeToast((value) => (contents = [...value]))
  }
</script>

<div class="toasts">
  {#each contents as content}
    <div class={`toast ${content.type}`}>{content.messages}</div>
  {/each}
</div>

<style>
  @keyframes toast-fade-in {
    from {
      opacity: 0;
      bottom: -2rem;
    }
    to {
      opacity: 1;
      bottom: 0.8rem;
    }
  }
  .toasts {
    position: fixed;
    left: 1rem;
    bottom: 0.4rem;
    width: calc(100% - 3.6rem);
    display: flex;
    flex-direction: column-reverse;
    gap: 0.4rem;
  }
  .toast {
    width: 100%;
    padding: 0.5rem 0.8rem;
    border-radius: 0.04rem;
    opacity: 0;
    animation: toast-fade-in 1.5s ease forwards;
  }
  .toast.info {
    background-color: #636363;
    color: #ffffff;
  }
  .toast.success {
    background-color: #2aabb7;
    color: #ffffff;
  }
  .toast.error {
    background-color: #d20f63;
    color: #ffffff;
  }
</style>
