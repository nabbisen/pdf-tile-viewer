import { invoke } from '@tauri-apps/api/core'
import {
  getDocument,
  type PDFDocumentProxy,
} from 'pdfjs-dist'

const getPageBuffers = async (filepath: string): Promise<ArrayBuffer[]> => {
  const res = (await invoke('read_pdf', { filepath: filepath })) as ArrayBuffer[]
  return res
}

const getDocumentProxy = async (buffer: ArrayBuffer): Promise<PDFDocumentProxy> => {
  const CMAP_URL = 'pdfjs-dist/cmaps/'
  const CMAP_PACKED = true

  const loadingTask = getDocument({
    data: buffer,
    url: undefined,
    cMapUrl: CMAP_URL,
    cMapPacked: CMAP_PACKED,
  })

  return loadingTask.promise
}

export { getPageBuffers, getDocumentProxy }