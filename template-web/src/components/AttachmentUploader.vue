<template>
  <div class="attachment-uploader">
    <!-- 上传入口（list 模式：按钮 / 拖拽区） -->
    <template v-if="showUpload && !readonly">
      <a-upload
        v-if="uploadAllowed && listType === 'list' && !showDragger"
        :before-upload="beforeUpload"
        :custom-request="doUpload"
        :show-upload-list="false"
        :accept="accept"
        :disabled="disabled || reachMaxCount"
        :multiple="multiple"
      >
        <slot name="uploadTrigger">
          <a-button :disabled="disabled || reachMaxCount">
            <UploadOutlined />
            {{ buttonText }}
          </a-button>
        </slot>
      </a-upload>
      <a-upload-dragger
        v-else-if="uploadAllowed && listType === 'list' && showDragger"
        :before-upload="beforeUpload"
        :custom-request="doUpload"
        :show-upload-list="false"
        :accept="accept"
        :disabled="disabled || reachMaxCount"
        :multiple="multiple"
        class="mt-2"
      >
        <p class="ant-upload-drag-icon">
          <InboxOutlined />
        </p>
        <p class="ant-upload-text">
          点击或拖拽文件到此区域上传
        </p>
        <p class="ant-upload-hint">
          {{ effectiveHint }}
        </p>
      </a-upload-dragger>
    </template>

    <a-spin :spinning="loading">
      <!-- list 列表模式 -->
      <template v-if="listType === 'list'">
        <div
          v-if="files.length > 0"
          class="attachment-list"
        >
          <div
            v-for="file in files"
            :key="file.id"
            class="attachment-item"
          >
            <img
              v-if="isImage(file.mimeType) && urlMap[file.id]"
              :src="urlMap[file.id]"
              class="attachment-thumb"
              @click="preview(file)"
            />
            <div
              v-else
              class="attachment-file-icon"
            >
              <FileOutlined />
            </div>
            <div class="attachment-info">
              <a-tooltip :title="file.originalName">
                <span class="attachment-name">{{ file.originalName }}</span>
              </a-tooltip>
              <span class="attachment-size">{{ formatSize(file.fileSize) }}</span>
            </div>
            <div class="attachment-actions">
              <a @click.prevent="preview(file)">预览</a>
              <a-divider type="vertical" />
              <a @click.prevent="download(file)">下载</a>
              <template v-if="!readonly">
                <a-divider type="vertical" />
                <a-popconfirm
                  title="确定删除?"
                  @confirm="remove(file)"
                >
                  <a style="color: red">删除</a>
                </a-popconfirm>
              </template>
            </div>
          </div>
        </div>

        <!-- 延迟上传模式下已选择的待传文件 -->
        <div
          v-if="deferUpload && pending.length > 0"
          class="attachment-list"
        >
          <div
            v-for="(f, i) in pending"
            :key="'p' + i"
            class="attachment-item"
          >
            <div class="attachment-file-icon">
              <FileOutlined />
            </div>
            <div class="attachment-info">
              <a-tooltip :title="f.name">
                <span class="attachment-name">{{ f.name }}</span>
              </a-tooltip>
              <span class="attachment-size">{{ formatSize(f.size) }}</span>
            </div>
            <div class="attachment-actions">
              <a @click.prevent="removePending(i)">移除</a>
            </div>
          </div>
        </div>

        <a-empty
          v-else-if="readonly && showEmpty && files.length === 0 && pending.length === 0"
          :description="emptyText"
          style="padding: 24px 0"
        />
      </template>

      <!-- picture-card 卡片网格模式（批量展示） -->
      <template v-else>
        <div
          v-if="files.length > 0 || pending.length > 0 || (uploadAllowed && showUpload && !readonly)"
          class="attachment-grid"
        >
          <!-- 图片：统一进预览组，点击可放大并左右切换 -->
          <a-image-preview-group v-if="imageFiles.length > 0">
            <div
              v-for="file in imageFiles"
              :key="file.id"
              class="attachment-card"
            >
              <div class="attachment-card-body">
                <span
                  v-if="reorderable && !readonly && file.id === mainFileId"
                  class="attachment-card-main"
                >主图</span>
                <a-image
                  :src="urlMap[file.id]"
                  :width="104"
                  :height="104"
                  style="object-fit: cover; display: block"
                />
                <div class="attachment-card-mask">
                  <a-tooltip
                    v-if="reorderable && !readonly && canMoveLeft(file)"
                    title="左移"
                  >
                    <LeftOutlined @click.stop="moveImage(file, -1)" />
                  </a-tooltip>
                  <a-tooltip
                    v-if="reorderable && !readonly && canMoveRight(file)"
                    title="右移"
                  >
                    <RightOutlined @click.stop="moveImage(file, 1)" />
                  </a-tooltip>
                  <a-tooltip title="下载">
                    <DownloadOutlined @click="download(file)" />
                  </a-tooltip>
                  <a-tooltip
                    v-if="!readonly"
                    title="删除"
                  >
                    <a-popconfirm
                      title="确定删除?"
                      @confirm="remove(file)"
                    >
                      <DeleteOutlined />
                    </a-popconfirm>
                  </a-tooltip>
                </div>
              </div>
              <div
                class="attachment-card-name"
                :title="file.originalName"
              >
                {{ file.originalName }}
              </div>
            </div>
          </a-image-preview-group>

          <!-- 非图片（视频/音频/文档）：点击弹窗预览 -->
          <div
            v-for="file in nonImageFiles"
            :key="file.id"
            class="attachment-card"
          >
            <div
              class="attachment-card-body"
              @click="preview(file)"
            >
              <div class="attachment-card-file">
                <FileOutlined />
              </div>
              <div class="attachment-card-mask">
                <a-tooltip title="预览">
                  <EyeOutlined @click.stop="preview(file)" />
                </a-tooltip>
                <a-tooltip title="下载">
                  <DownloadOutlined @click.stop="download(file)" />
                </a-tooltip>
                <a-tooltip
                  v-if="!readonly"
                  title="删除"
                >
                  <a-popconfirm
                    title="确定删除?"
                    @confirm="remove(file)"
                  >
                    <DeleteOutlined />
                  </a-popconfirm>
                </a-tooltip>
              </div>
            </div>
            <div
              class="attachment-card-name"
              :title="file.originalName"
            >
              {{ file.originalName }}
            </div>
          </div>

          <!-- 延迟上传模式下已选择的待传文件 -->
          <div
            v-for="(f, i) in pending"
            :key="'p' + i"
            class="attachment-card"
          >
            <div class="attachment-card-body">
              <span
                v-if="reorderable && !readonly && i === pendingMainIndex"
                class="attachment-card-main"
              >主图</span>
              <div class="attachment-card-file">
                <FileOutlined />
              </div>
              <div class="attachment-card-mask">
                <a-tooltip
                  v-if="reorderable && !readonly && i > 0"
                  title="左移"
                >
                  <LeftOutlined @click.stop="movePending(i, -1)" />
                </a-tooltip>
                <a-tooltip
                  v-if="reorderable && !readonly && i < pending.length - 1"
                  title="右移"
                >
                  <RightOutlined @click.stop="movePending(i, 1)" />
                </a-tooltip>
                <a-tooltip title="移除">
                  <a-popconfirm
                    title="确定移除?"
                    @confirm="removePending(i)"
                  >
                    <DeleteOutlined />
                  </a-popconfirm>
                </a-tooltip>
              </div>
            </div>
            <div
              class="attachment-card-name"
              :title="f.name"
            >
              {{ f.name }}
            </div>
          </div>

          <div
            v-if="uploadAllowed && showUpload && !readonly && !reachMaxCount"
            class="attachment-card attachment-card-upload"
          >
            <a-upload
              :before-upload="beforeUpload"
              :custom-request="doUpload"
              :show-upload-list="false"
              :accept="accept"
              :disabled="disabled"
              :multiple="multiple"
            >
              <div class="attachment-card-upload-inner">
                <PlusOutlined />
                <div>上传</div>
              </div>
            </a-upload>
          </div>
        </div>
        <a-empty
          v-else-if="readonly && showEmpty"
          :description="emptyText"
          style="padding: 24px 0"
        />
      </template>
    </a-spin>

    <!-- 预览弹窗 -->
    <a-modal
      v-model:open="previewVisible"
      :title="previewing?.originalName || '附件预览'"
      :footer="null"
      width="800px"
      @cancel="closePreview"
    >
      <template v-if="previewing">
        <template v-if="isImage(previewing.mimeType)">
          <a-image
            :src="urlMap[previewing.id]"
            style="width: 100%"
          />
        </template>
        <template v-else-if="isVideo(previewing.mimeType)">
          <video
            :src="urlMap[previewing.id]"
            controls
            style="width: 100%; max-height: 500px"
          />
        </template>
        <template v-else-if="isAudio(previewing.mimeType)">
          <audio
            :src="urlMap[previewing.id]"
            controls
            style="width: 100%"
          />
        </template>
        <template v-else>
          <a-result title="暂不支持预览">
            <template #extra>
              <a-button
                type="primary"
                @click="download(previewing)"
              >
                下载文件
              </a-button>
            </template>
          </a-result>
        </template>
      </template>
    </a-modal>
  </div>
