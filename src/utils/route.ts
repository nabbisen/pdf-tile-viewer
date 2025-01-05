import { goto } from '$app/navigation'

const returnHome = () => {
  goto('/dashboard')
}

const openDocumentViewer = (filepath: string) => {
  goto(`/document-viewer/${encodeURIComponent(filepath)}`)
}

export { returnHome, openDocumentViewer }
