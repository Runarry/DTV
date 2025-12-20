<template>
  <motion.div
    class="folder-item"
    :data-folder-id="folder.id"
    :class="{ 
      'is-dragging': isDragging, 
      'is-expanded': folder.expanded,
      'is-drag-over': isDragOver,
      'can-accept-drop': canAcceptDrop
    }"
    @mousedown="handleHeaderMouseDown"
    @mouseup="handleHeaderMouseUp"
    @contextmenu.prevent="handleContextMenu"
    @mouseenter="handleMouseEnter"
    @mouseleave="handleMouseLeave"
  >
    <div class="folder-header" @click="handleToggleClick">
      <svg 
        class="folder-icon" 
        :class="{ 'is-expanded': folder.expanded }"
        xmlns="http://www.w3.org/2000/svg" 
        width="16" 
        height="16" 
        viewBox="0 0 24 24" 
        fill="none" 
        stroke="currentColor" 
        stroke-width="2" 
        stroke-linecap="round" 
        stroke-linejoin="round"
      >
        <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"></path>
      </svg>
      <span class="folder-name">{{ folder.name }}</span>
      <span class="folder-count">({{ folder.streamerIds.length }})</span>
      <motion.span
        class="expand-icon"
        :animate="{ rotate: folder.expanded ? 180 : 0 }"
        :transition="{ duration: 0.2, ease: [0.25, 0.8, 0.4, 1] }"
      >
        <svg 
          xmlns="http://www.w3.org/2000/svg" 
          width="12" 
          height="12" 
          viewBox="0 0 24 24" 
          fill="none" 
          stroke="currentColor" 
          stroke-width="2.5" 
          stroke-linecap="round" 
          stroke-linejoin="round"
        >
          <polyline points="6 9 12 15 18 9"></polyline>
        </svg>
      </motion.span>
    </div>
    <AnimatePresence>
      <motion.div
        v-if="folder.expanded && folderItems.length > 0"
        class="folder-content"
        :class="{ 'disable-pointer': globalDragging }"
        initial="hidden"
        animate="visible"
        exit="exit"
        :variants="{
          hidden: { height: 0, opacity: 0 },
          visible: { 
            height: 'auto', 
            opacity: 1,
            transition: {
              height: { duration: 0.28, ease: [0.22, 1, 0.36, 1] },
              opacity: { duration: 0.2 },
              staggerChildren: 0.025,
              delayChildren: 0.015
            }
          },
          exit: { 
            height: 0, 
            opacity: 0,
            transition: {
              height: { duration: 0.18, ease: [0.64, 0, 0.78, 0] },
              opacity: { duration: 0.12 },
              staggerChildren: 0.015,
              staggerDirection: -1
            }
          }
        }"
      >
        <ul class="folder-streamers-list">
          <motion.li
            v-for="streamer in folderItems"
            :key="`${streamer.platform}:${streamer.id}`"
            class="folder-streamer-item"
            :class="getStreamerItemClass(streamer)"
            :variants="{
              hidden: { opacity: 0, y: -6, scale: 0.98 },
              visible: { 
                opacity: 1, 
                y: 0, 
                scale: 1,
                transition: { duration: 0.22, ease: [0.22, 1, 0.36, 1] }
              },
              exit: { 
                opacity: 0, 
                scale: 0.98,
                transition: { duration: 0.12 }
              }
            }"
            @click.stop="handleClick(streamer)"
            @mousedown.stop="(e) => handleFolderStreamerMouseDown(streamer, e)"
            @mouseup.stop="handleFolderStreamerMouseUp"
            @mouseleave="handleFolderStreamerMouseUp"
          >
            <StreamerItem 
              :streamer="streamer"
              :getAvatarSrc="getAvatarSrc"
              :handleImgError="handleImgError"
              :getLiveIndicatorClass="getLiveIndicatorClass"
              :proxyBase="proxyBase"
              @clickItem="(s) => emit('selectAnchor', s)"
            />
          </motion.li>
        </ul>
      </motion.div>
    </AnimatePresence>
  </motion.div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { AnimatePresence, motion } from 'motion-v';
import type { FollowedStreamer } from '../../platforms/common/types';
import type { FollowFolder } from '../../store/followStore';
import StreamerItem from './StreamerItem.vue';

const props = defineProps<{
  folder: FollowFolder;
  allStreamers: FollowedStreamer[];
  getAvatarSrc: (s: FollowedStreamer) => string;
  handleImgError: (ev: Event, s: FollowedStreamer) => void;
  getLiveIndicatorClass: (s: FollowedStreamer) => string;
  proxyBase?: string;
  isDragging?: boolean;
  isDragOver?: boolean;
  canAcceptDrop?: boolean;
  globalDragging?: boolean;
}>();

