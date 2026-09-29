<template>
  <div>
  <a-card :bordered="false">
      <a-form
        layout="inline"
        :model="search"
        style="flex: 1; min-width: 520px; margin-bottom: 16px"
        @finish="fetchData"
      >
        <a-form-item label="附件分类">
          <a-select
            v-model:value="search.category"
            placeholder="请选择附件分类"
            allow-clear
            style="width: 140px"
            @change="fetchData"
          >
            <a-select-option value="purchase">
              采购附件
            </a-select-option>
            <a-select-option value="ai">
              AI附件
            </a-select-option>
            <a-select-option value="system">
              系统附件
            </a-select-option>
          </a-select>
        </a-form-item>
        <a-form-item label="业务类型">
          <a-input
            v-model:value="search.bizType"
            placeholder="请输入业务类型"
            allow-clear
            style="width: 140px"
            @press-enter="fetchData"
          />
        </a-form-item>
        <a-form-item>
          <a-space>
            <a-button
              type="primary"
              html-type="submit"
            >
              查询
            </a-button>
            <a-button @click="resetSearch">
              重置
            </a-button>
          </a-space>
        </a-form-item>
      </a-form>
  </a-card>

  <a-card
    ref="fitRef"
    :bordered="false"
    style="margin-top: 12px"
  >
    <template #extra>
      <a-space>
        <a-button
          v-permission="'attachment:upload'"
          type="primary"
          @click="openUploadModal"
        >
          <UploadOutlined />
          上传附件
        </a-button>
        <a-button
          danger
          :disabled="selectedRowKeys.length === 0"
          @click="handleBatchDelete"
        >
          批量删除
        </a-button>
      </a-space>
    </template>
    <a-table
      :columns="columns"
      :data-source="data"
      :loading="loading"
      :pagination="pagination"
      :scroll="{ x: 1100, y: fitY }"
      :row-selection="rowSelection"
      row-key="id"
      size="middle"
      @change="onTableChange"
    >
      <template #bodyCell="{ column, record, index }">
        <template v-if="column.key === 'index'">
          {{ (pagination.current - 1) * pagination.pageSize + index + 1 }}
        </template>
        <template v-if="column.key === 'category'">
          <a-tag :color="categoryColor(record.category)">
            {{ categoryLabel(record.category) }}
          </a-tag>
        </template>
        <template v-if="column.key === 'preview'">
          <template v-if="isImage(record.mimeType) || isVideo(record.mimeType)">
            <!-- 直读端点需 Authorization 头，必须经 AttachmentThumb 转 blob URL；
                 bizType/bizId 取记录自身的值，满足直读双因子校验 -->
            <AttachmentThumb
              :url="record.url"
              :biz-type="record.bizType"
              :biz-id="record.bizId || undefined"
              :kind="isVideo(record.mimeType) ? 'video' : 'image'"
            />
          </template>
          <template v-else>
            <div style="width: 60px; height: 60px; display: flex; align-items: center; justify-content: center; background: #f5f5f5; border-radius: 4px">
              <FileOutlined style="font-size: 24px; color: #999" />
            </div>
          </template>
        </template>
        <template v-if="column.key === 'fileSize'">
          {{ formatFileSize(record.fileSize) }}
        </template>
        <template v-if="column.key === 'action'">
          <a-space>
            <a @click.prevent="handlePreview(record)">预览</a>
            <a-divider type="vertical" />
            <a @click.prevent="handleDownload(record)">下载</a>
            <a-divider type="vertical" />
            <a-popconfirm
              title="确定删除?"
              @confirm="handleDelete(record.id)"
            >
              <a style="color: red">删除</a>
            </a-popconfirm>
          </a-space>
        </template>
      </template>
    </a-table>
  </a-card>

  <a-modal
      v-model:open="previewVisible"
      title="附件预览"
      :footer="null"
      width="800px"
      @cancel="closePreview"
    >
      <template v-if="previewFile">
        <template v-if="isImage(previewFile.mimeType)">
          <a-image
            :src="previewSrc"
            style="width: 100%"
          />
        </template>
        <template v-else-if="isVideo(previewFile.mimeType)">
          <video
            :src="previewSrc || undefined"
            controls
            style="width: 100%; max-height: 500px"
          />
        </template>
        <template v-else-if="isAudio(previewFile.mimeType)">
          <audio
            :src="previewSrc || undefined"
            controls
            style="width: 100%"
          />
        </template>
        <template v-else>
          <a-result title="不支持预览">
            <template #extra>
              <a-button
                type="primary"
                @click="handleDownload(previewFile)"
              >
                下载文件
              </a-button>
            </template>
          </a-result>
        </template>
      </template>
    </a-modal>

    <a-modal
      v-model:open="uploadVisible"
      title="上传附件"
      :confirm-loading="uploading"
      @ok="confirmUpload"
      @cancel="uploadVisible = false"
    >
      <a-form
        :label-col="{ span: 6 }"
        :wrapper-col="{ span: 16 }"
      >
        <a-form-item
          label="附件分类"
          required
        >
          <a-select
            v-model:value="uploadForm.category"
            placeholder="选择分类"
          >
            <a-select-option value="purchase">
              采购附件
            </a-select-option>
            <a-select-option value="ai">
              AI附件
            </a-select-option>
            <a-select-option value="system">
              系统附件
            </a-select-option>
          </a-select>
        </a-form-item>
        <a-form-item
          label="业务类型"
          required
        >
          <a-input
            v-model:value="uploadForm.bizType"
            placeholder="如 product/kyc_license/report"
          />
        </a-form-item>
        <a-form-item label="关联业务ID">
          <a-input
            v-model:value="uploadForm.bizId"
            placeholder="可选"
          />
        </a-form-item>
        <a-form-item label="备注">
          <a-textarea
            v-model:value="uploadForm.remark"
            placeholder="可选"
            :rows="2"
          />
        </a-form-item>
        <a-form-item
          label="选择文件"
          required
        >
          <AttachmentUploader
            ref="uploaderRef"
            :category="uploadForm.category"
            :biz-type="uploadForm.bizType || 'general'"
            :biz-id="uploadForm.bizId || undefined"
            :remark="uploadForm.remark"
            defer-upload
            :auto-load="false"
            button-text="选择文件"
            :show-empty="false"
          />
        </a-form-item>
      </a-form>
    </a-modal>
  </div>
