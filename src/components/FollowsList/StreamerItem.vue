<template>
  <div class="streamer-item-content" :class="{ big: big }">
    <div class="item-content" @click="onClick">
      <div class="avatar-container" :class="{ big: big }">
        <img 
          v-if="streamer.avatarUrl && (streamer.platform !== Platform.BILIBILI || !!proxyBase)"
          :src="getAvatarSrc(streamer)"
          :alt="streamer.nickname"
          @error="handleImgError($event, streamer)"
          class="avatar-image"
        >
        <div v-else class="avatar-fallback">{{ streamer.nickname[0] }}</div>
      </div>
      
      <div class="streamer-details">
        <div class="primary-row">
          <span class="nickname" :title="streamer.nickname">{{ streamer.nickname }}</span>
          <!-- 移除左侧平台名，改为右侧胶囊与状态点集成 -->
        </div>
        <div class="secondary-row" :title="streamer.roomTitle">
          {{ streamer.roomTitle || '暂无直播标题' }}
        </div>
      </div>
    </div>

    <div class="status-container">
      <div v-if="showPlatform" class="platform-badge">
        <span class="live-indicator" :class="getLiveIndicatorClass(streamer)"></span>
        <span class="badge-text">{{ platformLabel(streamer.platform) }}</span>
      </div>
      <div v-else class="live-indicator" :class="getLiveIndicatorClass(streamer)"></div>
    </div>
  </div>
</template>

<style scoped>
.streamer-item-content {
  display: flex;
  align-items: center;
  justify-content: flex-start;
  width: 100%;
  user-select: none;
  transition: all 0.4s cubic-bezier(0.175, 0.885, 0.32, 1.275);
  border-radius: 14px;
  position: relative;
  overflow: hidden;
  border: 1px solid var(--glass-border);
  background: var(--glass-bg);
  backdrop-filter: var(--glass-blur);
  -webkit-backdrop-filter: var(--glass-blur);
  margin-bottom: 2px;
}

.streamer-item-content:hover {
  background: var(--hover-bg);
  border-color: var(--accent-color);
  transform: translateY(-2px) scale(1.01);
  box-shadow: 0 12px 24px rgba(139, 92, 246, 0.15);
  z-index: 10;
}

.item-content {
  display: flex;
  align-items: center;
  padding: 10px 14px;
  gap: 14px;
  flex: 1;
  min-width: 0;
  z-index: 2;
}

.avatar-container {
  width: 40px;
  height: 40px;
  border-radius: 50%;
  position: relative;
  flex: 0 0 auto;
  transition: all 0.3s ease;
}

.avatar-image {
  width: 100%;
  height: 100%;
  object-fit: cover;
  border-radius: 50%;
  border: 2px solid transparent;
  background: var(--tertiary-bg);
  transition: all 0.3s ease;
}

.avatar-container.is-live::before {
  content: '';
  position: absolute;
  inset: -3px;
  border-radius: 50%;
  background: var(--accent-gradient);
  padding: 2px;
  -webkit-mask: linear-gradient(#fff 0 0) content-box, linear-gradient(#fff 0 0);
  mask: linear-gradient(#fff 0 0) content-box, linear-gradient(#fff 0 0);
  -webkit-mask-composite: xor;
  mask-composite: exclude;
  animation: rotate-border 3s linear infinite;
}

.avatar-container.is-live .avatar-image {
  border-color: #fff;
  box-shadow: 0 0 15px rgba(139, 92, 246, 0.4);
}

@keyframes rotate-border {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}

.avatar-fallback {
  font-size: 14px;
  font-weight: 700;
  color: var(--accent-color);
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--tertiary-bg);
  border-radius: 50%;
}

.streamer-details {
  display: flex;
  flex-direction: column;
  flex: 1;
  min-width: 0;
}

.nickname {
  font-weight: 700;
  color: var(--primary-text);
  font-size: 14px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  letter-spacing: 0.01em;
}

.secondary-row {
  font-size: 11px;
  color: var(--secondary-text);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-weight: 500;
  margin-top: 3px;
  opacity: 0.8;
}

.status-container {
  display: flex;
  align-items: center;
  margin-left: auto;
  flex: 0 0 auto;
  padding-right: 14px;
  z-index: 2;
}

.platform-badge {
  display: flex;
  align-items: center;
  gap: 6px;
  border-radius: 20px;
  padding: 3px 10px;
  background: var(--tertiary-bg);
  color: var(--accent-color);
  font-size: 10px;
  font-weight: 800;
  border: 1px solid var(--border-color);
  box-shadow: 0 2px 6px rgba(139, 92, 246, 0.1);
}

.live-indicator {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: var(--dim-text);
  transition: all 0.3s ease;
}

.live-indicator.is-live { 
  background: #10b981; 
  box-shadow: 0 0 12px #10b981;
  animation: breathing 2s ease-in-out infinite;
}

@keyframes breathing {
  0%, 100% { transform: scale(1); opacity: 1; }
  50% { transform: scale(1.2); opacity: 0.7; }
}

.live-indicator.is-replay { background: #f59e0b; box-shadow: 0 0 8px #f59e0b; }
.live-indicator.is-offline { background: var(--border-color); }
</style>

<script setup lang="ts">
import { Platform } from '../../platforms/common/types';
import type { FollowedStreamer } from '../../platforms/common/types';
import { computed } from 'vue';

const props = defineProps<{
  streamer: FollowedStreamer,
  getAvatarSrc: (s: FollowedStreamer) => string,
  handleImgError: (ev: Event, s: FollowedStreamer) => void,
  getLiveIndicatorClass: (s: FollowedStreamer) => string,
  proxyBase?: string,
  big?: boolean,
  showPlatform?: boolean
}>();

const emit = defineEmits<{ (e: 'clickItem', s: FollowedStreamer): void }>();

const onClick = () => emit('clickItem', props.streamer);

const platformLabel = (p: Platform): string => {
  switch (p) {
    case Platform.DOUYU: return '斗鱼';
    case Platform.DOUYIN: return '抖音';
    case Platform.HUYA: return '虎牙';
    case Platform.BILIBILI: return 'B站';
    default: return '未知';
  }
};

const proxyBase = computed(() => props.proxyBase || '');
const showPlatform = computed(() => !!props.showPlatform);
</script>
