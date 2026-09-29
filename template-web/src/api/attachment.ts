import request from './request'

/** 走 vite 代理的 /api 相对路径（开发 proxy 转发后端，生产由网关反代） */
const baseUrl = '/api'

export interface AttachmentVo {
  id: string
  url: string | null
  category: string
  bizType: string
  bizId: string | null
  imageType: string | null
  sort: number
  caption: string | null
  originalName: string
  mimeType: string
  fileSize: number
  storagePath: string
  uploaderId: string | null
  status: string
  remark: string | null
  createTime: string
  createBy: string | null
}

export interface AttachmentQuery {
  category?: string
  bizType?: string
  bizId?: string
  page?: number
  pageSize?: number
}

export interface PageResult<T> {
  items: T[]
  total: number
  page: number
  pageSize: number
  totalPages: number
}

export interface UploadParams {
  category: string
  bizType: string
  bizId?: string
  imageType?: string
  sort?: number
  caption?: string
  remark?: string
}

/** 附件元信息更新（排序 / 主图标记） */
export interface AttachmentMetaUpdate {
  sort?: number
  imageType?: string
}

export interface BatchUploadResult {
  success: AttachmentVo[]
  failed: { fileName: string; error: string }[]
}

/**
 * 附件接口适配器：默认走管理端附件中心（/api/system/attachment*，需 attachment:* 权限）。
 * 门户端等无该权限点的场景（只有 /api/portal/files 上传 + 签名 URL 直读）可注入自定义实现，
 * 从而复用通用附件组件。
 */
export interface AttachmentAdapter {
  /** 上传（必填） */
  upload: (file: File, params: UploadParams) => Promise<AttachmentVo>
  /** 按业务列出已有附件；无列表端点时可省略（组件跳过自动加载） */
  list?: (params: { category: string; bizType: string; bizId?: string }) => Promise<AttachmentVo[]>
  /** 删除附件；省略时仅从界面移除（不落库删除） */
  remove?: (id: string) => Promise<void>
  /**
   * 解析可直接用于 src/href 的访问地址。
   * 省略时组件走内置直读（带 token fetch 生成 blob URL）。
   */
  resolveUrl?: (file: AttachmentVo) => Promise<string> | string
  /** 下载；省略时走内置带鉴权下载 */
  download?: (file: AttachmentVo) => Promise<void> | void
  /** 更新附件元信息（排序 / 主图标记）；省略时走内置附件中心更新 */
  updateMeta?: (id: string, data: AttachmentMetaUpdate) => Promise<void>
}

export const attachmentApi = {
  upload: (file: File, params: UploadParams) => {
    const query = new URLSearchParams()
    query.append('category', params.category)
    query.append('bizType', params.bizType)
    if (params.bizId) query.append('bizId', params.bizId)
    if (params.imageType) query.append('imageType', params.imageType)
    if (params.sort != null) query.append('sort', String(params.sort))
    if (params.caption) query.append('caption', params.caption)
    if (params.remark) query.append('remark', params.remark)
    query.append('fileName', file.name)
    return request.post<AttachmentVo>(`/system/attachment?${query.toString()}`, file, {
      headers: { 'Content-Type': file.type || 'application/octet-stream' },
      // 上传原始字节：大文件 / 首次写盘可能较慢，放宽超时避免 30s 全局超时误中断
      timeout: 120000,
    })
  },

  uploadBatch: async (files: File[], params: UploadParams): Promise<BatchUploadResult> => {
    const success: AttachmentVo[] = []
    const failed: { fileName: string; error: string }[] = []
    for (const file of files) {
      try {
        const result = await attachmentApi.upload(file, params)
        success.push(result)
      } catch (e: any) {
        failed.push({ fileName: file.name, error: e.message || '上传失败' })
      }
    }
    return { success, failed }
  },

  list: (query?: AttachmentQuery) =>
    request.get<PageResult<AttachmentVo>>('/system/attachment', { params: query }),

  listByBiz: (category: string, bizType: string, bizId?: string) =>
    request.get<AttachmentVo[]>('/system/attachment', { params: { category, bizType, bizId, pageSize: 200 } }),

  delete: (id: string) => request.delete(`/system/attachment/${id}`),

  batchDelete: (ids: string[]) => request.delete('/system/attachment/batch', { data: ids }),

  /** 更新附件元信息（排序 / 主图标记），供图片位置调整使用 */
  updateMeta: (id: string, data: AttachmentMetaUpdate) =>
    request.put(`/system/attachment/${id}/meta`, data),

  downloadFile: async (id: string, fileName: string) => {
    const token = localStorage.getItem('token')
    const tenantId = localStorage.getItem('tenantId')
    const resp = await fetch(`${baseUrl}/system/attachment/${id}`, {
      headers: {
        ...(token ? { Authorization: `Bearer ${token}` } : {}),
        ...(tenantId ? { 'x-tenant-id': tenantId } : {}),
      },
    })
    if (!resp.ok) throw new Error('下载失败')
    const blob = await resp.blob()
    const url = URL.createObjectURL(blob)
    const a = document.createElement('a')
    a.href = url
    a.download = fileName
    a.click()
    URL.revokeObjectURL(url)
  },
}