</template>

<script setup lang="ts">
import { onBeforeUnmount, reactive, ref } from 'vue'
import { message } from 'ant-design-vue'
import { UploadOutlined, FileOutlined } from '@ant-design/icons-vue'
import AttachmentUploader from '@/components/AttachmentUploader.vue'
import AttachmentThumb from '@/components/AttachmentThumb.vue'
import {
  acquireImageUrl,
  attachmentApi,
  releaseImageUrl,
  type AttachmentVo,
} from '@/api/attachment'
import { useFitTableHeight } from '@/utils/useFitTableHeight'

const columns = [
  { title: '序号', key: 'index', width: 60 },
  { title: '缩略图', key: 'preview', width: 80 },
  { title: '分类', key: 'category', width: 100 },
  { title: '业务类型', dataIndex: 'bizType', width: 120 },
  { title: '原始文件名', dataIndex: 'originalName', ellipsis: true },
  { title: '文件类型', dataIndex: 'mimeType', width: 140 },
  { title: '文件大小', key: 'fileSize', width: 100 },
  { title: '上传人', dataIndex: 'createBy', width: 100 },
  { title: '上传时间', dataIndex: 'createTime', width: 170 },
  { title: '操作', key: 'action', width: 200, fixed: 'right' },
]

const loading = ref(false)
const uploading = ref(false)
const data = ref<AttachmentVo[]>([])
const selectedRowKeys = ref<string[]>([])
const previewVisible = ref(false)
const previewFile = ref<AttachmentVo | null>(null)
/** 预览弹窗当前渲染的 blob URL（直读端点需 Authorization 头，无法直接 <img src>） */
const previewSrc = ref('')
/** 预览弹窗持有的共享缓存引用，与 acquireImageUrl 成对 release */
let previewHolding: AttachmentVo | null = null

function releasePreview() {
  if (!previewHolding) return
  releaseImageUrl(previewHolding.id, previewHolding.bizType, previewHolding.bizId || undefined)
  previewHolding = null
}

const uploadVisible = ref(false)
const uploaderRef = ref<InstanceType<typeof AttachmentUploader>>()

const search = reactive({
  // category：界面「附件分类」下拉框，发送给后端 AttachmentQuery.category，筛选 auth_sys_attachment.category 精确 eq
  category: undefined as string | undefined,
  // bizType：界面「业务类型」输入框，发送给后端 AttachmentQuery.bizType，筛选 auth_sys_attachment.biz_type 精确 eq
  bizType: '',
})

const { fitRef, fitY } = useFitTableHeight()

