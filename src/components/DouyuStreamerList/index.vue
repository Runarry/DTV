<template>
  <div class="live-list-container-infinite">
    <div v-if="isLoading && streamers.length === 0" class="loading-initial-state">
      <div class="skeleton-grid">
        <div v-for="n in 8" :key="n" class="skeleton-card">
          <div class="skeleton-thumb skeleton-shimmer"></div>
          <div class="skeleton-line skeleton-shimmer"></div>
          <div class="skeleton-line short skeleton-shimmer"></div>
        </div>
      </div>
    </div>
    <div v-else-if="!isLoading && streamers.length === 0 && props.categoryId" class="no-streamers-state">
      <p>分类下暂无主播</p>
    </div>
    <div v-else-if="!props.categoryId && !isLoading" class="no-category-state">
       <p>请选择一个分类开始浏览</p>
    </div>

    <div class="live-grid-scroll-area" ref="scrollComponentRef">
      <div class="live-grid-infinite">
        <motion.div 
          v-for="(streamer, index) in streamers" 
          :key="streamer.rid + '-' + index" 
          class="card-shadow-wrapper"
          @click="goToPlayer(streamer.rid)"
          :initial="{ opacity: 1, scale: 1 }"
          :animate="{ opacity: 1, scale: 1 }"
          whileHover="{ scale: 1.02, transition: { duration: 0.2 } }"
        >
          <div class="streamer-card-revised">
            <div class="card-preview-revised">
              <div class="image-wrapper-frame">
                <SmoothImage 
                  :src="streamer.roomSrc || ''" 
                  :alt="streamer.roomName" 
                  class="preview-image-revised" 
                />
                <div class="card-overlay-gradient"></div>
                <span class="viewers-count-overlay">
                  <svg class="viewers-icon-revised" width="12" height="12" viewBox="0 0 24 24" fill="currentColor"><path d="M12 4.5C7 4.5 2.73 7.61 1 12c1.73 4.39 6 7.5 11 7.5s9.27-3.11 11-7.5c-1.73-4.39-6-7.5-11-7.5zM12 17c-2.76 0-5-2.24-5-5s2.24-5 5-5 5 2.24 5 5-2.24 5-5 5zm0-8c-1.66 0-3 1.34-3 3s1.34 3 3 3 3-1.34 3-3-1.34-3-3-3z"/></svg>
                  {{ streamer.hn }} 
                </span>
              </div>
            </div>
            <div class="card-info-footer-revised">
              <div class="avatar-container">
                <SmoothImage 
                  :src="streamer.avatar || ''" 
                  :alt="streamer.nickname" 
                  class="streamer-avatar-revised" 
                />
              </div>
              <div class="text-details-revised">
                <h3 class="room-title-revised" :title="streamer.roomName">{{ streamer.roomName }}</h3>
                <p class="nickname-revised" :title="streamer.nickname">{{ streamer.nickname }}</p>
              </div>
            </div>
          </div>
        </motion.div>
      </div>
      <div ref="sentinelRef" class="scroll-sentinel"></div>
      <div v-if="isLoading && streamers.length > 0" class="loading-more-indicator">
        <div class="mini-spinner"></div>
        <p>努力加载中...</p>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch, onMounted, onBeforeUnmount, nextTick } from 'vue'
import { useRouter } from 'vue-router'
import { motion } from 'motion-v'
import { useLiveData } from './composables/useLiveData'
import SmoothImage from '../Common/SmoothImage.vue'

const props = defineProps<{
  categoryType: 'cate2' | 'cate3';
  categoryId: string;
  categoryName?: string;
}>()

const router = useRouter();
const scrollComponentRef = ref<HTMLElement | null>(null);
const sentinelRef = ref<HTMLElement | null>(null);

const { 
  streamers, 
  hasMore, 
  isLoading, 
  loadNextPage, 
  resetAndFetch 
} = useLiveData(); 

let observer: IntersectionObserver | null = null;

const setupIntersectionObserver = () => {
  if (observer) observer.disconnect();

  const options = {
    root: scrollComponentRef.value,
    rootMargin: '200px',
    threshold: 0.1 
  };

  observer = new IntersectionObserver((entries) => {
    const entry = entries[0];
    if (entry.isIntersecting && hasMore.value && !isLoading.value) {
      if (props.categoryType && props.categoryId) {
        loadNextPage(props.categoryType, props.categoryId);
      }
    }
  }, options);

  if (sentinelRef.value) {
    observer.observe(sentinelRef.value);
  }
};

onMounted(() => {
  nextTick(setupIntersectionObserver);
});

onBeforeUnmount(() => {
  if (observer) observer.disconnect();
});

