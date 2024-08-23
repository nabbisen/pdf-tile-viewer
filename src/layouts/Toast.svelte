<script lang="ts">
  import { storedContent, type ToastContent } from '../stores/toast';

  let content: ToastContent | undefined
  $: {
    storedContent.subscribe(value => content = value)
  }
</script>

{#if content }
  <div class={`toast ${content.type}`}>{content.messages}</div>
{/if}

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
  .toast {
    position: fixed;
    left: 1.0rem;
    bottom: -2.0rem;
    width: calc(100% - 3.6rem);
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