<template>
  <a-form
    ref="formRef"
    :model="modelValue"
    :rules="rules"
    :label-col="labelCol"
    :wrapper-col="wrapperCol"
    :layout="layout"
  >
    <a-row :gutter="gutter">
      <template v-for="item in fields" :key="item.field">
        <a-col :span="item.span || span">
          <a-form-item :label="item.label" :name="item.field" :rules="item.rules">
            <template v-if="item.slotName">
              <slot :name="item.slotName" :model="modelValue" :field="item.field" />
            </template>
            <template v-else-if="item.type === 'input'">
              <a-input
                :value="modelValue[item.field]"
                :placeholder="item.placeholder || `请输入${item.label}`"
                :disabled="item.disabled"
                v-bind="item.props"
                @update:value="onUpdate(item.field, $event)"
              />
            </template>
            <template v-else-if="item.type === 'password'">
              <a-input-password
                :value="modelValue[item.field]"
                :placeholder="item.placeholder || `请输入${item.label}`"
                :disabled="item.disabled"
                v-bind="item.props"
                @update:value="onUpdate(item.field, $event)"
              />
            </template>
            <template v-else-if="item.type === 'textarea'">
              <a-textarea
                :value="modelValue[item.field]"
                :placeholder="item.placeholder || `请输入${item.label}`"
                :disabled="item.disabled"
                v-bind="item.props"
                @update:value="onUpdate(item.field, $event)"
              />
            </template>
            <template v-else-if="item.type === 'number'">
              <a-input-number
                :value="modelValue[item.field]"
                :placeholder="item.placeholder || `请输入${item.label}`"
                :disabled="item.disabled"
                style="width: 100%"
                v-bind="item.props"
                @update:value="onUpdate(item.field, $event)"
              />
            </template>
            <template v-else-if="item.type === 'select'">
              <a-select
                :value="modelValue[item.field]"
                :placeholder="item.placeholder || `请选择${item.label}`"
                :disabled="item.disabled"
                :options="item.options"
                allow-clear
                v-bind="item.props"
                @update:value="onUpdate(item.field, $event)"
              />
            </template>
            <template v-else-if="item.type === 'dictSelect'">
              <DictSelect
                :value="modelValue[item.field]"
                :dict-code="item.dictCode!"
                :placeholder="item.placeholder || `请选择${item.label}`"
                :disabled="item.disabled"
                v-bind="item.props"
                @update:value="onUpdate(item.field, $event)"
              />
            </template>
            <template v-else-if="item.type === 'userPicker'">
              <UserPicker
                :value="modelValue[item.field]"
                :display-field="item.props?.displayField || 'userName'"
                :placeholder="item.placeholder || `请选择${item.label}`"
                :disabled="item.disabled"
                v-bind="item.props"
                @update:value="onUpdate(item.field, $event)"
                @change="onFieldChange(item.field, $event)"
              />
            </template>
            <template v-else-if="item.type === 'datePicker'">
              <a-date-picker
                :value="modelValue[item.field]"
                :placeholder="item.placeholder || `请选择${item.label}`"
                :disabled="item.disabled"
                style="width: 100%"
                v-bind="item.props"
                @update:value="onUpdate(item.field, $event)"
              />
            </template>
            <template v-else-if="item.type === 'rangePicker'">
              <a-range-picker
                :value="modelValue[item.field]"
                :placeholder="item.placeholder || ['开始时间', '结束时间']"
                :disabled="item.disabled"
                style="width: 100%"
                v-bind="item.props"
                @update:value="onUpdate(item.field, $event)"
              />
            </template>
            <template v-else-if="item.type === 'switch'">
              <a-switch
                :checked="modelValue[item.field]"
                :disabled="item.disabled"
                v-bind="item.props"
                @update:checked="onUpdate(item.field, $event)"
              />
            </template>
            <template v-else-if="item.type === 'radio'">
              <a-radio-group
                :value="modelValue[item.field]"
                :disabled="item.disabled"
                :options="item.options"
                v-bind="item.props"
                @update:value="onUpdate(item.field, $event)"
              />
            </template>
            <template v-else-if="item.type === 'checkbox'">
              <a-checkbox-group
                :value="modelValue[item.field]"
                :disabled="item.disabled"
                :options="item.options"
                v-bind="item.props"
                @update:value="onUpdate(item.field, $event)"
              />
            </template>
          </a-form-item>
        </a-col>
      </template>
    </a-row>
  </a-form>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import type { FormInstance } from 'ant-design-vue'
import type { FormField } from './types'
import DictSelect from '../DictSelect.vue'
import UserPicker from '../UserPicker.vue'

const props = withDefaults(defineProps<{
  modelValue: Record<string, any>
  fields: FormField[]
  rules?: Record<string, any[]>
  labelCol?: Record<string, number>
  wrapperCol?: Record<string, number>
  layout?: 'horizontal' | 'vertical' | 'inline'
  gutter?: number
  span?: number
}>(), {
  labelCol: () => ({ span: 6 }),
  wrapperCol: () => ({ span: 16 }),
  layout: 'horizontal',
  gutter: 16,
  span: 24,
})

const emit = defineEmits<{
  'update:modelValue': [value: Record<string, any>]
  'fieldChange': [field: string, value: any]
}>()

const formRef = ref<FormInstance>()

function onUpdate(field: string, value: any) {
  emit('update:modelValue', { ...props.modelValue, [field]: value })
}

function onFieldChange(field: string, value: any) {
  emit('fieldChange', field, value)
}

function validate() {
  return formRef.value?.validate()
}

function resetFields() {
  formRef.value?.resetFields()
}

function clearValidate() {
  formRef.value?.clearValidate()
}

defineExpose({ validate, resetFields, clearValidate, formRef })
</script>
