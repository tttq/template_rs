/**
 * 树形表格勾选级联（菜单树 / 角色树共用）
 *
 * 规则：
 * - 勾选父节点：自动勾选其下全部子节点（含更深层级）
 * - 取消子节点：只要父节点下还有任意子节点处于勾选状态，父节点就保持勾选；
 *   只有整棵子树都被取消，父节点才会逐级向上取消
 * - 勾选子节点：自动补全父节点，避免出现「子级选中、父级没选中」的悬挂状态
 * - 不可勾选节点（如菜单树里的客户端分组行）不参与级联，也不会出现在结果里
 */

export interface TreeSelectionOptions<T> {
  /** 树形数据，节点通过 children 串联 */
  tree: readonly T[]
  /** 取节点唯一键，默认取 node.id */
  getKey?: (node: T) => string
  /** 取子节点列表，默认取 node.children */
  getChildren?: (node: T) => readonly T[] | undefined
  /** 节点是否可勾选，默认全部可勾选 */
  isSelectable?: (node: T) => boolean
}

export interface CascadeTreeSelectionOptions<T> extends TreeSelectionOptions<T> {
  /** 上一次的勾选结果 */
  prevKeys: readonly string[]
  /** 表格刚回调的勾选结果（仅含本次被点击节点的增删） */
  nextKeys: readonly string[]
}

interface TreeSelectionIndex {
  /** 子节点 key → 父节点 key */
  parentOf: Map<string, string>
  /** 节点 key → 其下全部后代 key（深度优先，含各级） */
  descendantsOf: Map<string, string[]>
  /** 可勾选节点 key */
  selectable: Set<string>
}

function buildIndex<T>(
  tree: readonly T[],
  getKey: (node: T) => string,
  getChildren: (node: T) => readonly T[] | undefined,
  isSelectable: (node: T) => boolean,
): TreeSelectionIndex {
  const parentOf = new Map<string, string>()
  const descendantsOf = new Map<string, string[]>()
  const selectable = new Set<string>()

  /** 返回该节点下全部后代 key */
  const walk = (node: T, parentKey?: string): string[] => {
    const key = String(getKey(node))
    if (parentKey !== undefined) parentOf.set(key, parentKey)
    if (isSelectable(node)) selectable.add(key)
    const descendants: string[] = []
    for (const child of getChildren(node) || []) {
      descendants.push(String(getKey(child)), ...walk(child, key))
    }
    descendantsOf.set(key, descendants)
    return descendants
  }

  for (const root of tree) walk(root)

  return { parentOf, descendantsOf, selectable }
}

export function cascadeTreeSelection<T>(options: CascadeTreeSelectionOptions<T>): string[] {
  const getKey = options.getKey || ((node: T) => String((node as any).id))
  const getChildren = options.getChildren || ((node: T) => (node as any).children)
  const isSelectable = options.isSelectable || (() => true)
  const index = buildIndex(options.tree, getKey, getChildren, isSelectable)

  const result = new Set<string>(options.nextKeys.map(String))
  const prev = new Set<string>(options.prevKeys.map(String))

  // 不可勾选的真实节点（客户端分组行等）不允许进入结果
  for (const key of [...result]) {
    if (index.descendantsOf.has(key) && !index.selectable.has(key)) result.delete(key)
  }

  const added = [...result].filter((key) => !prev.has(key))
  const removed = [...prev].filter((key) => !result.has(key))

  /** 向上补全父级：父级随子级勾选 */
  const ensureAncestors = (key: string) => {
    let current: string | undefined = key
    while (current !== undefined) {
      if (index.selectable.has(current)) result.add(current)
      current = index.parentOf.get(current)
    }
  }

  // 勾选：向下级联全部子级，向上补齐父级
  for (const key of added) {
    if (!index.selectable.has(key)) continue
    for (const descendant of index.descendantsOf.get(key) || []) {
      if (index.selectable.has(descendant)) result.add(descendant)
    }
    ensureAncestors(key)
  }

  // 取消：向下取消整棵子树，再自底向上修正父级
  for (const key of removed) {
    if (!index.selectable.has(key)) continue
    result.delete(key)
    for (const descendant of index.descendantsOf.get(key) || []) result.delete(descendant)

    let parent = index.parentOf.get(key)
    while (parent !== undefined) {
      const hasCheckedDescendant = (index.descendantsOf.get(parent) || []).some((descendant) =>
        result.has(descendant),
      )
      if (hasCheckedDescendant) {
        // 父节点下还有子节点被勾选 → 父节点保持勾选（并保证其上级也保持勾选）
        ensureAncestors(parent)
        break
      }
      result.delete(parent)
      parent = index.parentOf.get(parent)
    }
  }

  return Array.from(result)
}
