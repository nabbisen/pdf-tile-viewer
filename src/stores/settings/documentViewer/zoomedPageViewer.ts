import { writable, type Writable } from 'svelte/store'
import { write, load } from '../../../utils/settings'

let ZoomViewBackgroundLocked = writable<boolean>()
let zoomViewScale = writable<number>()

const setZoomViewBackgroundLocked = (value: boolean) => {
  write('backgroundLocked', value, ZoomViewBackgroundLocked)
}

const loadZoomViewBackgroundLocked = (defaultValue: boolean): Promise<boolean> => {
  return load('scale', ZoomViewBackgroundLocked.set, defaultValue)
}

const setZoomViewScale = (value: number) => {
  write('zoomViewScale', value, zoomViewScale)
}

const loadZoomViewScale = (defaultValue: number): Promise<number> => {
  return load('zoomViewScale', zoomViewScale.set, defaultValue)
}

export {
  setZoomViewBackgroundLocked,
  loadZoomViewBackgroundLocked,
  setZoomViewScale,
  loadZoomViewScale,
}
