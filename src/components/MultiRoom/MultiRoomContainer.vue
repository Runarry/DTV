<template>
  <div class="multi-room-container" :class="gridClass">
    <PlayerSlot
      v-for="room in store.rooms"
      :key="room.id"
      :room="room"
      @close-room="handleCloseRoom"
      @fullscreen-change="handleFullscreenChange"
    />
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { useMultiRoomStore } from '../../stores/multiRoom';
import PlayerSlot from './PlayerSlot.vue';

const store = useMultiRoomStore();

const emit = defineEmits<{
  (e: 'fullscreen-change', isFullscreen: boolean): void;
  (e: 'empty'): void;
}>();

const gridClass = computed(() => {
  const count = store.roomCount;
  if (count <= 1 || store.layoutMode === 'single') return 'grid-single';
  if (count === 2) return 'grid-2';
  return 'grid-4'; // 3 or 4 rooms
});

const handleCloseRoom = (id: string) => {
  store.closeRoom(id);
  if (store.roomCount === 0) {
    emit('empty');
  }
};

const handleFullscreenChange = (isFullscreen: boolean) => {
  emit('fullscreen-change', isFullscreen);
};
</script>

<style scoped>
.multi-room-container {
  display: grid;
  width: 100%;
  height: 100%;
  min-height: 0;
  gap: 2px;
  background: #000;
}

.grid-single {
  grid-template-columns: 1fr;
  grid-template-rows: 1fr;
}

.grid-2 {
  grid-template-columns: 1fr 1fr;
  grid-template-rows: 1fr;
}

.grid-4 {
  grid-template-columns: 1fr 1fr;
  grid-template-rows: 1fr 1fr;
}
</style>
