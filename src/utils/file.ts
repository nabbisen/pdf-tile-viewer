const filename = (filepath: string): string => {
  let ret: string
  const slashSplit = filepath.split('/')
  if (2 <= slashSplit.length) {
    ret = slashSplit[slashSplit.length - 1]
  } else {
    const backslashSplit = filepath.split('\\')
    ret = backslashSplit[backslashSplit.length - 1]
  }
  return ret
}

export { filename }