</template>

<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, reactive, ref, watch } from 'vue'
import { message } from 'ant-design-vue'
import {
  DeleteOutlined,
  DownloadOutlined,
  EyeOutlined,
  FileOutlined,
  InboxOutlined,
  LeftOutlined,
  PlusOutlined,
  RightOutlined,
  UploadOutlined,
} from '@ant-design/icons-vue'
import { attachmentApi, acquireImageUrl, releaseImageUrl, type AttachmentAdapter, type AttachmentVo, type UploadParams } from '@/api/attachment'
import { useUserStore } from '@/stores/user'

interface Props {
  /** 附件分类（如 purchase / ai / system） */
  category: string
  /** 业务类型（如 product / sample / report / factory） */
  bizType: string
  /** 关联业务 ID；变化时自动重新加载，未传时不自动加载 */
  bizId?: string
  /** 文件类型过滤（如 image/*、.pdf,.doc） */
  accept?: string
  /** 是否允许多选 */
  multiple?: boolean
  /** 最多文件数（0 表示不限制；含待传文件计数） */
  maxCount?: number
  /** 最多非视频附件数（0 表示不限制；如选品主图+细节图共 8 张） */
  maxImages?: number
  /** 最多视频数（0 表示不限制；如选品视频 1 个） */
  maxVideos?: number
  /** 单文件大小上限（MB） */
  maxSize?: number
  /** 禁用上传 */
  disabled?: boolean
  /** 只读模式：隐藏上传与删除，仅批量展示 + 预览 + 下载 */
  readonly?: boolean
  /** 是否显示上传入口 */
  showUpload?: boolean
  /** 是否显示拖拽上传区（仅 list 模式） */
  showDragger?: boolean
  /** 上传按钮文案 */
  buttonText?: string
  /** 拖拽区提示文案 */
  hintText?: string
  /** 展示形态：list 列表 / picture-card 卡片网格（批量展示） */
  listType?: 'list' | 'picture-card'
  /**
   * 是否允许调整图片位置（卡片 hover 出现左移/右移）。
   * 调整后自动重算 sort，并将第一个非视频图片标记为主图（main）、其余为细节图（detail）。
   * 需后端附件元信息更新能力（默认附件中心接口）。
   */
  reorderable?: boolean
  /** 图片类型标记：字符串则所有上传透传该值；
   * 函数则按「当前文件 + 已上传附件」动态计算（如首个图片标记主图），视频可返回 undefined。
   */
  imageType?: string | ((file: File, existing: AttachmentVo[]) => string | undefined)
  /** 备注（透传后端附件记录） */
  remark?: string
  /** 上传入口权限码（如 'attachment:upload'），不传则不校验；需与后端 sa_check_permission 对应 */
  uploadPermission?: string | string[]
  /**
   * 延迟上传：选择文件仅进入待传队列（expose.pending），
   * 由调用方在合适时机（如业务对象创建成功后）调用 uploadFiles() 统一上传。
   * 适用于「先建业务对象、再绑定附件」的表单场景。
   */
  deferUpload?: boolean
  /** 挂载及 bizId 变化时是否自动加载已有附件 */
  autoLoad?: boolean
  /** 只读且无附件时是否显示空态 */
  showEmpty?: boolean
  /** 空态文案 */
  emptyText?: string
  /**
   * 附件接口适配器：默认走管理端附件中心（需 attachment:* 权限）。
   * 门户端等无该权限点的场景注入自定义实现后即可复用本组件。
   */
  adapter?: AttachmentAdapter
}

