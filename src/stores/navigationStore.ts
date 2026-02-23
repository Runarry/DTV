import { defineStore } from 'pinia';

export type Platform = 'douyu' | 'douyin' | 'huya' | 'bilibili';

// Category state for Huya/Bilibili/Douyin
interface CommonCategoryState {
  cate2Href: string;
  cate2Name: string;
}

// Category state for Douyu (different structure)
interface DouyuCategoryState {
  douyuCategoryType: 'cate2' | 'cate3';
  douyuCategoryId: string;
  douyuCategoryName?: string;
  douyuParentCate2Id?: string;
}

type CategoryState = CommonCategoryState | DouyuCategoryState | null;

interface PlatformNavigationState {
  category: CategoryState;
  scrollPosition: number;
}

interface NavigationState {
  sourcePlatform: Platform;
  platformStates: Record<Platform, PlatformNavigationState>;
}

const getDefaultState = (): NavigationState => ({
  sourcePlatform: 'douyu',
  platformStates: {
    douyu: { category: null, scrollPosition: 0 },
    douyin: { category: null, scrollPosition: 0 },
    huya: { category: null, scrollPosition: 0 },
    bilibili: { category: null, scrollPosition: 0 },
  },
});

export const useNavigationStore = defineStore('navigation', {
  state: (): NavigationState => getDefaultState(),

  actions: {
    // Navigation state is session-only; no localStorage restore on app start.
    loadFromStorage() {
      // no-op
    },

    setSourcePlatform(platform: Platform) {
      if (this.sourcePlatform !== platform) {
        this.sourcePlatform = platform;
      }
    },

    saveCategoryState(platform: Platform, category: CategoryState) {
      this.platformStates[platform].category = category;
    },

    saveScrollPosition(platform: Platform, position: number) {
      this.platformStates[platform].scrollPosition = position;
    },

    persistScrollPosition(_platform: Platform) {
      // no-op: kept for compatibility with existing call sites.
    },

    getPlatformState(platform: Platform): PlatformNavigationState {
      return this.platformStates[platform];
    },

    getSourcePlatformRoute(): string {
      const routeMap: Record<Platform, string> = {
        douyu: '/',
        douyin: '/douyin',
        huya: '/huya',
        bilibili: '/bilibili',
      };
      return routeMap[this.sourcePlatform] || '/';
    },

    clearPlatformState(platform: Platform) {
      this.platformStates[platform] = { category: null, scrollPosition: 0 };
    },
  },
});
