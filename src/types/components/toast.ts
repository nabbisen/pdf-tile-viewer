export type ToastType = 'info' | 'success' | 'error'

export interface ToastContent {
  messages: string
  type: ToastType
  durationMilliseconds: number
}
