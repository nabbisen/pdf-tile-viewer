import { writable } from 'svelte/store'

let filepath = writable<string | undefined>()
let buffer = writable<ArrayBuffer | undefined>()
let matchedPageIndexes = writable<number[]>([])
let confirmedSearchTerm = writable<string | undefined>()
let displayMatchedPages = writable<string | undefined>()

const { subscribe: subscribeFilepath } = filepath
const { subscribe: subscribeBuffer } = buffer
const { subscribe: subscribeMatchedPageIndexes } = matchedPageIndexes
const { subscribe: subscribeConfirmedSearchTerm } = confirmedSearchTerm
const { subscribe: subscribeDisplayMatchedPages } = displayMatchedPages

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

const reset = () => {
  filepath.set(undefined)
  buffer.set(undefined)
  matchedPageIndexes.set([])
  confirmedSearchTerm.set(undefined)
  displayMatchedPages.set(undefined)
}

const reload = (currentFilepath: string) => {
  reset()
  filepath.set(currentFilepath)
}

export {
  setFilepath,
  subscribeFilepath,
  setBuffer,
  subscribeBuffer,
  setMatchedPageIndexes,
  subscribeMatchedPageIndexes,
  setConfirmedSearchTerm,
  subscribeConfirmedSearchTerm,
  setDisplayMatchedPages,
  subscribeDisplayMatchedPages,
  reset,
  reload,
}
