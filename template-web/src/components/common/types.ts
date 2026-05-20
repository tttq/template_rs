export interface PageResult<T> {
  items: T[]
  total: number
  page: number
  pageSize: number
  totalPages: number
}

export type FieldType =
  | 'input'
  | 'password'
  | 'textarea'
  | 'number'
  | 'select'
  | 'dictSelect'
  | 'userPicker'
  | 'datePicker'
  | 'rangePicker'
  | 'switch'
  | 'radio'
  | 'checkbox'
  | 'custom'

export interface FormField {
  field: string
  label: string
  type: FieldType
  placeholder?: string
  rules?: any[]
  span?: number
  disabled?: boolean
  options?: { label: string; value: any }[]
  dictCode?: string
  props?: Record<string, any>
  slotName?: string
}

export interface TableColumn {
  title: string
  dataIndex?: string
  key: string
  width?: number
  ellipsis?: boolean
  fixed?: 'left' | 'right'
  sorter?: boolean | ((a: any, b: any) => number)
  scopedSlots?: boolean
}
