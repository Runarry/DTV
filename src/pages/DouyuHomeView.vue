<template>
  <div class="home-page">
    <!-- 分类区域 -->
    <div class="category-section" ref="categorySectionRef">
      <CategoryList
        ref="categoryListRef"
        @category-selected="handleCategorySelected"
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
      </CategoryList>
    </div>
    <!-- 主播列表区域 -->
    <div
      class="live-list-section"
      v-if="selectedCategoryInfo"
    >
      <CommonStreamerList
        ref="streamerListRef"
        :douyu-category="selectedCategoryInfo"
        platformName="douyu"
        playerRouteName="douyuPlayer"
        :key="selectedCategoryInfo.type + '-' + selectedCategoryInfo.id"
      />
    </div>
    <!-- 加载状态显示 (for default category) -->
    <div class="loading-section" v-else-if="isLoadingDefaultCategory">
      <div class="loading-message">正在加载默认分类...</div>
    </div>
    <div class="loading-section" v-else>
      <div class="loading-message">请先选择一个分类。</div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref, onMounted, onActivated, onDeactivated, nextTick } from 'vue'

// Added for KeepAlive include by name
defineOptions({
  name: 'DouyuHomeView'
})

import CategoryList from '../components/DouyuCategory/index.vue';
import CommonStreamerList from '../components/CommonStreamerList/index.vue';
import { invoke } from '@tauri-apps/api/core'
import type { CategorySelectedEvent } from '../components/DouyuCategory/types';
import { useCustomCategoryStore } from '../store/customCategoryStore'
import { useNavigationStore } from '../stores/navigationStore'

// Types for the data structure returned by the Rust command `fetch_categories`
interface FrontendCate3Item {
    id: string;
    name: string;
}
interface FrontendCate2Item {
    id: string;
    name: string;
    short_name: string;
    icon: string;
    cate3List: FrontendCate3Item[];
}
interface FrontendCate1Item {
    id: string;
    name: string;
    cate2List: FrontendCate2Item[];
}

// Updated to match the new Rust response structure
interface FrontendCategoryResponse {
    cate1List: FrontendCate1Item[];
}

// Define a type for the selected category information to be passed to LiveList
interface SelectedCategoryInfo {
  type: 'cate2' | 'cate3';
  id: string; // shortName for cate2, cate3Id for cate3
  name?: string; // cate2Name or cate3Name
  parentCate2Id?: string; // shortName for parent cate2 (required for cate3 restore)
}

const selectedCategoryInfo = ref<SelectedCategoryInfo | null>(null);
const categorySectionRef = ref<HTMLElement | null>(null)
const categoryListRef = ref<InstanceType<typeof CategoryList> | null>(null)
const isLoadingDefaultCategory = ref(true);
const customStore = useCustomCategoryStore()
const navigationStore = useNavigationStore()
const streamerListRef = ref<InstanceType<typeof CommonStreamerList> | null>(null)
customStore.ensureLoaded()

const canSubscribe = computed(() => selectedCategoryInfo.value?.type === 'cate2' && !!selectedCategoryInfo.value?.id)
const isSubscribed = computed(() => {
  const id = selectedCategoryInfo.value?.id
  return !!id && customStore.isSubscribed('douyu', id)
})

const handleCategorySelected = (event: CategorySelectedEvent) => {
  if (event.type === 'cate2' && event.shortName) {
    selectedCategoryInfo.value = {
      type: 'cate2',
      id: event.shortName,
      name: event.cate2Name || event.shortName,
      parentCate2Id: event.shortName,
    };
  } else if (event.type === 'cate3' && event.cate3Id) {
    selectedCategoryInfo.value = {
      type: 'cate3',
      id: event.cate3Id,
      name: event.cate3Name || undefined,
      parentCate2Id: event.shortName,
    };
  } else {
    console.warn('Received category selection event with missing/invalid data:', event);
    selectedCategoryInfo.value = null;
  }
  // To ensure LiveList re-renders, we already use :key.
}

const toggleSubscribe = () => {
  if (!selectedCategoryInfo.value || selectedCategoryInfo.value.type !== 'cate2') return
  const id = selectedCategoryInfo.value.id
  if (!id) return
  if (customStore.isSubscribed('douyu', id)) {
    customStore.removeByKey(`douyu:${id}`)
  } else {
    customStore.addDouyuCate2(id, selectedCategoryInfo.value.name || id)
  }
}

const fetchDefaultCategory = async () => {
  isLoadingDefaultCategory.value = true;
  try {
    const response = await invoke('fetch_categories') as FrontendCategoryResponse;
    if (response && response.cate1List && response.cate1List.length > 0) {
      const firstCate1 = response.cate1List[0];
      if (firstCate1 && firstCate1.cate2List && firstCate1.cate2List.length > 0) {
        const defaultCate2 = firstCate1.cate2List[0];
        if (defaultCate2 && defaultCate2.short_name) {
          selectedCategoryInfo.value = {
            type: 'cate2',
            id: defaultCate2.short_name,
            name: defaultCate2.name 
          };
        } else {
          console.error('HomeView: Default second-level category or its short_name is missing.');
          selectedCategoryInfo.value = null;
        }
      } else {
        console.error('HomeView: Default first-level category does not have any second-level categories.');
        selectedCategoryInfo.value = null;
      }
    } else {
      console.error('HomeView: 未能获取到有效的默认分类数据或默认分类结构不正确. Response:', response);
      selectedCategoryInfo.value = null;
    }
  } catch (error) {
    console.error('HomeView: 获取默认分类失败:', error);
    selectedCategoryInfo.value = null;
  } finally {
    isLoadingDefaultCategory.value = false;
  }
}

const sleep = (ms: number) => new Promise<void>((resolve) => {
  setTimeout(resolve, ms)
})

