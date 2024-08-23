import { writable } from 'svelte/store';

const TOAST_DURATION_MILLISECONDS: number = 10000

type ToastType = 'error'
interface ToastContent {
  messages: string,
  type: ToastType,
}

const storedContents = writable<ToastContent[]>([])
const { subscribe: subscribeToast } = storedContents

function errorToast(messages: string) {
  show({
    messages,
    type: 'error',
  })
}

function show(content: ToastContent) {
  storedContents.update(current => {
    const ret = current
    ret.push(content)
    return ret
  })
  setTimeout(hide, TOAST_DURATION_MILLISECONDS)
}

function hide() {
  storedContents.update(current => {
    const ret = current
    ret.shift()
    return ret
  })
}

export { subscribeToast, errorToast, type ToastType, type ToastContent }
