<template>
  <div class="common-live-list-container">
    <div v-if="isLoading && rooms.length === 0" class="loading-initial-state">
      <LoadingDots />
    </div>
    <div v-else-if="!isLoading && rooms.length === 0 && hasCategory" class="no-streamers-state">
      <p>分类下暂无主播</p>
    </div>
    <div v-else-if="!hasCategory && !isLoading" class="no-category-state">
       <p>请选择一个分类开始浏览</p>
    </div>

    <div
      v-else
      ref="scrollComponentRef"
      class="live-grid-scroll-area"
      @scroll.passive="handleScrollerScroll"
    >
      <div class="live-grid-common" :style="{ '--items-per-row': itemsPerRow }">
        <div 
          v-for="room in rooms" 
          :key="room.room_id" 
          class="card-shadow-wrapper"
          :class="{ 'hover-paused': isScrolling }"
          @click="goToPlayer(room.room_id)"
        >
            <div class="streamer-card-common">
              <div class="card-preview-common">
                <div class="image-wrapper-frame">
                  <SmoothImage 
                    :src="room.room_cover || ''" 
                    :alt="room.title" 
                    class="preview-image-common" 
                  />
                  <div class="card-overlay-gradient"></div>
                  <span class="viewers-count-overlay-common">
                    <svg class="viewers-icon-common" width="12" height="12" viewBox="0 0 24 24" fill="currentColor"><path d="M12 4.5C7 4.5 2.73 7.61 1 12c1.73 4.39 6 7.5 11 7.5s9.27-3.11 11-7.5c-1.73-4.39-6-7.5-11-7.5zM12 17c-2.76 0-5-2.24-5-5s2.24-5 5-5 5 2.24 5 5-2.24 5-5 5zm0-8c-1.66 0-3 1.34-3 3s1.34 3 3 3 3-1.34 3-3-1.34-3-3-3z"/></svg>
                    {{ room.viewer_count_str || '0' }} 
                  </span>
                </div>
              </div>
              <div class="card-info-footer-common">
                <div class="avatar-container">
                  <SmoothImage 
                    :src="room.avatar || ''" 
                    :alt="room.nickname" 
                    class="streamer-avatar-common" 
                  />
                </div>
                <div class="text-details-common">
                  <h3 class="room-title-common" :title="room.title">{{ room.title }}</h3>
                  <div class="nickname-row">
                    <span class="nickname-common">{{ room.nickname || '主播' }}</span>
                  </div>
                </div>
              </div>
            </div>
        </div>
      </div>
    </div>
    <div v-if="isLoadingMore" class="loading-more-indicator">
      <LoadingDots />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch, onMounted, onBeforeUnmount, nextTick, computed } from 'vue';
import { useRouter } from 'vue-router';
import { useResizeObserver } from '@vueuse/core';
import type { CategorySelectedEvent } from '../../platforms/common/categoryTypes'
import { useHuyaLiveRooms } from './composables/useHuyaLiveRooms.ts'
import { useDouyinLiveRooms } from './composables/useDouyinLiveRooms.ts'
import { useBilibiliLiveRooms } from './composables/useBilibiliLiveRooms.ts'
import { useDouyuLiveRooms } from './composables/useDouyuLiveRooms.ts'
import { useMultiRoomStore } from '../../stores/multiRoom'
import { Platform } from '../../platforms/common/types'
import SmoothImage from '../Common/SmoothImage.vue'
import LoadingDots from '../Common/LoadingDots.vue'

type DouyuCategorySelection = {
  type: 'cate2' | 'cate3';
  id: string;
  name?: string;
};

const props = defineProps<{
  selectedCategory?: CategorySelectedEvent | null;
  categoriesData?: any[]; 
  playerRouteName?: string; 
  platformName?: 'huya' | 'douyin' | 'douyu' | 'bilibili' | string; 
  defaultPageSize?: number; 
  douyuCategory?: DouyuCategorySelection | null;
}>();