// ==================== 附件直读图片共享缓存 ====================
// 背景：直读端点需要 Authorization 头，无法直接 <img src>，只能 fetch 转 blob URL；
// 且带 Authorization 的 fetch 默认不进 HTTP 磁盘缓存。列表页每行一个缩略图都下载
// 整张原图，页面反复进出会疯狂重复下载 + blob 内存翻倍 → 页面卡死。
// 方案：模块级 Map 缓存 blob URL + 在途请求合并（同附件只发一次 fetch）+
// 引用计数（最后一个持有者 release 才 revoke）+ LRU 上限，跨组件/跨页面复用已下载图片。

const IMAGE_CACHE_MAX = 120 // 缓存的 blob URL 上限，超出后按 LRU 淘汰（仅淘汰无持有者的）

interface ImageCacheEntry {
  url: string
  refs: number // 持有引用数（组件 acquire / release 增减）
  touched: number // 最近使用时间戳（LRU）
}

const imageUrlCache = new Map<string, ImageCacheEntry>()
const imageInflight = new Map<string, Promise<string | null>>() // 在途请求合并：同附件并发只发一次

function imageCacheKey(id: string, bizType?: string, bizId?: string) {
  return `${id}|${bizType || ''}|${bizId || ''}`
}

/** 直读附件字节并生成 blob URL；失败抛错（调用方决定是否重试） */
async function fetchImageBlob(id: string, bizType?: string, bizId?: string): Promise<string> {
  const query = new URLSearchParams()
  if (bizType) query.append('bizType', bizType)
  if (bizId) query.append('bizId', bizId)
  const qs = query.toString()
  const token = localStorage.getItem('token')
  const tenantId = localStorage.getItem('tenantId')
  const resp = await fetch(`${baseUrl}/system/attachment/${id}${qs ? '?' + qs : ''}`, {
    headers: {
      ...(token ? { Authorization: `Bearer ${token}` } : {}),
      ...(tenantId ? { 'x-tenant-id': tenantId } : {}),
    },
  })
  if (!resp.ok) throw new Error('加载失败')
  const blob = await resp.blob()
  return URL.createObjectURL(blob)
}

/**
 * 取得（并持有）附件图片的共享 blob URL。
 * - 缓存命中：直接返回已下载的 URL（refs +1）;
 * - 未命中：发起一次 fetch（同附件并发请求合并，只发一次），成功后入缓存（refs=1）;
 * - 加载失败：返回 null 且**不做失败缓存**，下次调用可重试。
 * 必须与 releaseImageUrl 成对调用（如组件 onBeforeUnmount 释放），最后一个持有者释放时回收 blob。
 */
export function acquireImageUrl(id: string, bizType?: string, bizId?: string): Promise<string | null> {
  const key = imageCacheKey(id, bizType, bizId)
  const hit = imageUrlCache.get(key)
  if (hit) {
    hit.refs++
    hit.touched = Date.now()
    return Promise.resolve(hit.url)
  }

  const pending = imageInflight.get(key)
  if (pending) {
    // 在途请求：复用同一份下载结果，fetch 方完成入缓存后补登记持有引用
    void pending.then((url) => {
      if (url) {
        const entry = imageUrlCache.get(key)
        if (entry) entry.refs++
      }
    })
    return pending
  }

  const promise = (async (): Promise<string | null> => {
    try {
      const url = await fetchImageBlob(id, bizType, bizId)
      imageUrlCache.set(key, { url, refs: 1, touched: Date.now() })
      evictIfNeeded()
      return url
    } catch {
      return null
    } finally {
      imageInflight.delete(key)
    }
  })()

  imageInflight.set(key, promise)
  return promise
}

/** 释放持有引用（与 acquireImageUrl 成对）；最后一个持有者释放时 revoke 并移出缓存 */
export function releaseImageUrl(id: string, bizType?: string, bizId?: string) {
  const key = imageCacheKey(id, bizType, bizId)
  const entry = imageUrlCache.get(key)
  if (!entry) return
  entry.refs = Math.max(0, entry.refs - 1)
  if (entry.refs === 0) {
    imageUrlCache.delete(key)
    URL.revokeObjectURL(entry.url)
  }
}

/** LRU 淘汰：容量超限时优先淘汰最久未使用且已无人持有的条目 */
function evictIfNeeded() {
  if (imageUrlCache.size <= IMAGE_CACHE_MAX) return
  const candidates = [...imageUrlCache.entries()]
    .filter(([, e]) => e.refs === 0)
    .sort((a, b) => a[1].touched - b[1].touched)
  while (imageUrlCache.size > IMAGE_CACHE_MAX && candidates.length) {
    const [key] = candidates.shift()!
    const entry = imageUrlCache.get(key)
    if (entry) {
      imageUrlCache.delete(key)
      URL.revokeObjectURL(entry.url)
    }
  }
}