const props = withDefaults(defineProps<Props>(), {
  accept: '*',
  multiple: true,
  maxCount: 0,
  maxImages: 0,
  maxVideos: 0,
  maxSize: 50,
  disabled: false,
  readonly: false,
  showUpload: true,
  showDragger: false,
  buttonText: '上传附件',
  hintText: '支持常见格式，单文件最大 50MB，支持多文件上传',
  listType: 'list',
  imageType: '',
  uploadPermission: '',
  deferUpload: false,
  autoLoad: true,
  showEmpty: true,
  emptyText: '暂无附件',
  reorderable: false,
})

const emit = defineEmits<{
  (e: 'uploaded', file: AttachmentVo): void
  (e: 'uploadedBatch', files: AttachmentVo[]): void
  (e: 'removed', fileId: string): void
  (e: 'change', files: AttachmentVo[]): void
}>()

/** 附件列表（按 sort 升序） */
const files = ref<AttachmentVo[]>([])
/** 延迟上传模式下已选择、尚未上传的文件 */
const pending = ref<File[]>([])
/** 附件 id → 展示 URL（直读端点需带 token，走共享缓存 fetch 生成 blob/签名 URL，卸载时统一归还引用） */
const urlMap = reactive<Record<string, string>>({})
/** 本组件持有的共享缓存引用（附件 id 集合），删除/清空/卸载时与 acquireImageUrl 成对 release */
const heldIds = new Set<string>()
const loading = ref(false)
/** 在途上传并发数（>0 表示有上传进行中） */
const uploadingCount = ref(0)
const uploading = computed(() => uploadingCount.value > 0)
const previewVisible = ref(false)
const previewing = ref<AttachmentVo | null>(null)

