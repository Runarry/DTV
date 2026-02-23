<template>
  <div class="huya-home-view-layout">
    <CommonCategory
      :categories-data="huyaCategoriesData as any"
      :selected-category-href="currentSelectedCategory?.cate2Href"
      @category-selected="onCategorySelected"
      class="huya-category-section"
    >
      <template #actions>
        <button
          type="button"
          class="category-subscribe-btn"
          :disabled="!canSubscribe"
          @click="toggleSubscribe"
        >
          {{ isSubscribed ? '取消订阅' : '订阅分区' }}
        </button>
      </template>
    </CommonCategory>
    <CommonStreamerList
      ref="streamerListRef"
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

import { computed, ref, onActivated, onDeactivated, nextTick } from 'vue'
import CommonCategory from '../components/CommonCategory/index.vue'
import { huyaCategoriesData } from '../platforms/huya/huyaCategoriesData'
import CommonStreamerList from '../components/CommonStreamerList/index.vue'
import type { CategorySelectedEvent } from '../platforms/common/categoryTypes.ts'
import { useCustomCategoryStore } from '../store/customCategoryStore'
import { useNavigationStore } from '../stores/navigationStore'

const currentSelectedCategory = ref<CategorySelectedEvent | null>(null)
const customStore = useCustomCategoryStore()
const navigationStore = useNavigationStore()
const streamerListRef = ref<InstanceType<typeof CommonStreamerList> | null>(null)
customStore.ensureLoaded()

const canSubscribe = computed(() => !!currentSelectedCategory.value?.cate2Href)
const isSubscribed = computed(() => {
  const href = currentSelectedCategory.value?.cate2Href
  return !!href && customStore.isSubscribed('huya', href)
})
const onCategorySelected = (categoryEvent: CategorySelectedEvent) => {
  currentSelectedCategory.value = categoryEvent
}

const toggleSubscribe = () => {
  if (!currentSelectedCategory.value?.cate2Href) return
  const href = currentSelectedCategory.value.cate2Href
  if (customStore.isSubscribed('huya', href)) {
    customStore.removeByKey(`huya:${href}`)
  } else {
    customStore.addCommonCate2(
      'huya',
      href,
      currentSelectedCategory.value.cate2Name,
      currentSelectedCategory.value.cate1Name,
      currentSelectedCategory.value.cate1Href,
    )
  }
}

// Save state when leaving the view
onDeactivated(() => {
  navigationStore.setSourcePlatform('huya')

  // Proactively save scroll position (don't rely on debounced scroll handler)
  const scrollEl = streamerListRef.value?.getScrollElement?.()
  if (scrollEl) {
    navigationStore.saveScrollPosition('huya', scrollEl.scrollTop)
    navigationStore.persistScrollPosition('huya')
  }

  if (currentSelectedCategory.value) {
    navigationStore.saveCategoryState('huya', {
      cate2Href: currentSelectedCategory.value.cate2Href,
      cate2Name: currentSelectedCategory.value.cate2Name,
    })
  }
})

// Restore state when returning to the view
onActivated(() => {
  const savedState = navigationStore.getPlatformState('huya')
  if (savedState?.category && 'cate2Href' in savedState.category) {
    currentSelectedCategory.value = {
      cate2Href: savedState.category.cate2Href,
      cate2Name: savedState.category.cate2Name,
    } as CategorySelectedEvent

    nextTick(() => {
      streamerListRef.value?.restoreScrollPosition(savedState.scrollPosition)
    })
  }
})
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
  background: transparent;
  backdrop-filter: var(--glass-blur);
  z-index: 10;
}

.huya-streamer-list-section {
  flex: 1;
  overflow: hidden;
  background: transparent;
}
</style>
