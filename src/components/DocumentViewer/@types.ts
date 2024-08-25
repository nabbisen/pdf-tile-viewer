export interface SearchResult {
  pageBuffers: ArrayBuffer[],
  matchedPageIndexes: number[],
  confirmedSearchTerm: string
}
