import { writable, type Writable } from 'svelte/store'
import { invoke } from '@tauri-apps/api/core'
import type { ReadByKeyResponse } from '../../types/stores/documentViewerSettings'

let scale = writable<number>()
let pageNumVisible = writable<boolean>()
let fixPagesPerRow = writable<boolean>()
let pagesPerRow = writable<number>()

const { subscribe: subscribeScale } = scale
const { subscribe: subscribePageNumVisible } = pageNumVisible
const { subscribe: subscribeFixPagesPerRow } = fixPagesPerRow
const { subscribe: subscribePagesPerRow } = pagesPerRow

const setScale = (value: number) => {
  _write('scale', value, scale)
}

const loadScale = (defaultValue: number): Promise<number> => {
  return _load('scale', scale.set, defaultValue)
}

const setPageNumVisible = (value: boolean) => {
  _write('pageNumVisible', value, pageNumVisible)
}

const loadPageNumVisible = (defaultValue: boolean): Promise<boolean> => {
  return _load('pageNumVisible', pageNumVisible.set, defaultValue)
}

const setFixPagesPerRow = (value: boolean) => {
  _write('fixPagesPerRow', value, fixPagesPerRow)
}

const loadFixPagesPerRow = (defaultValue: boolean): Promise<boolean> => {
  return _load('fixPagesPerRow', fixPagesPerRow.set, defaultValue)
}

const setPagesPerRow = (value: number) => {
  _write('pagesPerRow', value, pagesPerRow)
}

const loadPagesPerRow = (defaultValue: number): Promise<number> => {
  return _load('pagesPerRow', pagesPerRow.set, defaultValue)
}

function _write<T>(key: string, value: any, writable: Writable<T>) {
  writable.set(value)
  invoke('settings_write_by_key', { key: key, value: value })
}

function _read(key: string): Promise<unknown> {
  return invoke('settings_read_by_key', { key: key }).then((res) => {
    const _res = res as ReadByKeyResponse
    if (!_res.file_exists || !_res.key_exists) return undefined
    return _res.value
  })
}

function _load<T>(key: string, set: Function, defaultValue: T): Promise<T> {
  return _read(key).then((res: unknown) => {
    const value = res === undefined ? defaultValue : (res as T)
    set(value)
    return value
  })
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