const emit = defineEmits<{
  (e: 'selectAnchor', streamer: FollowedStreamer): void;
  (e: 'toggleExpand', folderId: string): void;
  (e: 'dragStart', folderId: string, event: MouseEvent): void;
  (e: 'contextMenu', folderId: string, event: MouseEvent): void;
  (e: 'dragOver', folderId: string): void;
  (e: 'dragLeave'): void;
  (e: 'drop', folderId: string): void;
  (e: 'streamerDragStart', streamer: FollowedStreamer, event: MouseEvent): void;
}>();

const sortStreamersByStatus = (items: FollowedStreamer[]): FollowedStreamer[] => {
  const live: FollowedStreamer[] = [];
  const looping: FollowedStreamer[] = [];
  const rest: FollowedStreamer[] = [];
  items.forEach(streamer => {
    if (streamer.liveStatus === 'LIVE' || (!streamer.liveStatus && streamer.isLive)) {
      live.push(streamer);
    } else if (streamer.liveStatus === 'REPLAY') {
      looping.push(streamer);
    } else {
      rest.push(streamer);
    }
  });
  return [...live, ...looping, ...rest];
};

const folderItems = computed(() => {
  const seen = new Set<string>();
  const result: FollowedStreamer[] = [];
  for (const key of props.folder.streamerIds) {
    const [platform, id] = (key || '').split(':');
    const platformKey = (platform || '').toUpperCase();
    const dedupKey = `${platformKey}:${id}`;
    if (seen.has(dedupKey)) continue;
    const found = props.allStreamers.find((s: FollowedStreamer) => String(s.platform).toUpperCase() === platformKey && s.id === id);
    if (found) {
      seen.add(dedupKey);
      result.push(found);
    }
  }
  return sortStreamersByStatus(result);
});

const toggleExpand = () => emit('toggleExpand', props.folder.id);

const LONG_PRESS_MS = 220;
const MOVE_THRESHOLD_PX = 6;
let headerPressTimer: number | null = null;
let headerLongPressTriggered = false;
let headerDragStarted = false;
let headerStartPoint: { x: number; y: number } | null = null;

const clearHeaderPress = () => {
  if (headerPressTimer !== null) {
    clearTimeout(headerPressTimer);
    headerPressTimer = null;
  }
};

const cleanupHeaderListeners = () => {
  document.removeEventListener('mousemove', handleHeaderMouseMove);
  document.removeEventListener('mouseup', handleHeaderMouseUp);
};

const handleHeaderMouseMove = (e: MouseEvent) => {
  if (!headerStartPoint || headerDragStarted) return;
  const movedDist = Math.hypot(e.clientX - headerStartPoint.x, e.clientY - headerStartPoint.y);
  if (movedDist >= MOVE_THRESHOLD_PX) {
    headerDragStarted = true;
    clearHeaderPress();
    emit('dragStart', props.folder.id, e);
    headerStartPoint = null;
    cleanupHeaderListeners();
  }
};

const handleHeaderMouseDown = (e: MouseEvent) => {
  if (e.button !== 0 || props.globalDragging) return;
  e.preventDefault();
  headerLongPressTriggered = false;
  headerDragStarted = false;
  headerStartPoint = { x: e.clientX, y: e.clientY };
  clearHeaderPress();
  cleanupHeaderListeners();
  document.addEventListener('mousemove', handleHeaderMouseMove);
  document.addEventListener('mouseup', handleHeaderMouseUp);
  headerPressTimer = window.setTimeout(() => {
    headerLongPressTriggered = true;
    headerDragStarted = true;
    emit('dragStart', props.folder.id, e);
    headerStartPoint = null;
    cleanupHeaderListeners();
  }, LONG_PRESS_MS);
};

const handleHeaderMouseUp = () => {
  clearHeaderPress();
  headerStartPoint = null;
  cleanupHeaderListeners();
};

const handleToggleClick = () => {
  if (headerLongPressTriggered) {
    headerLongPressTriggered = false;
    return;
  }
  toggleExpand();
};

const handleContextMenu = (e: MouseEvent) => {
  emit('contextMenu', props.folder.id, e);
};

const handleClick = (streamer: FollowedStreamer) => {
  // 若已进入长按拖动，阻止点击进入观看
  if (longPressTriggered) {
    longPressTriggered = false;
    return;
  }
  emit('selectAnchor', streamer);
};

const handleMouseEnter = () => {
  if (props.canAcceptDrop) {
    emit('dragOver', props.folder.id);
  }
};

const handleMouseLeave = () => {
  clearHeaderPress();
  headerStartPoint = null;
  cleanupHeaderListeners();
  if (props.canAcceptDrop) {
    emit('dragLeave');
  }
};

// 长按触发拖动，避免单击立即进入拖动
const LONG_PRESS_MS_FOL = 220;
const MOVE_THRESHOLD_PX_FOL = 6;
let longPressTimer: number | null = null;
let longPressTriggered = false;
let longPressDragStarted = false;
let longPressStartPoint: { x: number; y: number } | null = null;
let longPressStreamer: FollowedStreamer | null = null;