const userStore = useUserStore()

/** 非视频附件数（已上传 + 待传） */
const imageCount = computed(
  () =>
    files.value.filter((f) => !isVideo(f.mimeType)).length +
    pending.value.filter((f) => !isVideo(f.type)).length,
)
/** 视频数（已上传 + 待传） */
const videoCount = computed(
  () =>
    files.value.filter((f) => isVideo(f.mimeType)).length +
    pending.value.filter((f) => isVideo(f.type)).length,
)

/** 总数量/图片数/视频数均达上限时隐藏上传入口（任一类型仍可加则保留入口） */
const reachMaxCount = computed(() => {
  if (props.maxCount > 0 && files.value.length + pending.value.length >= props.maxCount) return true
  const imageFull = props.maxImages > 0 && imageCount.value >= props.maxImages
  const videoFull = props.maxVideos > 0 && videoCount.value >= props.maxVideos
  return imageFull && videoFull
})

/** 提示文案（叠加数量限制说明） */
const effectiveHint = computed(() => {
  const limits: string[] = []
  if (props.maxImages > 0) limits.push(`最多 ${props.maxImages} 张图片/附件`)
  if (props.maxVideos > 0) limits.push(`最多 ${props.maxVideos} 个视频`)
  return limits.length ? `${props.hintText}（${limits.join('，')}）` : props.hintText
})

