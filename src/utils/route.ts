import { goto } from '$app/navigation'
import { setFilepath, reset } from '../stores/components/documentViewer'

const returnHome = () => {
  goto('/dashboard')
}

const openDocumentViewer = (filepath: string) => {
  setFilepath(filepath)
  goto('/document-viewer')
}

export { returnHome, openDocumentViewer }