const clearLongPress = () => {
  if (longPressTimer !== null) {
    clearTimeout(longPressTimer);
    longPressTimer = null;
  }
};

const cleanupFolderStreamerListeners = () => {
  document.removeEventListener('mousemove', handleFolderStreamerMouseMove);
  document.removeEventListener('mouseup', handleFolderStreamerMouseUp);
};

const handleFolderStreamerMouseMove = (event: MouseEvent) => {
  if (!longPressStartPoint || longPressDragStarted) return;
  const movedDist = Math.hypot(event.clientX - longPressStartPoint.x, event.clientY - longPressStartPoint.y);
  if (movedDist >= MOVE_THRESHOLD_PX_FOL) {
    longPressDragStarted = true;
    longPressTriggered = true;
    clearLongPress();
    if (longPressStreamer) {
      emit('streamerDragStart', longPressStreamer, event);
    }
    longPressStartPoint = null;
    cleanupFolderStreamerListeners();
  }
};

const handleFolderStreamerMouseDown = (streamer: FollowedStreamer, event: MouseEvent) => {
  if (props.globalDragging) return;
  event.preventDefault();
  longPressTriggered = false;
  longPressDragStarted = false;
  longPressStartPoint = { x: event.clientX, y: event.clientY };
  longPressStreamer = streamer;
  clearLongPress();
  cleanupFolderStreamerListeners();
  document.addEventListener('mousemove', handleFolderStreamerMouseMove);
  document.addEventListener('mouseup', handleFolderStreamerMouseUp);
  longPressTimer = window.setTimeout(() => {
    longPressTriggered = true;
    longPressDragStarted = true;
    emit('streamerDragStart', streamer, event);
    longPressStartPoint = null;
    cleanupFolderStreamerListeners();
  }, LONG_PRESS_MS_FOL);
};

const handleFolderStreamerMouseUp = () => {
  clearLongPress();
  longPressStartPoint = null;
  longPressStreamer = null;
  cleanupFolderStreamerListeners();
};

const getStreamerItemClass = (streamer: FollowedStreamer) => {
  return {
    'status-live': streamer.liveStatus === 'LIVE',
    'status-replay': streamer.liveStatus === 'REPLAY',
    'status-offline': streamer.liveStatus === 'OFFLINE' || !streamer.liveStatus || streamer.liveStatus === 'UNKNOWN',
  };
};

</script>

<style scoped>
.folder-item {
  position: relative;
  margin-bottom: 8px;
  border-radius: 18px;
  background: transparent;
  overflow: hidden;
  transition: all 0.4s cubic-bezier(0.175, 0.885, 0.32, 1.275);
  user-select: none;
}

.folder-item.is-expanded {
  background: var(--glass-bg);
  backdrop-filter: var(--glass-blur);
  -webkit-backdrop-filter: var(--glass-blur);
  border: 1px solid var(--glass-border);
  box-shadow: var(--glass-shadow);
  transform: scale(1.02);
}

.folder-header {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 10px 16px;
  cursor: pointer;
  border-radius: 14px;
  transition: all 0.3s ease;
  background: var(--tertiary-bg);
  border: 1px solid var(--border-color-light);
}

.folder-header:hover {
  background: var(--hover-bg);
  border-color: var(--accent-color);
}

.folder-icon {
  width: 16px;
  height: 16px;
  color: var(--accent-color);
  transition: all 0.3s cubic-bezier(0.175, 0.885, 0.32, 1.275);
}

.folder-item.is-expanded .folder-icon {
  color: var(--accent-color);
  transform: scale(1.2);
  filter: drop-shadow(0 0 5px rgba(139, 92, 246, 0.3));
}

.folder-name {
  flex: 1;
  font-weight: 700;
  font-size: 14px;
  color: var(--primary-text);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  letter-spacing: 0.02em;
}

.folder-count {
  font-size: 10px;
  color: var(--accent-color);
  font-weight: 800;
  background: var(--primary-bg);
  padding: 2px 8px;
  border-radius: 20px;
  border: 1px solid var(--border-color);
  box-shadow: 0 2px 6px rgba(139, 92, 246, 0.05);
}

.expand-icon {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 12px;
  height: 12px;
  color: var(--accent-color);
}

.folder-content {
  padding: 8px 6px 14px;
  overflow: hidden;
}

.folder-streamers-list {
  list-style: none;
  margin: 0;
  padding: 0;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.folder-streamer-item {
  position: relative;
  display: flex;
  align-items: center;
  padding: 0;
  border-radius: var(--radius-sm);
  background: transparent;
  transition: all 0.2s ease;
}

.folder-streamer-item:hover {
  background: var(--hover-bg);
}
</style>
