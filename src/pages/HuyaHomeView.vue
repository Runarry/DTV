<template>
  <div class="huya-home-view-layout">
    <CommonCategory 
      :categories-data="huyaCategoriesData as any"
      @category-selected="onCategorySelected"
      class="huya-category-section"
    />
    <CommonStreamerList 
      :selected-category="currentSelectedCategory"
      :categories-data="huyaCategoriesData as any"
      :default-page-size="120"
      playerRouteName="huyaPlayer"
      class="huya-streamer-list-section"
    />
  </div>
</template>

<script setup lang="ts">
defineOptions({
  name: 'HuyaHomeView'
})

import { ref } from 'vue'
import CommonCategory from '../components/CommonCategory/index.vue'
import { huyaCategoriesData } from '../platforms/huya/huyaCategoriesData'
import CommonStreamerList from '../components/CommonStreamerList/index.vue'
import type { CategorySelectedEvent } from '../platforms/common/categoryTypes.ts'

const currentSelectedCategory = ref<CategorySelectedEvent | null>(null)
const onCategorySelected = (categoryEvent: CategorySelectedEvent) => {
  currentSelectedCategory.value = categoryEvent
}
</script>

<style scoped>
.huya-home-view-layout {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: transparent;
  overflow: hidden;
}

.huya-category-section {
  flex-shrink: 0;
  background: var(--glass-bg);
  backdrop-filter: blur(8px);
  z-index: 10;
}

.huya-streamer-list-section {
  flex: 1;
  overflow: hidden;
  background: transparent;
}
</style>
