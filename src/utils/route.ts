import { goto } from '$app/navigation'

const returnHome = () => {
  goto('/dashboard')
}

const openDocumentViewer = (filepath: string) => {
  const urlParams = `filepath=${encodeURIComponent(filepath)}`
  goto(`/document-viewer?${urlParams}`)
}

export { returnHome, openDocumentViewer }
