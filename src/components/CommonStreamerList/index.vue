<template>
  <div class="common-live-list-container">
    <div v-if="isLoading && rooms.length === 0" class="loading-initial-state">
      <div class="loading-spinner"></div>
      <p>正在加载主播列表...</p>
    </div>
    <div v-else-if="!isLoading && rooms.length === 0 && categoryHref" class="no-streamers-state">
      <p>分类下暂无主播</p>
    </div>
    <div v-else-if="!categoryHref && !isLoading" class="no-category-state">
       <p>请选择一个分类开始浏览</p>
    </div>

    <div class="live-grid-scroll-area" ref="scrollComponentRef">
      <div class="live-grid-common">
        <motion.div 
          v-for="(room, index) in rooms" 
          :key="room.room_id + '-' + index" 
          class="card-shadow-wrapper"
          @click="goToPlayer(room.room_id)"
          :initial="{ opacity: 1, scale: 1 }"
          :animate="{ opacity: 1, scale: 1 }"
          whileHover="{ scale: 1.02, transition: { duration: 0.2 } }"
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
        </motion.div>
      </div>
      <div ref="sentinelRef" class="scroll-sentinel"></div>
    </div>
    <div v-if="isLoadingMore" class="loading-more-indicator">
      <div class="mini-spinner"></div>
      <p>努力加载中...</p>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch, onMounted, onBeforeUnmount, nextTick, computed } from 'vue';
import { useRouter } from 'vue-router';
import { motion } from 'motion-v';
import type { CategorySelectedEvent } from '../../platforms/common/categoryTypes'
import { useHuyaLiveRooms } from './composables/useHuyaLiveRooms'
import { useDouyinLiveRooms } from './composables/useDouyinLiveRooms'
import { useBilibiliLiveRooms } from './composables/useBilibiliLiveRooms'
import SmoothImage from '../Common/SmoothImage.vue'

const props = defineProps<{
  selectedCategory: CategorySelectedEvent | null;
  categoriesData?: any[]; 
  playerRouteName?: string; 
  platformName?: 'huya' | 'douyin' | 'douyu' | 'bilibili' | string; 
  defaultPageSize?: number; 
}>();

const router = useRouter();
const scrollComponentRef = ref<HTMLElement | null>(null);
const sentinelRef = ref<HTMLElement | null>(null);
const categoryHref = computed(() => props.selectedCategory?.cate2Href || null);
const platformName = computed(() => props.platformName ?? 'huya');

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

const selectedComposable = computed(() => {
  if (platformName.value === 'douyin') return douyinComposable;
  if (platformName.value === 'bilibili') return bilibiliComposable;
  return huyaComposable;
});

const rooms = computed(() => selectedComposable.value.rooms.value);
const isLoading = computed(() => selectedComposable.value.isLoading.value);
const isLoadingMore = computed(() => selectedComposable.value.isLoadingMore.value);
const hasMore = computed(() => selectedComposable.value.hasMore.value);
const loadInitialRooms = () => selectedComposable.value.loadInitialRooms();
const loadMoreRooms = () => selectedComposable.value.loadMoreRooms();

let observer: IntersectionObserver | null = null;
let resizeRaf: number | null = null;

const setupIntersectionObserver = () => {
  if (observer) observer.disconnect();
  const options = { root: scrollComponentRef.value, rootMargin: '200px', threshold: 0.1 };

  observer = new IntersectionObserver((entries) => {
    const entry = entries[0];
    if (entry.isIntersecting && hasMore.value && !isLoading.value && !isLoadingMore.value) {
      loadMoreRooms();
    }
  }, options);

  if (sentinelRef.value) observer.observe(sentinelRef.value);
};

const maybeEnsureContentFillsViewport = () => {
  const rootEl = scrollComponentRef.value;
  if (!rootEl || !hasMore.value || isLoading.value || isLoadingMore.value) return;
  const needsMore = rootEl.scrollHeight - rootEl.clientHeight <= 100;
  if (needsMore) loadMoreRooms();
};

