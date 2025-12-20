<template>
  <div class="cate1-list">
    <div 
      v-for="cate1 in cate1List" 
      :key="cate1.cate1Id"
      class="cate1-item"
      :class="{ active: selectedCate1Id === cate1.cate1Id }"
      @click="$emit('select', cate1.cate1Id)"
    >
      <span class="cate1-name">{{ cate1.cate1Name }}</span>
    </div>
  </div>
</template>

<script setup lang="ts">
import type { Category1 } from '../types'

defineProps<{
  cate1List: Category1[]
  selectedCate1Id: number | null
}>()

defineEmits<{
  (e: 'select', cate1Id: number): void
}>()
</script>

<style scoped>
.cate1-list {
  padding: 10px 12px 8px;
  display: flex;
  gap: 12px;
  overflow-x: auto;
  background: var(--cate1-list-bg-dark, #1f1f23);
  flex-shrink: 0;
  transition: background-color 0.3s ease;
}

:root[data-theme="light"] .cate1-list {
  background-color: var(--primary-bg, #FFFFFF);
  color: var(--primary-text);
}

.cate1-item {
  height: 40px;
  padding: 0 12px;
  cursor: pointer;
  transition: background 0.22s ease, border-color 0.22s ease, box-shadow 0.22s ease, color 0.22s ease;
  font-size: 13px;
  border-radius: var(--radius-md);
  white-space: nowrap;
  border: 1px solid rgba(90, 176, 255, 0.14);
  background: #1e2022;
  color: rgba(227, 236, 255, 0.82);
  display: inline-flex;
  align-items: center;
}

:root[data-theme="light"] .cate1-item {
  background: rgba(248, 250, 255, 0.92);
  border-color: rgba(203, 213, 225, 0.6);
  color: #334155;
  box-shadow: none;
}

.cate1-item:hover {
  background: #272a2d;
  border-color: rgba(125, 211, 252, 0.3);
  color: rgba(236, 244, 255, 0.92);
}

:root[data-theme="light"] .cate1-item:hover {
  background: rgba(241, 245, 255, 0.96);
  border-color: rgba(148, 163, 184, 0.65);
  color: #1f2f4d;
}

.cate1-item.active {
  background: #2c3036;
  border: 1px solid #3b4147;
  color: #f8fafc;
  font-weight: 600;
}

:root[data-theme="light"] .cate1-item.active {
  background: #e4e9e5;
  border-color: #cfd3cf;
  color: #111827;
  font-weight: 600;
}
</style>
