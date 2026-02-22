<template>
  <div v-if="store.roomCount > 1" class="room-tab-bar">
    <div class="tab-list">
      <div
        v-for="room in store.rooms"
        :key="room.id"
        class="tab-item"
        :class="{ active: room.id === store.activeRoomId }"
        @click="store.setActiveRoom(room.id)"
      >
        <span class="tab-platform-badge" :class="room.platform.toLowerCase()">
          {{ platformLabel(room.platform) }}
        </span>
        <span class="tab-name" :title="room.anchorName || room.roomId">
          {{ room.anchorName || room.roomId }}
        </span>
        <button
          class="tab-mute-btn"
          :title="room.isMuted ? '取消静音' : '静音'"
          @click.stop="store.toggleMute(room.id)"
        >
          <svg v-if="room.isMuted" xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <polygon points="11 5 6 9 2 9 2 15 6 15 11 19 11 5"></polygon>
            <line x1="23" y1="9" x2="17" y2="15"></line>
            <line x1="17" y1="9" x2="23" y2="15"></line>
          </svg>
          <svg v-else xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <polygon points="11 5 6 9 2 9 2 15 6 15 11 19 11 5"></polygon>
            <path d="M19.07 4.93a10 10 0 0 1 0 14.14"></path>
            <path d="M15.54 8.46a5 5 0 0 1 0 7.07"></path>
          </svg>
        </button>
        <button
          class="tab-close-btn"
          title="关闭"
          @click.stop="$emit('close-room', room.id)"
        >
          <svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
            <line x1="18" y1="6" x2="6" y2="18"></line>
            <line x1="6" y1="6" x2="18" y2="18"></line>
          </svg>
        </button>
      </div>
    </div>
    <div class="tab-actions">
      <button
        class="layout-toggle-btn"
        :title="store.layoutMode === 'split' ? '切换为单屏' : '切换为分屏'"
        @click="store.setLayoutMode(store.layoutMode === 'split' ? 'single' : 'split')"
      >
        <svg v-if="store.layoutMode === 'split'" xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <rect x="3" y="3" width="18" height="18" rx="2" ry="2"></rect>
          <line x1="12" y1="3" x2="12" y2="21"></line>
        </svg>
        <svg v-else xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <rect x="3" y="3" width="18" height="18" rx="2" ry="2"></rect>
        </svg>
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { useMultiRoomStore } from '../../stores/multiRoom';
import { Platform } from '../../platforms/common/types';

defineEmits<{
  (e: 'close-room', id: string): void;
}>();

const store = useMultiRoomStore();

const platformLabel = (platform: Platform) => {
  switch (platform) {
    case Platform.DOUYU: return '斗鱼';
    case Platform.DOUYIN: return '抖音';
    case Platform.HUYA: return '虎牙';
    case Platform.BILIBILI: return 'B站';
    default: return platform;
  }
};
</script>

<style scoped>
.room-tab-bar {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 4px 8px;
  background: var(--bg-secondary, #1a1a2e);
  border-bottom: 1px solid var(--glass-border, rgba(255, 255, 255, 0.1));
  min-height: 36px;
  flex-shrink: 0;
}

.tab-list {
  display: flex;
  gap: 4px;
  flex: 1;
  overflow-x: auto;
  scrollbar-width: none;
}

.tab-list::-webkit-scrollbar {
  display: none;
}

.tab-item {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 4px 8px;
  border-radius: 6px;
  cursor: pointer;
  background: transparent;
  transition: background-color 0.15s ease;
  flex-shrink: 0;
  max-width: 200px;
}

.tab-item:hover {
  background: var(--hover-bg, rgba(255, 255, 255, 0.08));
}

.tab-item.active {
  background: var(--accent-color, #5c16c5);
  color: white;
}

.tab-platform-badge {
  font-size: 10px;
  font-weight: 700;
  padding: 1px 4px;
  border-radius: 3px;
  text-transform: uppercase;
  flex-shrink: 0;
}

.tab-platform-badge.douyu { background: #ff6600; color: white; }
.tab-platform-badge.douyin { background: #111; color: white; }
.tab-platform-badge.huya { background: #f4a700; color: #000; }
.tab-platform-badge.bilibili { background: #fb7299; color: white; }

.tab-name {
  font-size: 12px;
  font-weight: 600;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  color: var(--primary-text, #e0e0e0);
}

.tab-item.active .tab-name {
  color: white;
}

.tab-mute-btn,
.tab-close-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 20px;
  height: 20px;
  border: none;
  background: transparent;
  color: var(--secondary-text, #999);
  cursor: pointer;
  border-radius: 4px;
  flex-shrink: 0;
  padding: 0;
  transition: background-color 0.15s ease, color 0.15s ease;
}

.tab-mute-btn:hover,
.tab-close-btn:hover {
  background: rgba(255, 255, 255, 0.15);
  color: var(--primary-text, #e0e0e0);
}

.tab-item.active .tab-mute-btn,
.tab-item.active .tab-close-btn {
  color: rgba(255, 255, 255, 0.7);
}

.tab-item.active .tab-mute-btn:hover,
.tab-item.active .tab-close-btn:hover {
  color: white;
  background: rgba(255, 255, 255, 0.2);
}

.tab-actions {
  display: flex;
  align-items: center;
  gap: 4px;
  flex-shrink: 0;
}

.layout-toggle-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  border: none;
  background: transparent;
  color: var(--secondary-text, #999);
  cursor: pointer;
  border-radius: 6px;
  padding: 0;
  transition: background-color 0.15s ease, color 0.15s ease;
}

.layout-toggle-btn:hover {
  background: var(--hover-bg, rgba(255, 255, 255, 0.08));
  color: var(--primary-text, #e0e0e0);
}
</style>
