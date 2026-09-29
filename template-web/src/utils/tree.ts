/**
 * 将后端返回的扁平列表（每项含 id / parentId）按 parentId 构建为树形结构，
 * 供 antdv Table（树形行）/ TreeSelect / Tree 等树形展示使用。
 *
 * @param items 扁平列表
 * @param parentIdField 父级 ID 字段名，默认 'parentId'
 * @returns 树形根节点数组（children 一律初始化为数组）
 */
export function buildTree<T extends Record<string, any>>(
  items: T[],
  parentIdField: keyof T = 'parentId',
): T[] {
  const map = new Map<unknown, T & { children: T[] }>()
  for (const item of items) {
    map.set(item.id, { ...item, children: [] })
  }
  const roots: (T & { children: T[] })[] = []
  for (const node of map.values()) {
    const pid = node[parentIdField]
    // 自引用（parentId === id）或父级不在列表内时视为根节点，避免死循环
    const parent = pid !== undefined && pid !== null && pid !== node.id
      ? map.get(pid)
      : undefined
    if (parent) {
      parent.children.push(node)
    } else {
      roots.push(node)
    }
  }
  return roots
}