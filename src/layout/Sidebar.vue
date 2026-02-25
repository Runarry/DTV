<template>
  <aside
    class="sidebar-shell"
    :style="{
      width: isCollapsed ? 'var(--sidebar-hidden-width)' : 'var(--sidebar-width)',
      minWidth: isCollapsed ? 'var(--sidebar-hidden-width)' : 'var(--sidebar-width)',
      height: '100%',
      display: 'flex',
      flexDirection: 'column',
      overflow: 'hidden',
      pointerEvents: isCollapsed ? 'none' : 'auto',
      transition: 'none',
    }"
  >
    <div v-show="!isCollapsed" class="sidebar-body">
      <FollowList
        :followedAnchors="followedAnchors"
        :show-collapse-trigger="!isCollapsed"
        @toggleSidebar="emit('toggle')"
        @selectAnchor="emit('select-anchor', $event)"
        @unfollow="emit('unfollow', $event)"
        @reorderList="emit('reorder-list', $event)"
      />
    </div>
  </aside>
</template>

<script setup lang="ts">
import type { FollowedStreamer, Platform } from '../platforms/common/types';
import FollowList from '../components/FollowsList/index.vue';

defineProps<{
  isCollapsed: boolean;
  followedAnchors: FollowedStreamer[];
}>();

const emit = defineEmits<{
  (event: 'toggle'): void;
  (event: 'select-anchor', streamer: FollowedStreamer): void;
  (event: 'unfollow', payload: { platform: Platform; id: string } | string): void;
  (event: 'reorder-list', newList: FollowedStreamer[]): void;
}>();
</script>

<style scoped>
.sidebar-shell {
  z-index: 100;
  background-color: var(--bg-primary);
}

.sidebar-body {
  flex: 1;
  min-height: 0;
  overflow: visible;
  padding: 8px;
}
</style>
