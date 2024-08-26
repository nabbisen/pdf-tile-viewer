import { errorToast } from '../stores/layouts/toast'

const handleInvokeError = (error: unknown) => {
  const messages: string =
    typeof error === 'string'
      ? (error as string)
      : error instanceof Error
        ? error.message
        : 'unknown error'
  errorToast(messages, 10000)
}

export { handleInvokeError }