const scheduleEnsureContentFill = () => {
  if (typeof window === 'undefined') return;
  if (resizeRaf) cancelAnimationFrame(resizeRaf);
  resizeRaf = window.requestAnimationFrame(() => {
    resizeRaf = null;
    nextTick(() => maybeEnsureContentFillsViewport());
  });
};

onMounted(() => {
  if (typeof window !== 'undefined') window.addEventListener('resize', scheduleEnsureContentFill);
  nextTick(() => {
    setupIntersectionObserver();
    scheduleEnsureContentFill();
  });
});

onBeforeUnmount(() => {
  if (observer) observer.disconnect();
  if (typeof window !== 'undefined') window.removeEventListener('resize', scheduleEnsureContentFill);
  if (resizeRaf) cancelAnimationFrame(resizeRaf);
});

watch(() => props.selectedCategory, (newCategory) => {
  if (newCategory?.cate2Href) loadInitialRooms();
  else {
    if (platformName.value === 'douyin') douyinComposable.rooms.value = [];
    else if (platformName.value === 'bilibili') bilibiliComposable.rooms.value = [];
    else huyaComposable.rooms.value = [];
  }
  nextTick(() => {
    if (scrollComponentRef.value && sentinelRef.value) setupIntersectionObserver();
    scheduleEnsureContentFill();
  });
}, { immediate: true, deep: true });

watch([rooms, isLoading, isLoadingMore], () => {
  if (!isLoading.value && !isLoadingMore.value) scheduleEnsureContentFill();
});

const goToPlayer = (roomId: string) => {
  if (roomId && props.playerRouteName) {
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
  padding: 60px;
  color: var(--secondary-text);
  gap: 16px;
}

.loading-initial-state { flex-grow: 1; }

.live-grid-scroll-area {
  flex-grow: 1;
  overflow-y: auto;
  padding: 32px;
  --squircle-radius: 1%;
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
  grid-template-columns: repeat(auto-fill, minmax(180px, 1fr));
  gap: 20px;
}

.card-shadow-wrapper {
  position: relative;
  transition: all 0.5s cubic-bezier(0.16, 1, 0.3, 1);
}

.card-shadow-wrapper:hover {
  transform: translateY(-6px);
  filter: drop-shadow(0 16px 32px rgba(0, 0, 0, 0.4));
}

.streamer-card-common {
  background: var(--glass-bg);
  backdrop-filter: var(--glass-blur);
  -webkit-backdrop-filter: var(--glass-blur);
  clip-path: url(#squircle-clip);
  border-radius: var(--squircle-radius);
  overflow: hidden;
  display: flex;
  flex-direction: column;
  cursor: pointer;
  border: 1px solid var(--glass-border);
  transition: all 0.5s cubic-bezier(0.16, 1, 0.3, 1);
  padding: 10px;
}

.streamer-card-common:hover {
  background: var(--hover-bg);
}

.card-preview-common {
  width: 100%;
  aspect-ratio: 16 / 10;
  position: relative;
  overflow: hidden;
}

.image-wrapper-frame {
  width: 100%;
  height: 100%;
  clip-path: url(#squircle-clip);
  border-radius: var(--squircle-radius);
  overflow: hidden;
  position: relative;
}

.preview-image-common {
  width: 100%;
  height: 100%;
  object-fit: cover;
  transition: transform 1s cubic-bezier(0.16, 1, 0.3, 1);
}

.streamer-card-common:hover .preview-image-common {
  transform: scale(1.1);
}

.card-overlay-gradient {
  position: absolute;
  inset: 0;
  background: linear-gradient(to top, rgba(0,0,0,0.5) 0%, transparent 40%);
  pointer-events: none;
}

.viewers-count-overlay-common {
  position: absolute;
  bottom: 8px;
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
  padding: 10px 8px 4px 8px;
  gap: 10px;
  align-items: center;
}

.avatar-container {
  flex-shrink: 0;
}

.streamer-avatar-common {
  width: 32px;
  height: 32px;
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
  font-size: 13px;
  font-weight: 700;
  color: var(--primary-text);
  margin-bottom: 1px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  line-height: 1.2;
}

.nickname-common {
  font-size: 11px;
  color: var(--secondary-text);
  font-weight: 600;
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
