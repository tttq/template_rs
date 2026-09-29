<template>
  <a-image
    v-if="src && kind === 'image'"
    :src="src"
    :width="56"
    :height="56"
    :preview="true"
    style="object-fit: cover; border-radius: 4px; display: block"
  />
  <video
    v-else-if="src && kind === 'video'"
    :src="src"
    width="56"
    height="56"
    style="object-fit: cover; border-radius: 4px; display: block"
  />
  <div
    v-else
    class="thumb-empty"
  >
    —
  </div>
</template>

<script setup lang="ts">
import { onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { acquireImageUrl, releaseImageUrl } from '@/api/attachment'

const props = withDefaults(
  defineProps<{
    /** 附件直读路径（形如 /api/system/attachment/{id}） */
    url?: string | null
    /** 业务类型（直读双因子校验） */
    bizType?: string
    /** 业务对象 id（直读双因子校验） */
    bizId?: string
    /**
     * 渲染形态：image 走 a-image（可点开大图），video 走 video。
     * 直读端点需 Authorization 头，两者都只能经 blob URL 渲染。
     */
    kind?: 'image' | 'video'
  }>(),
  { url: null, bizType: 'product', kind: 'image' },
)

const src = ref('')
let disposed = false
/** 当前持有的缓存引用（附件 id），url 变化或卸载时成对 release */
let holdingId = ''

function parseId(): string {
  return String(props.url || '').split('?')[0].split('/').pop() || ''
}

function load() {
  if (disposed) return
  const id = parseId()
  // 释放上一张图的持有引用；同一 id 未变化时直接跳过（避免重复 acquire）
  if (id !== holdingId) {
    if (holdingId) releaseImageUrl(holdingId, props.bizType, props.bizId)
    holdingId = ''
    src.value = ''
  }
  if (!id) return
  acquireImageUrl(id, props.bizType, props.bizId).then((u) => {
    if (disposed || id !== parseId()) return
    holdingId = id
    src.value = u || ''
  })
}

onMounted(load)
watch(() => props.url, load)
onBeforeUnmount(() => {
  disposed = true
  if (holdingId) releaseImageUrl(holdingId, props.bizType, props.bizId)
})
</script>

<style scoped>
.thumb-empty {
  width: 56px;
  height: 56px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: #fafafa;
  border: 1px solid #f0f0f0;
  border-radius: 4px;
  color: #bbb;
}
</style>
