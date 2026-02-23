<template>
  <div
    class="persistent-multi-room-host"
    :class="{ 'persistent-multi-room-host--visible': visible, 'persistent-multi-room-host--minimized': !visible }"
    :aria-hidden="!visible"
  >
    <div class="persistent-multi-room-host__shell">
      <RoomTabBar @close-room="handleCloseRoom" />
      <MultiRoomContainer
        @fullscreen-change="handleFullscreenChange"
        @empty="handleEmptyRooms"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { watch } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { useMultiRoomStore } from '../../stores/multiRoom';
import MultiRoomContainer from './MultiRoomContainer.vue';
import RoomTabBar from './RoomTabBar.vue';

const FORCE_RESUME_EVENT = 'dtv-force-resume-players';

const props = defineProps<{
  visible: boolean;
}>();

const emit = defineEmits<{
  (e: 'fullscreen-change', isFullscreen: boolean): void;
}>();

const store = useMultiRoomStore();
const router = useRouter();
const route = useRoute();

const dispatchForceResume = () => {
  if (typeof window === 'undefined') {
    return;
  }
  window.dispatchEvent(new CustomEvent(FORCE_RESUME_EVENT));
};

const handleEmptyRooms = () => {
  emit('fullscreen-change', false);
  if (props.visible && route.name === 'multiPlayer') {
    router.replace('/');
  }
};

const handleCloseRoom = (id: string) => {
  store.closeRoom(id);
  if (store.roomCount === 0) {
    handleEmptyRooms();
  }
};

const handleFullscreenChange = (isFullscreen: boolean) => {
  emit('fullscreen-change', isFullscreen);
};

watch(
  () => props.visible,
  (visible) => {
    if (visible) {
      dispatchForceResume();
    }
  },
);
</script>

<style scoped>
.persistent-multi-room-host {
  min-height: 0;
}

.persistent-multi-room-host--visible {
  position: absolute;
  inset: 0;
  z-index: 20;
  pointer-events: auto;
}

.persistent-multi-room-host--minimized {
  position: fixed;
  inset: 0;
  z-index: -1;
  opacity: 0;
  pointer-events: none;
}

.persistent-multi-room-host__shell {
  display: flex;
  flex-direction: column;
  width: 100%;
  height: 100%;
  min-height: 0;
}
</style>