/** picture-card 模式：图片进预览组（需已加载 blob URL），其余（视频/音频/文档/未加载图）走弹窗预览 */
const imageFiles = computed(() => files.value.filter((f) => isImage(f.mimeType) && urlMap[f.id]))
const nonImageFiles = computed(() => files.value.filter((f) => !isImage(f.mimeType) || !urlMap[f.id]))

/** 主图：按 sort 排序后第一个非视频图片（未加载 URL 时不显示角标，但排序/主图标记仍以其为准） */
const mainFileId = computed(() => files.value.find((f) => isImage(f.mimeType))?.id)
/** 待传队列中第一个非视频文件的索引（延迟上传模式下用于显示主图角标） */
const pendingMainIndex = computed(() => pending.value.findIndex((f) => !isVideo(f.type)))

function fileIndex(id: string) {
  return files.value.findIndex((f) => f.id === id)
}

function canMoveLeft(file: AttachmentVo) {
  const i = fileIndex(file.id)
  return i > 0
}

function canMoveRight(file: AttachmentVo) {
  const i = fileIndex(file.id)
  return i >= 0 && i < files.value.length - 1
}

/**
 * 重算全部附件排序与主图标记并持久化变更：
 * - sort 重排为 1..n；
 * - 第一个非视频图片标记为 main，其余图片标记为 detail（视频保持原标记）；
 * 仅对实际发生变化的附件调用更新接口，失败时回滚。
 */
async function persistMeta() {
  const prev = files.value.map((f) => ({ sort: f.sort, imageType: f.imageType }))
  let mainSet = false
  files.value.forEach((f, i) => {
    f.sort = i + 1
    if (isImage(f.mimeType)) {
      f.imageType = mainSet ? 'detail' : 'main'
      mainSet = true
    }
  })
  const changed = files.value.filter((f, i) => f.sort !== prev[i].sort || f.imageType !== prev[i].imageType)
  if (!changed.length) return
  try {
    await Promise.all(
      changed.map((f) =>
        (props.adapter?.updateMeta ?? attachmentApi.updateMeta)(f.id, { sort: f.sort, imageType: f.imageType || undefined }),
      ),
    )
    emit('change', [...files.value])
    message.success('位置已调整')
  } catch (e: any) {
    message.error('位置保存失败：' + (e.message || '请重试'))
    files.value = files.value.map((f, i) => ({ ...f, sort: prev[i].sort, imageType: prev[i].imageType }))
  }
}

/** 已上传图片左移 / 右移（与相邻卡片交换，交换后重算排序与主图并持久化） */
async function moveImage(file: AttachmentVo, dir: number) {
  const idx = fileIndex(file.id)
  const target = idx + dir
  if (idx < 0 || target < 0 || target >= files.value.length) return
  const arr = [...files.value]
  ;[arr[idx], arr[target]] = [arr[target], arr[idx]]
  files.value = arr
  await persistMeta()
}

/** 待传文件左移 / 右移：仅调整队列顺序（上传时按队列顺序计算 sort 与主图） */
function movePending(i: number, dir: number) {
  const target = i + dir
  if (target < 0 || target >= pending.value.length) return
  const arr = [...pending.value]
  ;[arr[i], arr[target]] = [arr[target], arr[i]]
  pending.value = arr
}

/** 上传入口权限门控：传了权限码则校验，admin 直通 */
const uploadAllowed = computed(() => {
  if (!props.uploadPermission) return true
  if (userStore.roles.includes('admin')) return true
  const list = Array.isArray(props.uploadPermission) ? props.uploadPermission : [props.uploadPermission]
  return list.some((p) => userStore.hasPermission(p))
})

