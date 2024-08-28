import { type Writable } from 'svelte/store'
import { invoke } from '@tauri-apps/api/core'
import type { ReadByKeyResponse } from '../types/stores/documentViewerSettings'

const write = <T>(key: string, value: any, writable: Writable<T>) => {
  writable.set(value)
  invoke('settings_write_by_key', { key: key, value: value })
}

const read = (key: string): Promise<unknown> => {
  return invoke('settings_read_by_key', { key: key }).then((res) => {
    const _res = res as ReadByKeyResponse
    if (!_res.file_exists || !_res.key_exists) return undefined
    return _res.value
  })
}

const load = <T>(key: string, set: Function, defaultValue: T): Promise<T> => {
  return read(key).then((res: unknown) => {
    const value = res === undefined ? defaultValue : (res as T)
    set(value)
    return value
  })
}

export { write, load }
