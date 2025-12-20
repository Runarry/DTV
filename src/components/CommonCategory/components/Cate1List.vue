<template>
  <div class="cate1-list-container">
    <ul class="cate1-list">
      <li
        v-for="cate1 in cate1List"
        :key="cate1.href"
        class="cate1-item"
        :class="{ selected: cate1.href === selectedCate1Href }"
        @click="$emit('select', cate1)"
      >
        <span class="cate1-name">{{ cate1.title }}</span>
      </li>
    </ul>
  </div>
</template>

<script setup lang="ts">
import type { Category1 } from '../../../platforms/common/categoryTypes.ts'

defineProps<{
  cate1List: Category1[]
  selectedCate1Href: string | null
}>()
</script>

<style scoped>
.cate1-list-container {
  overflow-x: auto;
  background: transparent;
  flex-shrink: 0;
  padding: 12px 0 6px;
  transition: all 0.2s ease;
}

.cate1-list {
  list-style: none;
  margin: 0;
  padding: 0 24px;
  display: flex;
  gap: 16px;
  flex-wrap: nowrap;
}

.cate1-item {
  height: 38px;
  padding: 0 16px;
  display: inline-flex;
  align-items: center;
  cursor: pointer;
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  font-size: 13px;
  font-weight: 600;
  border-radius: 100px;
  white-space: nowrap;
  color: var(--secondary-text);
  background: var(--hover-bg);
  backdrop-filter: var(--glass-blur);
  -webkit-backdrop-filter: var(--glass-blur);
  border: none;
  box-shadow: none;
}

.cate1-item:hover {
  background: var(--hover-bg);
  color: var(--primary-text);
  transform: translateY(-1px);
}

.cate1-item.selected {
  background: #2c3036;
  color: #f8fafc;
  font-weight: 700;
  box-shadow: none;
}

.cate1-name {
  display: inline-block;
  letter-spacing: 0.2px;
}

.cate1-list-container::-webkit-scrollbar {
  height: 0;
}

:root[data-theme="light"] .cate1-item.selected {
  background: #e4e9e5;
  color: #111827;
}
</style>