function isImage(mime: string) {
  return String(mime || '').startsWith('image/')
}
function isVideo(mime: string) {
  return String(mime || '').startsWith('video/')
}
function isAudio(mime: string) {
  return String(mime || '').startsWith('audio/')
}
function formatSize(bytes: number) {
  if (bytes < 1024) return bytes + ' B'
  if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(1) + ' KB'
  return (bytes / (1024 * 1024)).toFixed(1) + ' MB'
}

/** 文件校验：返回错误文案，null 表示通过 */
function validate(file: File): string | null {
  if (props.maxSize > 0 && file.size > props.maxSize * 1024 * 1024) {
    return `文件大小超过 ${props.maxSize}MB 限制`
  }
  if (isVideo(file.type)) {
    if (props.maxVideos > 0 && videoCount.value >= props.maxVideos) {
      return `最多上传 ${props.maxVideos} 个视频`
    }
  } else if (props.maxImages > 0 && imageCount.value >= props.maxImages) {
    return `最多上传 ${props.maxImages} 张图片/附件`
  }
  if (props.maxCount > 0 && files.value.length + pending.value.length >= props.maxCount) {
    return `最多上传 ${props.maxCount} 个文件`
  }
  if (props.accept && props.accept !== '*' && props.accept !== '*/*') {
    const accepted = props.accept.split(',').map((a) => a.trim().toLowerCase())
    const pass = accepted.some((rule) => {
      if (rule.endsWith('/*')) return file.type.toLowerCase().startsWith(rule.slice(0, -1))
      if (rule.startsWith('.')) return file.name.toLowerCase().endsWith(rule)
      return file.type.toLowerCase() === rule
    })
    if (!pass) return `文件类型不允许（支持 ${props.accept}）`
  }
  if (props.maxCount > 0 && files.value.length + pending.value.length >= props.maxCount) {
    return `最多上传 ${props.maxCount} 个文件`
  }
  return null
}

function beforeUpload(file: File) {
  const err = validate(file)
  if (err) {
    message.error(`${file.name}：${err}`)
    return false
  }
  if (props.deferUpload) {
    pending.value.push(file)
    return false
  }
  return true
}

async function doUpload(options: { file: File; onSuccess: (body?: unknown) => void; onError: (err: Error) => void }) {
  try {
    const vo = await uploadOne(options.file)
    options.onSuccess(vo)
  } catch (e: any) {
    options.onError(e)
    const reason = e?.code === 'ECONNABORTED' || /timeout|超时/.test(e?.message || '') ? '上传超时，请重试' : e?.message || '上传失败'
    message.error(`${options.file.name} 上传失败：${reason}`)
  }
}

/** 单文件上传：入库 + 追加列表 + 触发事件 */
async function uploadOne(file: File): Promise<AttachmentVo> {
  const imageType =
    typeof props.imageType === 'function' ? props.imageType(file, files.value) : props.imageType || undefined
  const params: UploadParams = {
    category: props.category,
    bizType: props.bizType,
    bizId: props.bizId || undefined,
    imageType,
    remark: props.remark || undefined,
    sort: files.value.length + 1,
  }
  uploadingCount.value++
  try {
    const vo = await (props.adapter?.upload ?? attachmentApi.upload)(file, params)
    files.value = [...files.value, vo].sort((a, b) => a.sort - b.sort)
    if (isImage(vo.mimeType) || isVideo(vo.mimeType) || isAudio(vo.mimeType)) {
      void ensureUrl(vo)
    }
    emit('uploaded', vo)
    emit('change', [...files.value])
    return vo
  } finally {
    uploadingCount.value--
  }
}

/**
 * 编程式批量上传：
 * - 不传参数：上传全部待传文件（deferUpload 模式，成功后清空待传队列）并触发 uploadedBatch；
 * - 传 File[]：直接上传指定文件（UI 多选走 a-upload 逐文件触发 uploaded）。
 * 返回成功/失败明细，供调用方汇总展示。
 */