const router = useRouter();
const multiRoomStore = useMultiRoomStore();
const scrollComponentRef = ref<any | null>(null);
const containerWidth = ref(0);
const categoryHref = computed(() => props.selectedCategory?.cate2Href || null);
const platformName = computed(() => props.platformName ?? 'huya');
const douyuCategoryId = computed(() => props.douyuCategory?.id || null);
const douyuCategoryType = computed(() => props.douyuCategory?.type || null);
const hasCategory = computed(() => {
  if (platformName.value === 'douyu') return !!douyuCategoryId.value;
  return !!categoryHref.value;
});

const resolvedSubcategoryId = computed(() => {
  const href = props.selectedCategory?.cate2Href;
  const data = props.categoriesData;
  if (!href || !Array.isArray(data)) return null;
  for (const c1 of data) {
    if (!Array.isArray(c1.subcategories)) continue;
    const c2 = c1.subcategories.find((s: any) => s.href === href);
    if (c2 && (c2.id || c2.gid)) return String(c2.id ?? c2.gid);
  }
  return null;
});

const douyinPartition = computed(() => { 
  const href = props.selectedCategory?.cate2Href;
  if (!href) return null;
  const parts = href.split('_');
  return parts.length >= 1 ? parts[parts.length - 1] : null;
});
const douyinPartitionType = computed(() => { 
  const href = props.selectedCategory?.cate2Href;
  if (!href) return null;
  const parts = href.split('_');
  return parts.length >= 2 ? parts[parts.length - 2] : null;
});

const resolvedParentCategoryId = computed(() => {
  const href = props.selectedCategory?.cate2Href;
  const data = props.categoriesData;
  if (!href || !Array.isArray(data)) return null;
  for (const c1 of data) {
    if (!Array.isArray(c1.subcategories)) continue;
    const c2 = c1.subcategories.find((s: any) => s.href === href);
    if (c2 && (c2.parent_id || c2.parentId || c1.id)) return String(c2.parent_id ?? c2.parentId ?? c1.id);
  }
  return null;
});

const huyaComposable = useHuyaLiveRooms(resolvedSubcategoryId, { defaultPageSize: props.defaultPageSize ?? 120 });
const douyinComposable = useDouyinLiveRooms(douyinPartition, douyinPartitionType);
const bilibiliComposable = useBilibiliLiveRooms(resolvedSubcategoryId, resolvedParentCategoryId);
const douyuComposable = useDouyuLiveRooms(douyuCategoryType, douyuCategoryId);

const selectedComposable = computed(() => {
  if (platformName.value === 'douyin') return douyinComposable;
  if (platformName.value === 'bilibili') return bilibiliComposable;
  if (platformName.value === 'douyu') return douyuComposable;
  return huyaComposable;
});

const rooms = computed(() => selectedComposable.value.rooms.value);
const isLoading = computed(() => selectedComposable.value.isLoading.value);
const isLoadingMore = computed(() => selectedComposable.value.isLoadingMore.value);
const hasMore = computed(() => selectedComposable.value.hasMore.value);
const loadInitialRooms = () => selectedComposable.value.loadInitialRooms();
const loadMoreRooms = () => selectedComposable.value.loadMoreRooms();

let resizeRaf: number | null = null;
let ensureTimer: number | null = null;
const minCardWidth = 180;
const gridGap = 18;

const scrollElement = computed<HTMLElement | null>(() => {
  const el = scrollComponentRef.value as any;
  return (el?.$el ?? el) as HTMLElement | null;
});

useResizeObserver(scrollElement, (entries) => {
  const entry = entries[0];
  if (entry) containerWidth.value = entry.contentRect.width;
});

const itemsPerRow = computed(() => {
  const width = containerWidth.value || minCardWidth;
  return Math.max(1, Math.floor((width + gridGap) / (minCardWidth + gridGap)));
});


const isScrolling = ref(false);
let scrollStopTimer: number | null = null;

const handleScrollerScroll = (event: Event) => {
  const target = event.target as HTMLElement | null;
  if (!target || !hasMore.value || isLoading.value || isLoadingMore.value) return;
  const nearBottom = target.scrollTop + target.clientHeight >= target.scrollHeight - 260;
  if (nearBottom) loadMoreRooms();
  isScrolling.value = true;
  if (scrollStopTimer !== null) window.clearTimeout(scrollStopTimer);
  scrollStopTimer = window.setTimeout(() => {
    isScrolling.value = false;
    scrollStopTimer = null;
  }, 120);
};

