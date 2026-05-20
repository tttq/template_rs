<template>
  <a-select
    :value="modelValue"
    :placeholder="placeholder"
    :disabled="disabled"
    :loading="loading"
    :options="options"
    :allow-clear="allowClear"
    :mode="multiple ? 'multiple' : undefined"
    v-bind="$attrs"
    @update:value="onUpdate"
  />
</template>

<script setup lang="ts">
import { ref, onMounted, watch } from 'vue'
import { dictItemApi, type DictItemVo } from '@/api/dict'

const props = withDefaults(defineProps<{
  modelValue?: any
  dictCode: string
  placeholder?: string
  disabled?: boolean
  allowClear?: boolean
  multiple?: boolean
}>(), {
  placeholder: '请选择',
  disabled: false,
  allowClear: true,
  multiple: false,
})

const emit = defineEmits<{
  'update:modelValue': [value: any]
}>()

const loading = ref(false)
const options = ref<{ label: string; value: any }[]>([])
const dictItems = ref<DictItemVo[]>([])

async function fetchDictItems() {
  if (!props.dictCode) return
  loading.value = true
  try {
    const res = await dictItemApi.getByTypeCode(props.dictCode)
    dictItems.value = res
    options.value = res
      .filter((item) => item.status === 1)
      .map((item) => ({
        label: item.dictLabel,
        value: item.dictValue,
      }))
  } catch {
    options.value = []
  } finally {
    loading.value = false
  }
}

function onUpdate(value: any) {
  emit('update:modelValue', value)
}

function getDictItem(value: any): DictItemVo | undefined {
  return dictItems.value.find((item) => item.dictValue === value)
}

function getDictLabel(value: any): string {
  const item = getDictItem(value)
  return item?.dictLabel || ''
}

onMounted(() => {
  fetchDictItems()
})

watch(() => props.dictCode, () => {
  fetchDictItems()
})

defineExpose({ fetchDictItems, getDictItem, getDictLabel, dictItems })
</script>
