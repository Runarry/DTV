<template>
  <div v-if="!isLoading && (cate3List.length > 0 || hasAllOption)" class="cate3-list">
    <!-- 全部选项 -->
    <div
      class="cate3-item"
      :class="{ active: selectedCate3Id === null || selectedCate3Id === 'all' }"
      @click="selectAll"
    >
      全部
    </div>
    
    <!-- 其他三级分类 -->
    <div
      v-for="cate3 in cate3List"
      :key="cate3.id"
      class="cate3-item"
      :class="{ active: selectedCate3Id === cate3.id }"
      @click="$emit('select', cate3)"
    >
      {{ cate3.name }}
    </div>
  </div>
  <div v-if="isLoading" class="loading-cate3">正在加载三级分类...</div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import type { Category3 } from '../types'

const props = defineProps<{
  cate3List: Category3[]
  selectedCate3Id: string | null
  isLoading: boolean
}>()

const emit = defineEmits<{
  (e: 'select', cate3: Category3): void
}>()

// 计算属性：是否显示全部选项
const hasAllOption = computed(() => {
  return props.cate3List && props.cate3List.length > 0
})

// 选择"全部"
const selectAll = () => {
  // 创建一个特殊的"全部"分类对象
  const allCategory: Category3 = {
    id: 'all',
    name: '全部'
  }
  emit('select', allCategory)
}
</script>

<style scoped>
.cate3-list {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  margin: 2px 16px 0 16px;
  padding-bottom: 4px;
}

.cate3-item {
  padding: 4px 12px;
  height: 28px;
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
  box-sizing: border-box;
  display: inline-flex;
  align-items: center;
  font-size: 12px;
  background: var(--secondary-bg);
  border: 1px solid var(--border-color);
  color: var(--secondary-text);
  font-weight: 450;
}

.cate3-item:hover {
  background: var(--tertiary-bg);
  border-color: var(--accent-color);
  color: var(--primary-text);
}

.cate3-item.active {
  background: var(--accent-gradient);
  border-color: transparent;
  color: #fff;
  font-weight: 600;
}

:root[data-theme="light"] .cate3-item {
  background: #f4f4f5;
  border: 1px solid #e4e4e7;
  color: #71717a;
}

:root[data-theme="light"] .cate3-item:hover {
  background: #e4e4e7;
  color: #18181b;
}

:root[data-theme="light"] .cate3-item.active {
  background: var(--accent-gradient);
  border-color: transparent;
  color: #fff;
}

.loading-cate3 {
  display: flex;
  justify-content: center;
  align-items: center;
  padding: 10px;
  color: var(--cate3-loading-text-dark, rgba(255, 255, 255, 0.5)); 
  font-size: 13px;
}

:root[data-theme="light"] .loading-cate3 {
  color: var(--main-text-secondary-light, #495057); 
}
</style>
