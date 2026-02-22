import { createRouter, createWebHistory } from 'vue-router'
import DouyuHomeView from '../pages/DouyuHomeView.vue'
import DouyinHomeView from '../pages/DouyinHomeView.vue'
import DouyuPlayerView from '../pages/DouyuPlayerView.vue';
import DouyinPlayerView from '../pages/DouyinPlayerView.vue';
import HuyaHomeView from '../pages/HuyaHomeView.vue'
import HuyaPlayerView from '../pages/HuyaPlayerView.vue'
import BilibiliHomeView from '../pages/BilibiliHomeView.vue'
import BilibiliPlayerView from '../pages/BilibiliPlayerView.vue'
import CustomHomeView from '../pages/CustomHomeView.vue'
import MultiRoomView from '../pages/MultiRoomView.vue'
import { Platform } from '../platforms/common/types'
import { useMultiRoomStore } from '../stores/multiRoom'

const platformMap: Record<string, Platform> = {
  douyuPlayer: Platform.DOUYU,
  douyinPlayer: Platform.DOUYIN,
  huyaPlayer: Platform.HUYA,
  bilibiliPlayer: Platform.BILIBILI,
}

const router = createRouter({
  history: createWebHistory(),
  routes: [
    {
      path: '/',
      name: 'DouyuHome',
      component: DouyuHomeView
    },
    {
      path: '/douyin',
      name: 'DouyinHome',
      component: DouyinHomeView
    },
    {
      path: '/huya',
      name: 'HuyaHome',
      component: HuyaHomeView
    },
    {
      path: '/bilibili',
      name: 'BilibiliHome',
      component: BilibiliHomeView
    },
    {
      path: '/custom',
      name: 'CustomHome',
      component: CustomHomeView
    },
    {
      path: '/player/douyu/:roomId',
      name: 'douyuPlayer',
      component: DouyuPlayerView,
      props: true,
      beforeEnter: (to) => {
        const store = useMultiRoomStore()
        const platform = platformMap[to.name as string]
        if (platform) {
          store.openRoom(platform, to.params.roomId as string)
          return { name: 'multiPlayer' }
        }
      },
    },
    {
      path: '/player/douyin/:roomId',
      name: 'douyinPlayer',
      component: DouyinPlayerView,
      props: true,
      beforeEnter: (to) => {
        const store = useMultiRoomStore()
        const platform = platformMap[to.name as string]
        if (platform) {
          store.openRoom(platform, to.params.roomId as string)
          return { name: 'multiPlayer' }
        }
      },
    },
    {
      path: '/player/huya/:roomId',
      name: 'huyaPlayer',
      component: HuyaPlayerView,
      props: true,
      beforeEnter: (to) => {
        const store = useMultiRoomStore()
        const platform = platformMap[to.name as string]
        if (platform) {
          store.openRoom(platform, to.params.roomId as string)
          return { name: 'multiPlayer' }
        }
      },
    },
    {
      path: '/player/bilibili/:roomId',
      name: 'bilibiliPlayer',
      component: BilibiliPlayerView,
      props: true,
      beforeEnter: (to) => {
        const store = useMultiRoomStore()
        const platform = platformMap[to.name as string]
        if (platform) {
          store.openRoom(platform, to.params.roomId as string)
          return { name: 'multiPlayer' }
        }
      },
    },
    {
      path: '/multi-player',
      name: 'multiPlayer',
      component: MultiRoomView,
    },
  ]
})

export default router
