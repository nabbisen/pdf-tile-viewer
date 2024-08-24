import { writable } from 'svelte/store'

const DEFAULT_DURATION_MILLISECONDS: number = 5000

type ToastType = 'info' | 'success' | 'error'
interface ToastContent {
  messages: string
  type: ToastType
  durationMilliseconds: number
}

const storedContents = writable<ToastContent[]>([])
const { subscribe: subscribeToast } = storedContents

function infoToast(messages: string, durationMilliseconds?: number) {
  show({
    messages,
    type: 'info',
    durationMilliseconds: durationMilliseconds ?? DEFAULT_DURATION_MILLISECONDS,
  })
}

function successToast(messages: string, durationMilliseconds?: number) {
  show({
    messages,
    type: 'success',
    durationMilliseconds: durationMilliseconds ?? DEFAULT_DURATION_MILLISECONDS,
  })
}

function errorToast(messages: string, durationMilliseconds?: number) {
  show({
    messages,
    type: 'error',
    durationMilliseconds: durationMilliseconds ?? DEFAULT_DURATION_MILLISECONDS,
  })
}

function show(content: ToastContent) {
  storedContents.update((current) => {
    const ret = current
    ret.push(content)
    return ret
  })
  setTimeout(hide, content.durationMilliseconds)
}

function hide() {
  storedContents.update((current) => {
    const ret = current
    ret.shift()
    return ret
  })
}

export { infoToast, successToast, errorToast, subscribeToast, type ToastType, type ToastContent }
