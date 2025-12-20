<template>
  <aside class="app-sidebar" data-tauri-drag-region>
    <div class="sidebar-header" data-tauri-drag-region>
      <!-- Logo removed for density -->
    </div>

    <nav 
      class="navigation" 
      ref="navListRef" 
      data-tauri-drag-region
    >
      <div 
        v-if="highlight.visible"
        class="nav-shared-highlight"
        aria-hidden="true"
        :style="highlightStyles"
      ></div>

      <router-link 
        v-for="item in navItems" 
        :key="item.key"
        :to="item.path" 
        custom
        v-slot="{ href, navigate, isActive }"
      >
        <motion.a
          :href="href"
          class="nav-item"
          :class="{ 'is-active': isActive }"
          @click="(event) => handleNavClick(event, navigate, item.path)"
          :ref="(el) => setNavItemRef(item.key, el)"
          whileHover="{ scale: 1.02 }"
          whileTap="{ scale: 0.98 }"
        >
          <div class="nav-content">
            <span class="nav-icon">
              <img :src="item.logo" :alt="item.name" />
            </span>
            <span class="nav-name">{{ item.name }}</span>
          </div>
          <motion.div 
            v-if="isActive"
            class="active-dot"
            layoutId="activeDot"
          ></motion.div>
        </motion.a>
      </router-link>
    </nav>
    
    <div class="follow-list-wrapper">
      <FollowList 
        :followedAnchors="sortedFollowedAnchors"
        @selectAnchor="handleSelectAnchor"
        @unfollow="handleUnfollow"
        @reorderList="handleReorderList"
        class="follow-list-component"
      />
    </div>
  </aside>
</template>

<script setup lang="ts">
import { ref, computed, reactive, watch, nextTick, onMounted, onBeforeUnmount } from 'vue';
import { useRouter, useRoute } from 'vue-router';
import { motion } from 'motion-v';
import type { FollowedStreamer } from '../platforms/common/types';
import FollowList from '../components/FollowsList/index.vue';
import douyuLogo from '../assets/douyu.webp';
import douyinLogo from '../assets/douyin.webp';
import huyaLogo from '../assets/huya.webp';
import bilibiliLogo from '../assets/bilibili.webp';

const emit = defineEmits(['selectAnchor', 'unfollow', 'navigate', 'reorderList']);
const router = useRouter();
const route = useRoute();

type PlatformKey = 'douyu' | 'douyin' | 'huya' | 'bilibili';

type NavItem = {
  key: PlatformKey;
  name: string;
  path: string;
  logo: string;
};

const navItems = ref<NavItem[]>([
  { key: 'douyu', name: '斗鱼', path: '/', logo: douyuLogo },
  { key: 'douyin', name: '抖音', path: '/douyin', logo: douyinLogo },
  { key: 'huya', name: '虎牙', path: '/huya', logo: huyaLogo },
  { key: 'bilibili', name: 'B站', path: '/bilibili', logo: bilibiliLogo },
]);

const props = withDefaults(defineProps<{
  followedAnchors?: FollowedStreamer[];
}>(), {
  followedAnchors: () => []
});

const navListRef = ref<HTMLElement | null>(null);
const navItemRefs = new Map<PlatformKey, HTMLElement>();

const highlight = reactive({
  offset: 0,
  height: 0,
  visible: false,
});

const activePlatformKey = computed<PlatformKey | null>(() => {
  const match = navItems.value.find(item => item.path === route.path);
  return match?.key ?? null;
});

const highlightStyles = computed(() => ({
  transform: `translateY(${highlight.offset}px)`,
  height: `${highlight.height}px`,
  opacity: highlight.visible ? 1 : 0,
}));

const updateHighlight = () => {
  nextTick(() => {
    const key = activePlatformKey.value;
    if (!key) {
      highlight.visible = false;
      return;
    }
    const el = navItemRefs.get(key);
    const container = navListRef.value;
    if (!el || !container) {
      highlight.visible = false;
      return;
    }
    highlight.offset = el.offsetTop;
    highlight.height = el.offsetHeight;
    highlight.visible = true;
  });
};

const setNavItemRef = (key: PlatformKey, el: any) => {
  if (!el) {
    navItemRefs.delete(key);
    return;
  }
  const element = el.$el || el;
  if (element instanceof HTMLElement) {
    navItemRefs.set(key, element);
    if (key === activePlatformKey.value) {
      updateHighlight();
    }
  }
};

const handleNavClick = (event: MouseEvent, navigate: (e?: MouseEvent) => void, path: string) => {
  navigate(event);
  emit('navigate', path);
};

watch(() => route.path, () => updateHighlight(), { immediate: true });

onMounted(() => {
  window.addEventListener('resize', updateHighlight);
  updateHighlight();
});

