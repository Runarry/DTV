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
  padding: 16px 0;
  border-bottom: 1px solid var(--border-color);
  transition: all 0.2s ease;
}

.cate1-list {
  list-style: none;
  margin: 0;
  padding: 0 32px;
  display: flex;
  gap: 16px;
  flex-wrap: nowrap;
}

.cate1-item {
  padding: 10px 24px;
  cursor: pointer;
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  font-size: 15px;
  font-weight: 600;
  border-radius: 100px;
  white-space: nowrap;
  color: var(--secondary-text);
  background: var(--glass-bg);
  backdrop-filter: var(--glass-blur);
  -webkit-backdrop-filter: var(--glass-blur);
  border: 1px solid var(--glass-border);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.05);
}

.cate1-item:hover {
  background: var(--hover-bg);
  color: var(--primary-text);
  border-color: var(--accent-color);
  transform: translateY(-1px);
}

.cate1-item.selected {
  background: var(--accent-gradient);
  color: var(--accent-text);
  font-weight: 700;
  border-color: transparent;
  box-shadow: 0 8px 24px rgba(191, 255, 0, 0.2);
}

.cate1-name {
  display: inline-block;
  letter-spacing: 0.2px;
}

.cate1-list-container::-webkit-scrollbar {
  height: 0;
}
</style>
