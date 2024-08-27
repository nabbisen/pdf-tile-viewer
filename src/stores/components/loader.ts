import { writable } from 'svelte/store'

const _loading = writable<boolean>(false)
const { subscribe: subscribeLoading } = _loading

const loaderStart = () => {
  _loading.set(true)
}

const loaderStop = () => {
  _loading.set(false)
}

export { loaderStart, loaderStop, subscribeLoading }
