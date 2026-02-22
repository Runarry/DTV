import { defineStore } from 'pinia';
import { Platform } from '../platforms/common/types';

export interface OpenRoom {
  id: string;           // "${platform}:${roomId}"
  platform: Platform;
  roomId: string;
  title?: string;
  anchorName?: string;
  avatar?: string;
  isLive?: boolean;
  isMuted: boolean;
}

export type LayoutMode = 'single' | 'split';

const MAX_ROOMS = 4;

export const useMultiRoomStore = defineStore('multiRoom', {
  state: () => ({
    rooms: [] as OpenRoom[],
    activeRoomId: null as string | null,
    layoutMode: 'split' as LayoutMode,
  }),

  getters: {
    roomCount: (state) => state.rooms.length,
    activeRoom: (state) => state.rooms.find(r => r.id === state.activeRoomId) || null,
    isMultiRoom: (state) => state.rooms.length > 1,
  },

  actions: {
    openRoom(platform: Platform, roomId: string, meta?: { title?: string; anchorName?: string; avatar?: string; isLive?: boolean }) {
      const id = `${platform}:${roomId}`;
      const existing = this.rooms.find(r => r.id === id);
      if (existing) {
        // Room already open, set active via shared logic to keep mute state in sync
        this.setActiveRoom(id);
        if (meta) {
          if (meta.title !== undefined) existing.title = meta.title;
          if (meta.anchorName !== undefined) existing.anchorName = meta.anchorName;
          if (meta.avatar !== undefined) existing.avatar = meta.avatar;
          if (meta.isLive !== undefined) existing.isLive = meta.isLive;
        }
        return;
      }

      if (this.rooms.length >= MAX_ROOMS) {
        // Remove the oldest non-active room, or the first room
        const toRemoveIdx = this.rooms.findIndex(r => r.id !== this.activeRoomId);
        if (toRemoveIdx >= 0) {
          this.rooms.splice(toRemoveIdx, 1);
        } else {
          this.rooms.shift();
        }
      }

      const newRoom: OpenRoom = {
        id,
        platform,
        roomId,
        title: meta?.title,
        anchorName: meta?.anchorName,
        avatar: meta?.avatar,
        isLive: meta?.isLive,
        isMuted: this.rooms.length > 0, // First room unmuted, others muted
      };

      this.rooms.push(newRoom);
      this.activeRoomId = id;

      // Mute all other rooms when a new room becomes active
      for (const room of this.rooms) {
        room.isMuted = room.id !== id;
      }
    },

    closeRoom(id: string) {
      const idx = this.rooms.findIndex(r => r.id === id);
      if (idx === -1) return;

      this.rooms.splice(idx, 1);

      if (this.activeRoomId === id) {
        // Transfer active to first remaining room
        this.activeRoomId = this.rooms.length > 0 ? this.rooms[0].id : null;
        // Unmute the new active room
        if (this.activeRoomId) {
          const newActive = this.rooms.find(r => r.id === this.activeRoomId);
          if (newActive) newActive.isMuted = false;
        }
      }
    },

    setActiveRoom(id: string) {
      const room = this.rooms.find(r => r.id === id);
      if (!room) return;

      this.activeRoomId = id;
      // Mute all rooms except the active one
      for (const r of this.rooms) {
        r.isMuted = r.id !== id;
      }
    },

    toggleMute(id: string) {
      const room = this.rooms.find(r => r.id === id);
      if (!room) return;
      room.isMuted = !room.isMuted;

      // If unmuting, this becomes the active room and mute others
      if (!room.isMuted) {
        this.activeRoomId = id;
        for (const r of this.rooms) {
          if (r.id !== id) r.isMuted = true;
        }
      }
    },

    setLayoutMode(mode: LayoutMode) {
      this.layoutMode = mode;
    },

    updateRoomMeta(id: string, meta: Partial<Pick<OpenRoom, 'title' | 'anchorName' | 'avatar' | 'isLive'>>) {
      const room = this.rooms.find(r => r.id === id);
      if (!room) return;
      if (meta.title !== undefined) room.title = meta.title;
      if (meta.anchorName !== undefined) room.anchorName = meta.anchorName;
      if (meta.avatar !== undefined) room.avatar = meta.avatar;
      if (meta.isLive !== undefined) room.isLive = meta.isLive;
    },

    clearAll() {
      this.rooms = [];
      this.activeRoomId = null;
    },
  },
});
