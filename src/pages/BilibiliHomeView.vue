<template>
  <div class="bili-home-view-layout">
    <CommonCategory
      :categories-data="biliCategoriesData as any"
      :selected-category-href="currentSelectedCategory?.cate2Href"
      @category-selected="onCategorySelected"
      class="bili-category-section"
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

import { computed, ref, onActivated, onDeactivated, nextTick } from 'vue'
import CommonCategory from '../components/CommonCategory/index.vue'
import CommonStreamerList from '../components/CommonStreamerList/index.vue'
import { biliCategoriesData } from '../platforms/bilibili/biliCategoriesData'
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
  return !!href && customStore.isSubscribed('bilibili', href)
})
const onCategorySelected = (categoryEvent: CategorySelectedEvent) => {
  currentSelectedCategory.value = categoryEvent
}

const toggleSubscribe = () => {
  if (!currentSelectedCategory.value?.cate2Href) return
  const href = currentSelectedCategory.value.cate2Href
  if (customStore.isSubscribed('bilibili', href)) {
    customStore.removeByKey(`bilibili:${href}`)
  } else {
    customStore.addCommonCate2(
      'bilibili',
      href,
      currentSelectedCategory.value.cate2Name,
      currentSelectedCategory.value.cate1Name,
      currentSelectedCategory.value.cate1Href,
    )
  }
}

// Save state when leaving the view
onDeactivated(() => {
  navigationStore.setSourcePlatform('bilibili')

  // Proactively save scroll position (don't rely on debounced scroll handler)
  const scrollEl = streamerListRef.value?.getScrollElement?.()
  if (scrollEl) {
    navigationStore.saveScrollPosition('bilibili', scrollEl.scrollTop)
    navigationStore.persistScrollPosition('bilibili')
  }

  if (currentSelectedCategory.value) {
    navigationStore.saveCategoryState('bilibili', {
      cate2Href: currentSelectedCategory.value.cate2Href,
      cate2Name: currentSelectedCategory.value.cate2Name,
    })
  }
})

// Restore state when returning to the view
onActivated(() => {
  const savedState = navigationStore.getPlatformState('bilibili')
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
.bili-home-view-layout {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: transparent;
  overflow: hidden;
}


.bili-category-section {
  flex-shrink: 0;
  background: transparent;
  backdrop-filter: none;
  z-index: 10;
}

.bili-streamer-list-section {
  flex: 1;
  overflow: hidden;
  background: transparent;
}
</style>