const maybeEnsureContentFillsViewport = () => {
  const rootEl = scrollElement.value;
  if (!rootEl || !hasMore.value || isLoading.value || isLoadingMore.value) return;
  const needsMore = rootEl.scrollHeight - rootEl.clientHeight <= 100;
  if (needsMore) loadMoreRooms();
};

const scheduleEnsureContentFill = () => {
  if (typeof window === 'undefined') return;
  if (resizeRaf) cancelAnimationFrame(resizeRaf);
  if (ensureTimer) {
    window.clearTimeout(ensureTimer);
    ensureTimer = null;
  }
  resizeRaf = window.requestAnimationFrame(() => {
    resizeRaf = null;
    nextTick(() => maybeEnsureContentFillsViewport());
  });
  ensureTimer = window.setTimeout(() => {
    ensureTimer = null;
    maybeEnsureContentFillsViewport();
  }, 160);
};

onMounted(() => {
  if (typeof window !== 'undefined') window.addEventListener('resize', scheduleEnsureContentFill);
  nextTick(() => {
    scheduleEnsureContentFill();
  });
});

onBeforeUnmount(() => {
  if (typeof window !== 'undefined') window.removeEventListener('resize', scheduleEnsureContentFill);
  if (resizeRaf) cancelAnimationFrame(resizeRaf);
  if (ensureTimer) window.clearTimeout(ensureTimer);
  if (scrollStopTimer !== null) window.clearTimeout(scrollStopTimer);
});

const lastSelectionKey = ref<string | null>(null);

const getSelectionKey = (category: CategorySelectedEvent | null | undefined): string | null => {
  if (platformName.value === 'douyu') {
    if (!douyuCategoryId.value || !douyuCategoryType.value) return `${platformName.value}:none`;
    return `${platformName.value}:${douyuCategoryType.value}:${douyuCategoryId.value}`;
  }
  if (!category?.cate2Href) return `${platformName.value}:none`;
  return `${platformName.value}:${category.cate2Href}`;
};

watch([() => props.selectedCategory, () => props.douyuCategory, platformName], ([newCategory]) => {
  const nextKey = getSelectionKey(newCategory ?? null);
  const isSameSelection = nextKey === lastSelectionKey.value;
  lastSelectionKey.value = nextKey;

  if (isSameSelection && rooms.value.length > 0) {
    nextTick(() => {
      scheduleEnsureContentFill();
    });
    return;
  }

  if (platformName.value === 'douyu') {
    if (douyuCategoryId.value) loadInitialRooms();
    else douyuComposable.rooms.value = [];
  } else if (newCategory?.cate2Href) {
    loadInitialRooms();
  } else {
    if (platformName.value === 'douyin') douyinComposable.rooms.value = [];
    else if (platformName.value === 'bilibili') bilibiliComposable.rooms.value = [];
    else huyaComposable.rooms.value = [];
  }
  nextTick(() => {
    scheduleEnsureContentFill();
  });
}, { immediate: true, deep: true });

watch([rooms, isLoading, isLoadingMore], () => {
  if (!isLoading.value && !isLoadingMore.value) scheduleEnsureContentFill();
});

const platformEnumMap: Record<string, Platform> = {
  douyu: Platform.DOUYU,
  douyin: Platform.DOUYIN,
  huya: Platform.HUYA,
  bilibili: Platform.BILIBILI,
};

const goToPlayer = (roomId: string) => {
  if (!roomId) return;
  const platform = platformEnumMap[platformName.value];
  if (platform) {
    multiRoomStore.openRoom(platform, roomId);
    router.push({ name: 'multiPlayer' });
  } else if (props.playerRouteName) {
    router.push({ name: props.playerRouteName, params: { roomId } });
  }
};
</script>

<style scoped>
.common-live-list-container {
  display: flex;
  flex-direction: column;
  height: 100%;
  width: 100%;
  box-sizing: border-box;
  background: transparent;
  overflow: hidden;
}