const pagination = reactive({
  current: 1,
  pageSize: 20,
  total: 0,
  showSizeChanger: true,
  showTotal: (total: number) => `共 ${total} 条`,
  pageSizeOptions: ['20', '40', '60', '100'],
})

const uploadForm = reactive({
  category: 'purchase',
  bizType: '',
  bizId: '',
  remark: '',
})

const rowSelection = {
  selectedRowKeys,
  onChange: (keys: string[]) => {
    selectedRowKeys.value = keys
  },
}

function categoryLabel(cat: string) {
  const map: Record<string, string> = {
    purchase: '采购附件',
    ai: 'AI附件',
    system: '系统附件',
  }
  return map[cat] || cat
}

function categoryColor(cat: string) {
  const map: Record<string, string> = {
    purchase: 'blue',
    ai: 'green',
    system: 'orange',
  }
  return map[cat] || 'default'
}

function isImage(mime: string) {
  return mime.startsWith('image/')
}

function isVideo(mime: string) {
  return mime.startsWith('video/')
}

function isAudio(mime: string) {
  return mime.startsWith('audio/')
}

function formatFileSize(bytes: number) {
  if (bytes < 1024) return bytes + ' B'
  if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(1) + ' KB'
  return (bytes / (1024 * 1024)).toFixed(1) + ' MB'
}

async function fetchData() {
  loading.value = true
  try {
    const res = await attachmentApi.list({
      category: search.category,
      bizType: search.bizType || undefined,
      page: pagination.current,
      pageSize: pagination.pageSize,
    })
    data.value = res.items
    pagination.total = res.total
  } catch (e: any) {
    message.error(e.message || '加载失败')
  } finally {
    loading.value = false
  }
}

function onTableChange(pag: any) {
  pagination.current = pag.current
  pagination.pageSize = pag.pageSize
  fetchData()
}

function resetSearch() {
  search.category = undefined
  search.bizType = ''
  pagination.current = 1
  fetchData()
}

function openUploadModal() {
  uploadForm.bizType = ''
  uploadForm.bizId = ''
  uploadForm.remark = ''
  uploaderRef.value?.clearPending()
  uploadVisible.value = true
}

async function confirmUpload() {
  if (!uploadForm.category || !uploadForm.bizType) {
    message.warning('请填写分类和业务类型')
    return
  }
  if (!uploaderRef.value?.pending.length) {
    message.warning('请选择文件')
    return
  }
  uploading.value = true
  try {
    const result = await uploaderRef.value.uploadFiles()
    if (result.failed.length) {
      message.error(`${result.failed.length} 个文件上传失败`)
      return
    }
    message.success('上传成功')
    uploadVisible.value = false
    fetchData()
  } catch (e: any) {
    message.error(e.message || '上传失败')
  } finally {
    uploading.value = false
  }
}

function handlePreview(file: AttachmentVo) {
  previewFile.value = file
  previewVisible.value = true
  previewSrc.value = ''
  releasePreview()
  // 图/视频/音频需要下载字节转 blob（直读端点带鉴权，原生标签取不到）；其余走「不支持预览」分支
  if (!isImage(file.mimeType) && !isVideo(file.mimeType) && !isAudio(file.mimeType)) return
  acquireImageUrl(file.id, file.bizType, file.bizId || undefined).then((url) => {
    // 等待期间用户可能已关闭弹窗或切换到别的附件：丢弃过期结果并归还引用
    if (previewFile.value?.id !== file.id) {
      if (url) releaseImageUrl(file.id, file.bizType, file.bizId || undefined)
      return
    }
    previewHolding = file
    previewSrc.value = url || ''
  })
}

/** 关闭预览：清空状态并归还 blob 引用 */
function closePreview() {
  previewVisible.value = false
  previewFile.value = null
  previewSrc.value = ''
  releasePreview()
}

onBeforeUnmount(releasePreview)

async function handleDownload(file: AttachmentVo) {
  try {
    await attachmentApi.downloadFile(file.id, file.originalName)
  } catch (e: any) {
    message.error(e.message || '下载失败')
  }
}

async function handleDelete(id: string) {
  try {
    await attachmentApi.delete(id)
    message.success('删除成功')
    fetchData()
  } catch (e: any) {
    message.error(e.message || '删除失败')
  }
}

async function handleBatchDelete() {
  if (selectedRowKeys.value.length === 0) return
  try {
    await attachmentApi.batchDelete(selectedRowKeys.value)
    message.success('批量删除成功')
    selectedRowKeys.value = []
    fetchData()
  } catch (e: any) {
    message.error(e.message || '删除失败')
  }
}

fetchData()
</script>