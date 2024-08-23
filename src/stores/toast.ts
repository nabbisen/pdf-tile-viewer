import { writable } from 'svelte/store';

const TOAST_DURATION_MILLISECONDS: number = 10000

type ToastType = 'error'
interface ToastContent {
  messages: string,
  type: ToastType,
}

const storedContent = writable<ToastContent | undefined>()

function errorToast(messages: string) {
  storedContent.set({
    messages,
    type: 'error',
  })

  setTimeout(() => {
    hideToast()
  }, TOAST_DURATION_MILLISECONDS);
}

function hideToast() {
  storedContent.set(undefined)
}

export { storedContent, errorToast, type ToastType, type ToastContent }
