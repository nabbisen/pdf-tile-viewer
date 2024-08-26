function debounce<T extends (...args: any[]) => void>(
  callback: T,
  waitMilliseconds: number
): (...args: Parameters<T>) => void {
  let timerId: number | undefined
  return function (...args: Parameters<T>): void {
    if (timerId) {
      clearTimeout(timerId)
    }
    timerId = setTimeout(() => callback(...args), waitMilliseconds)
  }
}

export { debounce }
