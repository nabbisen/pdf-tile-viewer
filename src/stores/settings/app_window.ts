import { writable } from 'svelte/store'
import { write } from '../../utils/settings'

let windowWidth = writable<number>()
let windowHeight = writable<number>()

const setWindowWidth = (value: number) => {
  write('windowWidth', value, windowWidth)
}

const setWindowHeight = (value: number) => {
  write('windowHeight', value, windowHeight)
}

export { setWindowWidth, setWindowHeight }