watch(() => ({ type: props.categoryType, id: props.categoryId }), (newCategory) => {
  if (newCategory?.id) resetAndFetch(newCategory.type, newCategory.id);
  else {
    streamers.value = []; 
    hasMore.value = false; 
  }
  nextTick(() => {
    if (scrollComponentRef.value && sentinelRef.value) setupIntersectionObserver();
  });
}, { immediate: true, deep: true });

const goToPlayer = (roomId: string) => {
  if (roomId) router.push({ name: 'douyuPlayer', params: { roomId } });
}
</script>

<style scoped>
.live-list-container-infinite {
  display: flex;
  flex-direction: column;
  height: 100%;
  width: 100%;
  box-sizing: border-box;
  background-color: transparent;
  overflow: hidden; 
  --squircle-radius: 1%;
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
  padding: 12px 0;
}

.live-grid-scroll-area::-webkit-scrollbar {
  width: 6px;
}

.live-grid-scroll-area::-webkit-scrollbar-thumb {
  background: var(--border-color);
  border-radius: 10px;
}

.live-grid-infinite {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(180px, 1fr));
  gap: 10px;
}

.card-shadow-wrapper {
  position: relative;
  transition: all 0.5s cubic-bezier(0.16, 1, 0.3, 1);
}

.card-shadow-wrapper:hover {
  transform: translateY(-6px);
  filter: none;
}

.streamer-card-revised {
  background: var(--hover-bg);
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

.streamer-card-revised:hover {
  background: var(--hover-bg);
}

:global(:root:not([data-theme="light"])) .streamer-card-revised {
  background: rgba(255, 255, 255, 0.08);
  border-color: rgba(255, 255, 255, 0.12);
}

.card-preview-revised {
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

.preview-image-revised {
  width: 100%;
  height: 100%;
  object-fit: cover;
  transition: transform 1s cubic-bezier(0.16, 1, 0.3, 1);
}

.streamer-card-revised:hover .preview-image-revised {
  transform: scale(1.1);
}

.card-overlay-gradient {
  position: absolute;
  inset: 0;
  background: linear-gradient(to top, rgba(0,0,0,0.5) 0%, transparent 40%);
  pointer-events: none;
}

.viewers-count-overlay {
  position: absolute;
  bottom: 8px;
  right: 10px;
  background: rgba(0, 0, 0, 0.4);
  backdrop-filter: blur(12px);
  -webkit-backdrop-filter: blur(12px);
  color: var(--primary-text);
  padding: 2px 8px;
  border-radius: 100px;
  font-size: 9px;
  font-weight: 700;
  display: flex;
  align-items: center;
  gap: 3px;
  border: 1px solid rgba(255, 255, 255, 0.1);
}

.card-info-footer-revised {
  display: flex;
  align-items: center;
  padding: 0 8px;
  gap: 10px;
}

.avatar-container {
  flex-shrink: 0;
}

.streamer-avatar-revised {
  width: 32px;
  height: 32px;
  border-radius: 50%;
  object-fit: cover;
  border: 1.5px solid var(--border-color);
  transition: border-color 0.3s ease;
}

.streamer-card-revised:hover .streamer-avatar-revised {
  border-color: var(--accent-color);
}

.text-details-revised {
  flex: 1;
  min-width: 0;
}

.room-title-revised {
  font-size: 13px;
  font-weight: 700;
  color: var(--primary-text);
  margin-bottom: 1px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  letter-spacing: -0.1px;
  line-height: 1.2;
}

.nickname-revised {
  font-size: 11px;
  color: var(--secondary-text);
  font-weight: 600;
}

.mini-spinner {
  width: 20px;
  height: 20px;
  border: 2px solid var(--border-color);
  border-top-color: var(--accent-color);
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

.scroll-sentinel {
  height: 20px;
}

.skeleton-grid {
  width: 100%;
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(180px, 1fr));
  gap: 10px;
}

.skeleton-card {
  background: var(--glass-bg);
  border: 1px solid var(--glass-border);
  border-radius: var(--squircle-radius);
  padding: 10px;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.skeleton-thumb {
  width: 100%;
  aspect-ratio: 16 / 10;
  border-radius: var(--squircle-radius);
  background: var(--skeleton-bg);
}

.skeleton-line {
  height: 10px;
  border-radius: 999px;
  background: var(--skeleton-bg);
}

.skeleton-line.short {
  width: 60%;
}

.skeleton-shimmer {
  position: relative;
  overflow: hidden;
}

.skeleton-shimmer::after {
  content: '';
  position: absolute;
  inset: 0;
  background: linear-gradient(90deg, transparent 25%, rgba(255, 255, 255, 0.2) 50%, transparent 75%);
  transform: translateX(-100%);
  animation: skeleton-loading 1.2s ease-in-out infinite;
}

@keyframes skeleton-loading {
  100% {
    transform: translateX(100%);
  }
}
</style>
 
