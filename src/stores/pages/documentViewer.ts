import { writable } from 'svelte/store'

let filepath = writable<string | undefined>()
let buffer = writable<ArrayBuffer | undefined>()
let matchedPageIndexes = writable<number[]>([])
let confirmedSearchTerm = writable<string | undefined>()
let displayMatchedPages = writable<string | undefined>()
let zoomedPageIndex = writable<number | undefined>()
let zenMode = writable<boolean>(false)

const { subscribe: subscribeFilepath } = filepath
const { subscribe: subscribeBuffer } = buffer
const { subscribe: subscribeMatchedPageIndexes } = matchedPageIndexes
const { subscribe: subscribeConfirmedSearchTerm } = confirmedSearchTerm
const { subscribe: subscribeDisplayMatchedPages } = displayMatchedPages
const { subscribe: subscribeZoomedPageIndex } = zoomedPageIndex
const { subscribe: subscribeZenMode } = zenMode

const setFilepath = (value: string) => {
  filepath.set(value)
}

const setBuffer = (value: ArrayBuffer) => {
  buffer.set(value)
}

const setMatchedPageIndexes = (value: number[]) => {
  matchedPageIndexes.set(value)
}

const setConfirmedSearchTerm = (value: string) => {
  confirmedSearchTerm.set(value)
}

const setDisplayMatchedPages = (value: string | undefined) => {
  displayMatchedPages.set(value)
}

const setZoomedPageIndex = (value: number | undefined) => {
  zoomedPageIndex.set(value)
}

const setZenMode = (value: boolean) => {
  zenMode.set(value)
}

const reset = () => {
  filepath.set(undefined)
  buffer.set(undefined)
  matchedPageIndexes.set([])
  confirmedSearchTerm.set(undefined)
  displayMatchedPages.set(undefined)
  zoomedPageIndex.set(undefined)
  zenMode.set(false)
}

const reload = (currentFilepath: string) => {
  reset()
  filepath.set(currentFilepath)
}

export {
  subscribeFilepath,
  setFilepath,
  subscribeBuffer,
  setBuffer,
  subscribeMatchedPageIndexes,
  setMatchedPageIndexes,
  subscribeConfirmedSearchTerm,
  setConfirmedSearchTerm,
  subscribeDisplayMatchedPages,
  setDisplayMatchedPages,
  subscribeZoomedPageIndex,
  setZoomedPageIndex,
  subscribeZenMode,
  setZenMode,
  reset,
  reload,
}