async function uploadFiles(
  fileList?: File[],
): Promise<{ success: AttachmentVo[]; failed: { fileName: string; error: string }[] }> {
  const usePending = fileList === undefined
  const source = usePending ? pending.value : fileList
  const success: AttachmentVo[] = []
  const failed: { fileName: string; error: string }[] = []
  for (const file of source) {
    const err = validate(file)
    if (err) {
      failed.push({ fileName: file.name, error: err })
      continue
    }
    try {
      success.push(await uploadOne(file))
    } catch (e: any) {
      failed.push({ fileName: file.name, error: e.message || '上传失败' })
    }
  }
  if (usePending) pending.value = []
  if (success.length) emit('uploadedBatch', success)
  return { success, failed }
}

function removePending(i: number) {
  pending.value.splice(i, 1)
}

function clearPending() {
  pending.value = []
}

/** 解析附件访问地址：适配器优先（可返回签名 URL），否则走内置带鉴权共享缓存生成 blob URL */
async function ensureUrl(vo: AttachmentVo) {
  if (urlMap[vo.id] || heldIds.has(vo.id)) return
  const url = props.adapter?.resolveUrl
    ? await props.adapter.resolveUrl(vo)
    : await acquireImageUrl(vo.id, props.bizType, props.bizId)
  if (url) {
    urlMap[vo.id] = url
    // 仅登记走共享缓存的引用（适配器返回的签名 URL 不归缓存管理）
    if (!props.adapter?.resolveUrl) heldIds.add(vo.id)
  }
}

/** 归还全部共享缓存引用并清空展示 URL；适配器的 URL 不在此列 */
function clearUrls() {
  for (const id of heldIds) releaseImageUrl(id, props.bizType, props.bizId)
  heldIds.clear()
  for (const k of Object.keys(urlMap)) delete urlMap[k]
}

/** 加载该业务对象下已有的附件（适配器无 list 能力时跳过） */
async function loadFiles() {
  if (!props.bizId) {
    files.value = []
    return
  }
  if (props.adapter && !props.adapter.list) {
    files.value = []
    return
  }
  loading.value = true
  try {
    if (props.adapter?.list) {
      files.value = (await props.adapter.list({
        category: props.category,
        bizType: props.bizType,
        bizId: props.bizId,
      })).sort((a, b) => a.sort - b.sort)
    } else {
      const res = await attachmentApi.list({
        category: props.category,
        bizType: props.bizType,
        bizId: props.bizId,
        page: 1,
        pageSize: 200,
      })
      files.value = [...res.items].sort((a, b) => a.sort - b.sort)
    }
    for (const f of files.value) {
      if (isImage(f.mimeType) || isVideo(f.mimeType) || isAudio(f.mimeType)) {
        await ensureUrl(f)
      }
    }
  } catch (e: any) {
    message.error(e.message || '附件加载失败')
  } finally {
    loading.value = false
  }
}

function preview(file: AttachmentVo) {
  previewing.value = file
  previewVisible.value = true
}

function closePreview() {
  previewVisible.value = false
  previewing.value = null
}

async function download(file: AttachmentVo) {
  try {
    if (props.adapter?.download) {
      await props.adapter.download(file)
    } else {
      await attachmentApi.downloadFile(file.id, file.originalName)
    }
  } catch (e: any) {
    message.error(e.message || '下载失败')
  }
}

async function remove(file: AttachmentVo) {
  try {
    if (props.adapter) {
      // 适配器未提供删除能力时仅从界面移除（不落库删除）
      if (props.adapter.remove) await props.adapter.remove(file.id)
    } else {
      await attachmentApi.delete(file.id)
    }
    files.value = files.value.filter((f) => f.id !== file.id)
    if (heldIds.has(file.id)) {
      releaseImageUrl(file.id, props.bizType, props.bizId)
      heldIds.delete(file.id)
    }
    delete urlMap[file.id]
    emit('removed', file.id)
    emit('change', [...files.value])
    message.success('删除成功')
    // 删除后重排：若删除的是主图/首位，自动把剩余第一个图片提升为主图
    if (props.reorderable) void persistMeta()
  } catch (e: any) {
    message.error(e.message || '删除失败')
  }
}

