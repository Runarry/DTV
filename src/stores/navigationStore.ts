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

const STORAGE_KEY = 'dtv_navigation_state';

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
    loadFromStorage() {
      try {
        const stored = localStorage.getItem(STORAGE_KEY);
        if (stored) {
          const parsed = JSON.parse(stored) as Partial<NavigationState>;
          if (parsed.sourcePlatform) {
            this.sourcePlatform = parsed.sourcePlatform;
          }
          if (parsed.platformStates) {
            // Merge stored states with defaults to handle new platforms
            for (const platform of Object.keys(parsed.platformStates) as Platform[]) {
              if (this.platformStates[platform]) {
                this.platformStates[platform] = parsed.platformStates[platform];
              }
            }
          }
        }
      } catch (error) {
        console.warn('[NavigationStore] Failed to load from storage:', error);
      }
    },

    persist() {
      try {
        localStorage.setItem(STORAGE_KEY, JSON.stringify({
          sourcePlatform: this.sourcePlatform,
          platformStates: this.platformStates,
        }));
      } catch (error) {
        console.warn('[NavigationStore] Failed to persist:', error);
      }
    },

    setSourcePlatform(platform: Platform) {
      if (this.sourcePlatform !== platform) {
        this.sourcePlatform = platform;
        this.persist();
      }
    },

    saveCategoryState(platform: Platform, category: CategoryState) {
      this.platformStates[platform].category = category;
      this.persist();
    },

    saveScrollPosition(platform: Platform, position: number) {
      this.platformStates[platform].scrollPosition = position;
      // Don't persist on every scroll - call persist separately if needed
    },

    persistScrollPosition(_platform: Platform) {
      this.persist();
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
      this.persist();
    },
  },
});