.loading-initial-state,
.no-streamers-state,
.no-category-state,
.loading-more-indicator {
  display: flex;
  flex-direction: column;
  justify-content: center;
  align-items: center;
  padding: 24px;
  color: var(--secondary-text);
  gap: 12px;
}

.loading-initial-state { flex-grow: 1; }

.live-grid-scroll-area {
  flex-grow: 1;
  overflow-y: auto;
  padding: 6px 8px;
  --card-radius: 12px;
}

.live-grid-scroll-area::-webkit-scrollbar {
  width: 5px;
}

.live-grid-scroll-area::-webkit-scrollbar-thumb {
  background: var(--glass-border);
  border-radius: 10px;
}

.live-grid-common {
  display: grid;
  grid-template-columns: repeat(var(--items-per-row, 1), minmax(0, 1fr));
  gap: 10px 10px;
  margin-bottom: 10px;
}

.card-shadow-wrapper {
  position: relative;
  transition: transform 0.2s ease;
  will-change: transform;
}

.card-shadow-wrapper:hover {
  transform: translateY(-4px);
}

.card-shadow-wrapper.hover-paused,
.card-shadow-wrapper.hover-paused:hover {
  transform: none;
}

.streamer-card-common {
  background: var(--hover-bg);
  border-radius: var(--card-radius);
  display: flex;
  flex-direction: column;
  cursor: pointer;
  border: 1px solid var(--glass-border);
  transition: background-color 0.2s ease, border-color 0.2s ease;
  padding: 0;
}

.streamer-card-common:hover {
  background: var(--hover-bg);
}

:global(:root:not([data-theme="light"])) .streamer-card-common {
  background: rgba(255, 255, 255, 0.08);
  border-color: rgba(255, 255, 255, 0.12);
}

.card-preview-common {
  width: 100%;
  aspect-ratio: 16 / 8.5;
  position: relative;
  border-radius: var(--card-radius) var(--card-radius) 0 0;
  overflow: hidden;
}

.image-wrapper-frame {
  width: 100%;
  height: 100%;
  position: relative;
}

.preview-image-common {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.card-overlay-gradient {
  position: absolute;
  inset: 0;
  background: linear-gradient(to top, rgba(0,0,0,0.5) 0%, transparent 40%);
  pointer-events: none;
}

.viewers-count-overlay-common {
  position: absolute;
  top: 8px;
  right: 10px;
  background: rgba(0, 0, 0, 0.4);
  backdrop-filter: blur(12px);
  -webkit-backdrop-filter: blur(12px);
  color: white;
  padding: 2px 8px;
  border-radius: 100px;
  font-size: 9px;
  font-weight: 700;
  display: flex;
  align-items: center;
  gap: 3px;
  border: 1px solid rgba(255, 255, 255, 0.1);
}

.card-info-footer-common {
  display: flex;
  padding: 12px 12px 14px;
  gap: 10px;
  align-items: center;
}

.avatar-container {
  flex-shrink: 0;
}

.streamer-avatar-common {
  width: 38px;
  height: 38px;
  border-radius: 50%;
  object-fit: cover;
  border: 1.5px solid var(--border-color);
  transition: border-color 0.3s ease;
}

.streamer-card-common:hover .streamer-avatar-common {
  border-color: var(--accent-color);
}

.text-details-common {
  flex: 1;
  min-width: 0;
}

.room-title-common {
  font-size: 14px;
  font-weight: 700;
  color: var(--primary-text);
  margin-bottom: 3px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  line-height: 1.2;
}

.nickname-common {
  font-size: 12px;
  color: var(--secondary-text);
  font-weight: 600;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  display: block;
}

.nickname-row {
  display: flex;
  align-items: center;
  min-width: 0;
}

.loading-spinner, .mini-spinner {
  width: 36px;
  height: 36px;
  border: 4px solid var(--border-color);
  border-top-color: var(--accent-color);
  border-radius: 50%;
  animation: spin 1s cubic-bezier(0.4, 0, 0.2, 1) infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

.scroll-sentinel {
  height: 60px;
}
</style>
