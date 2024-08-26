import { writable } from 'svelte/store'
import type { ToastContent } from '../../types/toast'

const DEFAULT_DURATION_MILLISECONDS: number = 5000

const _toastContents = writable<ToastContent[]>([])
const { subscribe: subscribeToast } = _toastContents

const infoToast = (messages: string, durationMilliseconds?: number) => {
  show({
    messages,
    type: 'info',
    durationMilliseconds: durationMilliseconds ?? DEFAULT_DURATION_MILLISECONDS,
  })
}

const successToast = (messages: string, durationMilliseconds?: number) => {
  show({
    messages,
    type: 'success',
    durationMilliseconds: durationMilliseconds ?? DEFAULT_DURATION_MILLISECONDS,
  })
}

const errorToast = (messages: string, durationMilliseconds?: number) => {
  show({
    messages,
    type: 'error',
    durationMilliseconds: durationMilliseconds ?? DEFAULT_DURATION_MILLISECONDS,
  })
}

const show = (content: ToastContent) => {
  _toastContents.update((current) => {
    const ret = current
    ret.push(content)
    return ret
  })
  setTimeout(hide, content.durationMilliseconds)
}

const hide = () => {
  _toastContents.update((current) => {
    const ret = current
    ret.shift()
    return ret
  })
}

export { infoToast, successToast, errorToast, subscribeToast }
