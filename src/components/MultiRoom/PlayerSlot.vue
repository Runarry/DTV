<template>
  <div
    class="player-slot"
    :class="{
      'player-slot--active': room.id === multiRoomStore.activeRoomId,
      'player-slot--hidden': multiRoomStore.layoutMode === 'single' && room.id !== multiRoomStore.activeRoomId,
    }"
    @click="multiRoomStore.setActiveRoom(room.id)"
  >
    <MainPlayer
      v-if="!isLoadingDetails"
      :room-id="room.roomId"
      :platform="room.platform"
      :is-followed="isFollowed"
      :title="streamerDetails?.roomTitle ?? room.title ?? undefined"
      :anchor-name="streamerDetails?.nickname ?? room.anchorName ?? undefined"
      :avatar="streamerDetails?.avatarUrl ?? room.avatar ?? undefined"
      :is-live="streamerDetails?.isLive ?? room.isLive ?? undefined"
      :initial-error="detailsError"
      :muted="room.isMuted"
      :compact-mode="multiRoomStore.isMultiRoom && multiRoomStore.layoutMode === 'split'"
      :should-play="!multiRoomStore.isMultiRoom || multiRoomStore.layoutMode === 'split' || room.id === multiRoomStore.activeRoomId"
      @follow="handleFollow"
      @unfollow="handleUnfollow"
      @close-player="handleClosePlayer"
      @fullscreen-change="handleFullscreenChange"
    />
    <div v-else class="slot-loading">
      <LoadingDots />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted } from 'vue';
import MainPlayer from '../player/index.vue';
import LoadingDots from '../Common/LoadingDots.vue';
import { useMultiRoomStore, type OpenRoom } from '../../stores/multiRoom';
import { useFollowStore } from '../../store/followStore';
import { Platform } from '../../platforms/common/types';
import type { FollowedStreamer, StreamerDetails } from '../../platforms/common/types';

const props = defineProps<{
  room: OpenRoom;
}>();

const emit = defineEmits<{
  (e: 'close-room', id: string): void;
  (e: 'fullscreen-change', isFullscreen: boolean): void;
}>();

const multiRoomStore = useMultiRoomStore();
const followStore = useFollowStore();

const streamerDetails = ref<StreamerDetails | null>(null);
const isLoadingDetails = ref(false);
const detailsError = ref<string | null>(null);

const isFollowed = computed(() => {
  return followStore.isFollowed(props.room.platform, props.room.roomId);
});

const loadStreamerDetails = async () => {
  const { platform, roomId } = props.room;
  if (!roomId) return;

  isLoadingDetails.value = true;
  detailsError.value = null;

  try {
    if (platform === Platform.DOUYU) {
      const { fetchDouyuStreamerDetails } = await import('../../platforms/douyu/streamerInfoParser');
      const result = await fetchDouyuStreamerDetails(roomId);
      if (result?.errorMessage) {
        detailsError.value = result.errorMessage;
      } else if (result) {
        streamerDetails.value = result;
        multiRoomStore.updateRoomMeta(props.room.id, {
          title: result.roomTitle,
          anchorName: result.nickname,
          avatar: result.avatarUrl ?? undefined,
          isLive: result.isLive,
        });
      }
    } else if (platform === Platform.DOUYIN) {
      // Douyin details are loaded inside MainPlayer via fetchAndPrepareDouyinStreamConfig
      // Just skip external loading
    } else if (platform === Platform.HUYA) {
      // Huya details loaded in MainPlayer via getHuyaStreamConfig
    } else if (platform === Platform.BILIBILI) {
      // Bilibili details loaded in MainPlayer
    }
  } catch (e: any) {
    detailsError.value = e.message || '加载主播详情失败';
  } finally {
    isLoadingDetails.value = false;
  }
};

const handleFollow = (streamerData: { nickname: string; avatarUrl: string; roomTitle?: string }) => {
  const streamer: FollowedStreamer = {
    id: props.room.roomId,
    platform: props.room.platform,
    nickname: streamerData.nickname,
    avatarUrl: streamerData.avatarUrl,
    roomTitle: streamerData.roomTitle,
  };
  followStore.followStreamer(streamer);
};

const handleUnfollow = () => {
  followStore.unfollowStreamer(props.room.platform, props.room.roomId);
};

const handleClosePlayer = () => {
  emit('close-room', props.room.id);
};

const handleFullscreenChange = (isFullscreen: boolean) => {
  emit('fullscreen-change', isFullscreen);
};

onMounted(() => {
  if (props.room.platform === Platform.DOUYU) {
    loadStreamerDetails();
  }
});

watch(() => props.room.roomId, () => {
  if (props.room.platform === Platform.DOUYU) {
    streamerDetails.value = null;
    loadStreamerDetails();
  }
});
</script>

<style scoped>
.player-slot {
  position: relative;
  overflow: hidden;
  border-radius: 4px;
  outline: 2px solid transparent;
  transition: outline-color 0.15s ease;
  min-height: 0;
  display: flex;
  flex-direction: column;
}

.player-slot--active {
  outline-color: var(--accent-color, #5c16c5);
}

.player-slot--hidden {
  display: none;
}

.slot-loading {
  display: flex;
  align-items: center;
  justify-content: center;
  flex: 1;
  min-height: 200px;
}
</style>
