<script lang="ts">
  import { onDestroy, onMount } from 'svelte'

  let isDragging: boolean = false
  let startX: number = 0
  let startY: number = 0
  let scrollLeft: number = 0
  let scrollBottom: number = 0

  onMount(dragMoveAddEventListener)

  onDestroy(dragMoveremoveEventListener)

  function dragMoveAddEventListener() {
    document.addEventListener('mousedown', windowOnMousedown)
    document.addEventListener('mousemove', windowOnMousemove)
    document.addEventListener('mouseup', windowOnMouseup)
    document.addEventListener('mouseleave', windowOnMouseleave)
  }

  function dragMoveremoveEventListener() {
    document.removeEventListener('mousedown', windowOnMousedown)
    document.removeEventListener('mousemove', windowOnMousemove)
    document.removeEventListener('mouseup', windowOnMouseup)
    document.removeEventListener('mouseleave', windowOnMouseleave)
  }

  function windowOnMousedown(e: MouseEvent) {
    if (!e.ctrlKey) return

    isDragging = true
    startX = e.pageX - window.scrollX
    startY = e.pageY - window.scrollY
    scrollLeft = window.scrollX
    scrollBottom = window.scrollY
    document.body.style.cursor = 'grabbing'
    e.preventDefault()
  }

  function windowOnMousemove(e: MouseEvent) {
    if (!isDragging) return

    const x = e.pageX - window.scrollX
    const y = e.pageY - window.scrollY
    const walkX = (x - startX) * 1.8
    const walkY = (y - startY) * 1.8
    window.scrollTo(scrollLeft - walkX, scrollBottom - walkY)
  }

  function windowOnMouseup() {
    isDragging = false
    document.body.style.cursor = 'default'
  }

  function windowOnMouseleave() {
    isDragging = false
    document.body.style.cursor = 'default'
  }
</script>
