<template>
  <div class="multi-room-view">
    <RoomTabBar @close-room="handleCloseRoom" />
    <MultiRoomContainer
      @fullscreen-change="handleFullscreenChange"
    />
  </div>
</template>

<script setup lang="ts">
import { useRouter } from 'vue-router';
import { useMultiRoomStore } from '../stores/multiRoom';
import MultiRoomContainer from '../components/MultiRoom/MultiRoomContainer.vue';
import RoomTabBar from '../components/MultiRoom/RoomTabBar.vue';

const store = useMultiRoomStore();
const router = useRouter();

const emit = defineEmits(['fullscreen-change']);

const handleCloseRoom = (id: string) => {
  store.closeRoom(id);
  if (store.roomCount === 0) {
    router.replace('/');
  }
};

const handleFullscreenChange = (isFullscreen: boolean) => {
  emit('fullscreen-change', isFullscreen);
};
</script>

<style scoped>
.multi-room-view {
  display: flex;
  flex-direction: column;
  height: 100%;
  width: 100%;
  min-height: 0;
  background-color: transparent;
}
</style>
