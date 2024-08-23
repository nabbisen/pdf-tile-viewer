<script lang="ts">
  import { subscribeToast, type ToastContent } from '../stores/toast';

  let contents: ToastContent[] = []
  $: {
    subscribeToast(value => contents = [...value])
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
      opacity: 0.0;
      bottom: -2.0rem;
    }
    to {
      opacity: 1.0;
      bottom: 0.8rem;
    }
  }
  .toasts {
    position: fixed;
    left: 1.0rem;
    bottom: 0.4rem;
    width: calc(100% - 3.6rem);
    display: flex;
    flex-direction: column-reverse;
    gap: 0.4rem;
  }
  .toast {
    width: 100%;
    padding: 0.5rem 0.8rem;
    background-color: #252525;
    color: #ffffff;
    border-radius: 0.04rem;
    opacity: 0.0;
    animation: toast-fade-in 1.5s ease forwards;
  }
  .toast.error {
    background-color: #d20f63;
  }
</style>