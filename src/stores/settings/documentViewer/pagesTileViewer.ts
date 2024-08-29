import { writable } from 'svelte/store'
import { write, load } from '../../../utils/settings'

let scale = writable<number>()
let pageNumVisible = writable<boolean>()
let fixPagesPerRow = writable<boolean>()
let pagesPerRow = writable<number>()

const { subscribe: subscribeScale } = scale
const { subscribe: subscribePageNumVisible } = pageNumVisible
const { subscribe: subscribeFixPagesPerRow } = fixPagesPerRow
const { subscribe: subscribePagesPerRow } = pagesPerRow

const setScale = (value: number) => {
  write('scale', value, scale)
}

const loadScale = (defaultValue: number): Promise<number> => {
  return load('scale', scale.set, defaultValue)
}

const setPageNumVisible = (value: boolean) => {
  write('pageNumVisible', value, pageNumVisible)
}

const loadPageNumVisible = (defaultValue: boolean): Promise<boolean> => {
  return load('pageNumVisible', pageNumVisible.set, defaultValue)
}

const setFixPagesPerRow = (value: boolean) => {
  write('fixPagesPerRow', value, fixPagesPerRow)
}

const loadFixPagesPerRow = (defaultValue: boolean): Promise<boolean> => {
  return load('fixPagesPerRow', fixPagesPerRow.set, defaultValue)
}

const setPagesPerRow = (value: number) => {
  write('pagesPerRow', value, pagesPerRow)
}

const loadPagesPerRow = (defaultValue: number): Promise<number> => {
  return load('pagesPerRow', pagesPerRow.set, defaultValue)
}

export {
  subscribeScale,
  setScale,
  loadScale,
  subscribePageNumVisible,
  setPageNumVisible,
  loadPageNumVisible,
  subscribeFixPagesPerRow,
  setFixPagesPerRow,
  loadFixPagesPerRow,
  subscribePagesPerRow,
  setPagesPerRow,
  loadPagesPerRow,
}
