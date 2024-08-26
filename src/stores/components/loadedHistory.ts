import { writable } from 'svelte/store'
import type { LoadedHistoryItem } from '../../types/loadedHistory'
import { filename } from '../../utils/file'

let stored = writable<LoadedHistoryItem[]>([])
const { subscribe: subscribeLoadedHistory } = stored

const pushToLoadedHistory = (filepath: string) => {
  stored.update((current) => {
    const ret = current

    const existingItemIndex = ret.findIndex((x) => x.filepath === filepath)
    if (existingItemIndex !== -1) {
      ret.splice(existingItemIndex, 1)
    }
    const loadedHistoryItem = <LoadedHistoryItem>{
      filename: filename(filepath),
      filepath: filepath,
      timestamp: new Date(),
    }
    ret.push(loadedHistoryItem)

    return ret
  })
}

export { pushToLoadedHistory, subscribeLoadedHistory }