function clear() {
  files.value = []
  clearUrls()
  emit('change', [])
}

watch(
  () => props.bizId,
  () => {
    clearUrls()
    files.value = []
    if (props.autoLoad) void loadFiles()
  },
)

onMounted(() => {
  if (props.autoLoad) void loadFiles()
})

onBeforeUnmount(() => {
  clearUrls()
})

defineExpose({
  files,
  pending,
  loading,
  uploading,
  loadFiles,
  reload: loadFiles,
  uploadFiles,
  clearPending,
  clear,
  remove,
  preview,
  download,
})
</script>

<style scoped>
.attachment-uploader {
  width: 100%;
}

.mt-2 {
  margin-top: 8px;
}

.attachment-list {
  margin-top: 8px;
}

.attachment-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 8px;
  border: 1px solid #f0f0f0;
  border-radius: 6px;
  margin-bottom: 8px;
}

.attachment-thumb {
  width: 80px;
  height: 80px;
  object-fit: cover;
  border-radius: 4px;
  cursor: pointer;
  flex-shrink: 0;
}

.attachment-file-icon {
  width: 80px;
  height: 80px;
  flex-shrink: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  background: #f5f5f5;
  border-radius: 4px;
  font-size: 32px;
  color: #999;
}

.attachment-info {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.attachment-name {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-weight: 500;
}

.attachment-size {
  color: #999;
  font-size: 12px;
}

.attachment-actions {
  white-space: nowrap;
}

/* picture-card 卡片网格（批量展示） */
.attachment-grid {
  display: flex;
  flex-wrap: wrap;
  gap: 12px;
  margin-top: 8px;
}

.attachment-card {
  width: 104px;
}

.attachment-card-body {
  position: relative;
  width: 104px;
  height: 104px;
  border: 1px solid #f0f0f0;
  border-radius: 6px;
  overflow: hidden;
  background: #fafafa;
}

/* 主图角标（左上角，不拦截点击） */
.attachment-card-main {
  position: absolute;
  top: 4px;
  left: 4px;
  z-index: 2;
  padding: 0 6px;
  font-size: 12px;
  line-height: 18px;
  color: #fff;
  background: var(--ant-primary-color, #1677ff);
  border-radius: 4px;
  pointer-events: none;
}

.attachment-card-file {
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 32px;
  color: #999;
  cursor: pointer;
}

.attachment-card-mask {
  position: absolute;
  inset: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 14px;
  background: rgba(0, 0, 0, 0.45);
  opacity: 0;
  transition: opacity 0.2s;
  /* 蒙层不拦截点击：让点击穿透到下方图片（触发预览组并定位到该图），仅图标可交互 */
  pointer-events: none;
}

.attachment-card-body:hover .attachment-card-mask {
  opacity: 1;
}

.attachment-card-mask :deep(span) {
  color: #fff;
  font-size: 18px;
  cursor: pointer;
  pointer-events: auto;
}

.attachment-card-name {
  margin-top: 4px;
  font-size: 12px;
  color: #666;
  text-align: center;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.attachment-card-upload {
  display: flex;
}

.attachment-card-upload :deep(.ant-upload) {
  width: 100%;
  height: 100%;
}

.attachment-card-upload-inner {
  width: 104px;
  height: 104px;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 4px;
  border: 1px dashed #d9d9d9;
  border-radius: 6px;
  color: #999;
  font-size: 12px;
  cursor: pointer;
  transition: border-color 0.2s, color 0.2s;
  background: #fafafa;
}

.attachment-card-upload-inner:hover {
  border-color: var(--ant-primary-color, #1677ff);
  color: var(--ant-primary-color, #1677ff);
}
</style>
