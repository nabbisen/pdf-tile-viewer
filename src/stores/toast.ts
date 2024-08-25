import { writable } from 'svelte/store'
import type {ToastContent} from '../types/toast'

const DEFAULT_DURATION_MILLISECONDS: number = 5000

const stored = writable<ToastContent[]>([])
const { subscribe: subscribeToast } = stored

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
  stored.update((current) => {
    const ret = current
    ret.push(content)
    return ret
  })
  setTimeout(hide, content.durationMilliseconds)
}

function hide() {
  stored.update((current) => {
    const ret = current
    ret.shift()
    return ret
  })
}

export { infoToast, successToast, errorToast, subscribeToast }