onBeforeUnmount(() => {
  window.removeEventListener('resize', updateHighlight);
});

const customSortedAnchors = ref<FollowedStreamer[]>([]);

const sortedFollowedAnchors = computed(() => {
  if (!props.followedAnchors?.length) return [];
  const toKey = (a: FollowedStreamer) => `${a.platform}:${a.id}`;
  const currentKeys = new Set(props.followedAnchors.map(toKey));
  const customSortedIsValid = customSortedAnchors.value.length > 0 && 
    customSortedAnchors.value.length === props.followedAnchors.length && 
    customSortedAnchors.value.every(customAnchor => currentKeys.has(toKey(customAnchor)));
  let baseOrder: FollowedStreamer[];
  if (customSortedIsValid) {
    const propsMap = new Map(props.followedAnchors.map(anchor => [toKey(anchor), anchor]));
    baseOrder = customSortedAnchors.value
      .map(customAnchor => propsMap.get(toKey(customAnchor)))
      .filter(Boolean) as FollowedStreamer[];
  } else {
    baseOrder = [...props.followedAnchors];
  }
  const live: FollowedStreamer[] = [];
  const looping: FollowedStreamer[] = [];
  const rest: FollowedStreamer[] = [];
  baseOrder.forEach(anchor => {
    if (anchor.liveStatus === 'LIVE') live.push(anchor);
    else if (anchor.liveStatus === 'REPLAY') looping.push(anchor);
    else rest.push(anchor);
  });
  return [...live, ...looping, ...rest];
});

const handleSelectAnchor = (anchor: FollowedStreamer) => emit('selectAnchor', anchor);
const handleUnfollow = (payload: any) => emit('unfollow', payload);
const handleReorderList = (reorderedList: FollowedStreamer[]) => {
  customSortedAnchors.value = [...reorderedList];
  emit('reorderList', reorderedList);
};

defineExpose({ router });
</script>

<style scoped>
.app-sidebar {
  width: 280px;
  background: transparent;
  display: flex;
  flex-direction: column;
  padding: 24px 16px;
  gap: 24px;
  z-index: 100;
}

.sidebar-header {
  height: 48px;
  display: flex;
  align-items: center;
  padding: 0 16px;
  margin-bottom: 8px;
}

/* Redesign Nav Items */
.navigation {
  display: flex;
  flex-direction: column;
  gap: 12px;
  position: relative;
}

.nav-shared-highlight {
  position: absolute;
  left: 0;
  right: 0;
  background: var(--hover-bg);
  border-radius: var(--radius-md);
  transition: all 0.3s cubic-bezier(0.16, 1, 0.3, 1);
  pointer-events: none;
  z-index: 0;
}

.nav-item {
  position: relative;
  display: flex;
  align-items: center;
  padding: 14px 18px;
  border-radius: var(--radius-md);
  color: var(--secondary-text);
  text-decoration: none;
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  background: transparent;
  z-index: 1;
}

.nav-item:hover {
  color: var(--primary-text);
}

.nav-item.is-active {
  color: var(--accent-text);
  font-weight: 700;
  background: var(--accent-gradient);
  box-shadow: 0 10px 30px rgba(191, 255, 0, 0.2);
}

.nav-content {
  display: flex;
  align-items: center;
  gap: 16px;
}

.nav-icon {
  width: 26px;
  height: 26px;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.3s ease;
}

.nav-icon img {
  width: 100%;
  height: 100%;
  object-fit: contain;
  transition: all 0.3s ease;
}

.nav-item.is-active .nav-icon img,
.nav-item:hover .nav-icon img {
  filter: brightness(0) saturate(100%) invert(0);
}

/* In light mode, revert invert if needed, but for accent color text, usually dark text on bright green */
html[data-theme="light"] .nav-item.is-active .nav-icon img {
  filter: brightness(0.2);
}

.nav-name {
  font-size: 16px;
  letter-spacing: 0.3px;
}

.active-dot {
  display: none; /* Removed dot in favor of full pill active state */
}

.follow-list-wrapper {
  flex: 1;
  overflow: hidden;
  margin-top: 16px;
  padding: 0 2px;
  background: var(--glass-bg);
  backdrop-filter: var(--glass-blur);
  -webkit-backdrop-filter: var(--glass-blur);
  border-radius: var(--radius-lg);
  border: 1px solid var(--glass-border);
  box-shadow: var(--glass-shadow);
}

.follow-list-component {
  height: 100%;
  padding: 16px 8px;
}

:deep(.follow-list-component::-webkit-scrollbar) {
  width: 4px;
}
:deep(.follow-list-component::-webkit-scrollbar-thumb) {
  background: var(--glass-border);
  border-radius: 10px;
}
</style>
```
