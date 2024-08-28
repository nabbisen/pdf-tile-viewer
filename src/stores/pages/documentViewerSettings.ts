import { writable } from 'svelte/store'
import { invoke } from '@tauri-apps/api/core'
import type { ReadByKeyResponse } from '../../types/stores/documentViewerSettings'

let pageNumVisible = writable<boolean>()

const { subscribe: subscribePageNumVisible } = pageNumVisible

const setPageNumVisible = (value: boolean) => {
  pageNumVisible.set(value)
  invoke('settings_write_by_key', { key: 'pageNumVisible', value: value })
}

const getPageNumVisible = async (): Promise<unknown> => {
  return invoke('settings_read_by_key', { key: 'pageNumVisible' }).then((res) => {
    const _res = res as ReadByKeyResponse
    if (!_res.file_exists || !_res.key_exists) return undefined
    return _res.value
  })
}

export { subscribePageNumVisible, setPageNumVisible, getPageNumVisible }
