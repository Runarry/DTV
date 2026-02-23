<template>
  <div class="douyin-home">
    <div class="douyin-content">
      <div class="left-panel">
        <CommonCategory
          :categoriesData="categoriesData"
          :selected-category-href="selectedCategory?.cate2Href"
          @category-selected="onCategorySelected"
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
      </div>
      <div class="right-panel">
        <CommonStreamerList
          ref="streamerListRef"
          :selectedCategory="selectedCategory"
          :categoriesData="categoriesData"
          platformName="douyin"
          playerRouteName="douyinPlayer"
        />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref, onActivated, onDeactivated, nextTick } from 'vue'
import CommonCategory from '../components/CommonCategory/index.vue'
import CommonStreamerList from '../components/CommonStreamerList/index.vue'
import { douyinCategoriesData } from '../platforms/douyin/douyinCategoriesData'
import type { CategorySelectedEvent } from '../platforms/common/categoryTypes'
import { useCustomCategoryStore } from '../store/customCategoryStore'
import { useNavigationStore } from '../stores/navigationStore'

defineOptions({
  name: 'DouyinHomeView',
})

const categoriesData = douyinCategoriesData
const selectedCategory = ref<CategorySelectedEvent | null>(null)
const customStore = useCustomCategoryStore()
const navigationStore = useNavigationStore()
const streamerListRef = ref<InstanceType<typeof CommonStreamerList> | null>(null)
customStore.ensureLoaded()

const canSubscribe = computed(() => !!selectedCategory.value?.cate2Href)
const isSubscribed = computed(() => {
  const href = selectedCategory.value?.cate2Href
  return !!href && customStore.isSubscribed('douyin', href)
})

function onCategorySelected(evt: CategorySelectedEvent) {
  selectedCategory.value = evt
}

const toggleSubscribe = () => {
  if (!selectedCategory.value?.cate2Href) return
  const href = selectedCategory.value.cate2Href
  if (customStore.isSubscribed('douyin', href)) {
    customStore.removeByKey(`douyin:${href}`)
  } else {
    customStore.addCommonCate2(
      'douyin',
      href,
      selectedCategory.value.cate2Name,
      selectedCategory.value.cate1Name,
      selectedCategory.value.cate1Href,
    )
  }
}

// Save state when leaving the view
onDeactivated(() => {
  navigationStore.setSourcePlatform('douyin')

  // Proactively save scroll position (don't rely on debounced scroll handler)
  const scrollEl = streamerListRef.value?.getScrollElement?.()
  if (scrollEl) {
    navigationStore.saveScrollPosition('douyin', scrollEl.scrollTop)
    navigationStore.persistScrollPosition('douyin')
  }

  if (selectedCategory.value) {
    navigationStore.saveCategoryState('douyin', {
      cate2Href: selectedCategory.value.cate2Href,
      cate2Name: selectedCategory.value.cate2Name,
    })
  }
})

// Restore state when returning to the view
onActivated(() => {
  const savedState = navigationStore.getPlatformState('douyin')
  if (savedState?.category && 'cate2Href' in savedState.category) {
    selectedCategory.value = {
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
.douyin-home {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: transparent;
}

.douyin-content {
  display: flex;
  flex-direction: column; /* 改为纵向排列，上下布局 */
  height: 100%;
}

.left-panel {
  width: 100%;
  background: transparent;
  backdrop-filter: none;
  z-index: 10;
  overflow: hidden;
}


.right-panel {
  flex: 1;
  overflow: hidden;
  background: transparent;
}
</style>
