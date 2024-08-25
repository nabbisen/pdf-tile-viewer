import { goto } from '$app/navigation'

const openDocumentViewer = (filepath: string) => {
  const urlParams = `filepath=${encodeURIComponent(filepath)}`
  goto(`/document-viewer?${urlParams}`)
}

export { openDocumentViewer }
