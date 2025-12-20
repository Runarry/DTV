<template>
  <div class="bili-home-view-layout">
    <CommonCategory 
      :categories-data="biliCategoriesData as any"
      @category-selected="onCategorySelected"
      class="bili-category-section"
    />
    <CommonStreamerList 
      :selected-category="currentSelectedCategory"
      :categories-data="biliCategoriesData as any"
      platformName="bilibili"
      playerRouteName="bilibiliPlayer"
      class="bili-streamer-list-section"
    />
  </div>
</template>

<script setup lang="ts">
defineOptions({
  name: 'BilibiliHomeView'
})

import { ref } from 'vue'
import CommonCategory from '../components/CommonCategory/index.vue'
import CommonStreamerList from '../components/CommonStreamerList/index.vue'
import { biliCategoriesData } from '../platforms/bilibili/biliCategoriesData'
import type { CategorySelectedEvent } from '../platforms/common/categoryTypes.ts'

const currentSelectedCategory = ref<CategorySelectedEvent | null>(null)
const onCategorySelected = (categoryEvent: CategorySelectedEvent) => {
  currentSelectedCategory.value = categoryEvent
}
</script>

<style scoped>
.bili-home-view-layout {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: transparent;
  overflow: hidden;
}

.bili-category-section {
  flex-shrink: 0;
  background: var(--glass-bg);
  backdrop-filter: blur(8px);
  z-index: 10;
}

.bili-streamer-list-section {
  flex: 1;
  overflow: hidden;
  background: transparent;
}
</style>