const waitForCategoryListReady = async (timeoutMs = 3000) => {
  const startAt = Date.now()
  while (Date.now() - startAt < timeoutMs) {
    const categoryList = categoryListRef.value as any
    if (categoryList && Array.isArray(categoryList.cate2List) && categoryList.cate2List.length > 0) {
      return categoryList
    }
    await nextTick()
    await sleep(60)
  }
  return null
}

const findCate3WithRetry = async (categoryList: any, cate3Id: string) => {
  for (let i = 0; i < 20; i += 1) {
    const currentCate3List = Array.isArray(categoryList.currentCate3List) ? categoryList.currentCate3List : []
    const cate3 = currentCate3List.find((c: any) => String(c.id) === cate3Id)
    if (cate3) return cate3
    await sleep(60)
  }
  return null
}

const restoreSavedDouyuState = async () => {
  const savedState = navigationStore.getPlatformState('douyu')
  if (!(savedState?.category && 'douyuCategoryType' in savedState.category)) {
    return false
  }

  const categoryList = await waitForCategoryListReady()
  if (!categoryList) {
    return false
  }

  const savedId = savedState.category.douyuCategoryId
  const savedType = savedState.category.douyuCategoryType
  const savedParentCate2Id = savedState.category.douyuParentCate2Id
    || (savedType === 'cate2' ? savedId : undefined)

  const cate2List: any[] = Array.isArray(categoryList.cate2List) ? categoryList.cate2List : []
  const targetCate2 = cate2List.find((c) => {
    if (savedType === 'cate2') {
      return String(c.cate2Id) === savedId || c.shortName === savedId
    }
    return !!savedParentCate2Id
      && (String(c.cate2Id) === savedParentCate2Id || c.shortName === savedParentCate2Id)
  })

  const fallbackCate2 = cate2List[0]
  const cate2ToUse = targetCate2 || fallbackCate2
  if (!cate2ToUse) {
    return false
  }

  categoryList.selectCate1(cate2ToUse.cate1Id)
  await nextTick()

  const sortedCate2List: any[] = Array.isArray(categoryList.sortedCate2List) ? categoryList.sortedCate2List : []
  const cate2InCurrentList = sortedCate2List.find((c) => c.cate2Id === cate2ToUse.cate2Id) || cate2ToUse
  categoryList.handleCate2SelectAndCollapse(cate2InCurrentList)

  let restoredAsCate3 = false
  if (savedType === 'cate3' && targetCate2) {
    await nextTick()
    const cate3 = await findCate3WithRetry(categoryList, savedId)
    if (cate3) {
      categoryList.handleCate3Click(cate3)
      restoredAsCate3 = true
    }
  }

  if (!targetCate2 || (savedType === 'cate3' && !restoredAsCate3)) {
    const fallbackId = cate2ToUse.shortName || String(cate2ToUse.cate2Id)
    navigationStore.saveCategoryState('douyu', {
      douyuCategoryType: 'cate2',
      douyuCategoryId: fallbackId,
      douyuCategoryName: cate2ToUse.cate2Name,
      douyuParentCate2Id: fallbackId,
    })
  }

  if (savedState.scrollPosition > 0) {
    await nextTick()
    setTimeout(() => {
      streamerListRef.value?.restoreScrollPosition(savedState.scrollPosition)
    }, 150)
  }

  return true
}

onMounted(() => {
  const savedState = navigationStore.getPlatformState('douyu')
  if (!(savedState?.category && 'douyuCategoryType' in savedState.category)) {
    fetchDefaultCategory()
  }
})

// Save state when leaving the view
onDeactivated(() => {
  navigationStore.setSourcePlatform('douyu')

  // Proactively save scroll position (don't rely on debounced scroll handler)
  const scrollEl = streamerListRef.value?.getScrollElement?.()
  if (scrollEl) {
    navigationStore.saveScrollPosition('douyu', scrollEl.scrollTop)
    navigationStore.persistScrollPosition('douyu')
  }

  if (selectedCategoryInfo.value) {
    const categoryList = categoryListRef.value
    const activeCate2 = categoryList?.sortedCate2List.find(
      (c: any) => c.cate2Id === categoryList.selectedCate2Id
    )
    const parentCate2Id = selectedCategoryInfo.value.type === 'cate2'
      ? selectedCategoryInfo.value.id
      : selectedCategoryInfo.value.parentCate2Id || activeCate2?.shortName

    navigationStore.saveCategoryState('douyu', {
      douyuCategoryType: selectedCategoryInfo.value.type,
      douyuCategoryId: selectedCategoryInfo.value.id,
      douyuCategoryName: selectedCategoryInfo.value.name,
      douyuParentCate2Id: parentCate2Id,
    })
  }
})

// Restore state when returning to the view
onActivated(async () => {
  const savedState = navigationStore.getPlatformState('douyu')
  if (!(savedState?.category && 'douyuCategoryType' in savedState.category)) {
    return
  }

  const restored = await restoreSavedDouyuState()
  if (!restored && !selectedCategoryInfo.value) {
    await fetchDefaultCategory()
  }
})
</script>

<style scoped>
.home-page {
  height: 100%;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  background: transparent;
  min-width: 0;
}

.category-section {
  flex-shrink: 0; 
  z-index: 10; 
  position: sticky; 
  top: 0;
  width: 100%;
  background: transparent;
  backdrop-filter: none;
}


.live-list-section {
  flex-grow: 1; 
  overflow: hidden; 
  width: 100%;
  background: transparent;
  display: flex; 
  flex-direction: column; 
}

.loading-section {
  flex: 1; 
  display: flex;
  justify-content: center;
  align-items: center;
  background: transparent;
}

.loading-message {
  color: var(--secondary-text);
  font-size: 16px;
}
</style>
