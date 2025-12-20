<template>
  <div class="cate2-container">
    <div
      class="cate2-content"
      :class="{ 'is-expanded': isExpanded }"
      ref="cate2ContentRef"
    >
      <div class="cate2-scroll-wrapper" :class="{ 'allow-scroll': isExpanded && hasMoreRows }">
        <motion.div 
          class="cate2-grid" 
          ref="cate2GridRef"
          initial="hidden"
          animate="visible"
          :variants="{
            visible: {
              transition: {
                staggerChildren: 0.02
              }
            }
          }"
        >
          <motion.div
            v-for="cate2 in cate2List"
            :key="cate2.cate2Id"
            class="cate2-card"
            :class="{ 'active': selectedCate2Id === cate2.cate2Id }"
            @click="$emit('select', cate2)"
            :variants="{
              hidden: { opacity: 0, y: 10, scale: 0.95 },
              visible: { 
                opacity: 1, 
                y: 0, 
                scale: 1,
                transition: { type: 'spring', stiffness: 300, damping: 30 }
              }
            }"
            whileHover="{ scale: 1.04, y: -2 }"
            whileTap="{ scale: 0.96 }"
          >
            <div class="cate2-icon">
              <img :src="cate2.icon" :alt="cate2.cate2Name">
            </div>
            <div class="cate2-info">
              <div class="cate2-name" :title="cate2.cate2Name">{{ formatCategoryName(cate2.cate2Name) }}</div>
            </div>
          </motion.div>
        </motion.div>
      </div>
    </div>

    <div v-if="shouldShowExpandButtonComputed" class="expand-button" @click="handleToggleExpand">
      <span>{{ isExpanded ? '收起' : '展开' }}</span>
      <motion.svg
        class="expand-icon"
        :animate="{ rotate: isExpanded ? 180 : 0 }"
        width="16"
        height="16"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
      >
        <path d="M6 9l6 6 6-6" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
      </motion.svg>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, watch, nextTick, computed } from 'vue'
import { motion } from 'motion-v'
import type { Category2 } from '../types'

const props = defineProps<{
  cate2List: Category2[]
  selectedCate2Id: number | null
  isExpanded: boolean
  hasMoreRows: boolean
}>()

const emit = defineEmits<{
  (e: 'select', cate2: Category2): void
  (e: 'toggle-expand'): void
  (e: 'height-changed'): void
}>()

// Constants for height calculation
const CARD_ACTUAL_HEIGHT = 42; 
const GRID_VERTICAL_GAP = 12;  
const CONTENT_PADDING_BOTTOM = 8; 
const GRID_INTERNAL_PADDING_BOTTOM = 16; 

const TARGET_CONTENT_HEIGHT_FOR_ONE_ROW = CARD_ACTUAL_HEIGHT + GRID_INTERNAL_PADDING_BOTTOM + CONTENT_PADDING_BOTTOM;
const TARGET_CONTENT_HEIGHT_FOR_TWO_ROWS = (2 * CARD_ACTUAL_HEIGHT + GRID_VERTICAL_GAP) + GRID_INTERNAL_PADDING_BOTTOM + CONTENT_PADDING_BOTTOM;
const EXPANDED_CONTENT_MAX_ROWS = 7;
const TARGET_CONTENT_HEIGHT_FOR_EXPANDED_MAX_ROWS = (EXPANDED_CONTENT_MAX_ROWS * CARD_ACTUAL_HEIGHT + (EXPANDED_CONTENT_MAX_ROWS - 1) * GRID_VERTICAL_GAP) + GRID_INTERNAL_PADDING_BOTTOM + CONTENT_PADDING_BOTTOM;

const cate2ContentRef = ref<HTMLElement | null>(null)
const cate2GridRef = ref<HTMLElement | null>(null)
const isAnimating = ref(false)
const actualGridScrollHeight = ref(0)

const getCurrentTargetHeight = (expandedState: boolean) => {
  const naturalContentHeight = actualGridScrollHeight.value + CONTENT_PADDING_BOTTOM;
  if (expandedState) {
    if (props.hasMoreRows) return TARGET_CONTENT_HEIGHT_FOR_EXPANDED_MAX_ROWS;
    return props.cate2List.length > 0 ? naturalContentHeight : GRID_INTERNAL_PADDING_BOTTOM + CONTENT_PADDING_BOTTOM;
  } else {
    if (naturalContentHeight <= TARGET_CONTENT_HEIGHT_FOR_ONE_ROW) return naturalContentHeight;
    return TARGET_CONTENT_HEIGHT_FOR_TWO_ROWS;
  }
};

const updateActualGridScrollHeight = () => {
  nextTick(() => {
    if (cate2GridRef.value) actualGridScrollHeight.value = cate2GridRef.value.scrollHeight;
    else actualGridScrollHeight.value = GRID_INTERNAL_PADDING_BOTTOM;
    
    if (cate2ContentRef.value) {
      cate2ContentRef.value.style.height = `${getCurrentTargetHeight(props.isExpanded)}px`;
      emit('height-changed');
    }
  });
};

watch(() => props.cate2List, updateActualGridScrollHeight, { deep: true });
onMounted(updateActualGridScrollHeight);

const shouldShowExpandButtonComputed = computed(() => {
  return (actualGridScrollHeight.value + CONTENT_PADDING_BOTTOM) > TARGET_CONTENT_HEIGHT_FOR_TWO_ROWS;
});

watch(() => props.isExpanded, (newValue) => {
  if (!cate2ContentRef.value) return;
  const targetHeight = getCurrentTargetHeight(newValue);
  cate2ContentRef.value.style.height = `${targetHeight}px`;
  
  // Wait for transition
  setTimeout(() => emit('height-changed'), 400);
});

const handleToggleExpand = () => emit('toggle-expand');

const formatCategoryName = (name: string) => {
  if (!name) return '';
  const getStringLength = (str: string) => {
    let len = 0;
    for (let i = 0; i < str.length; i++) {
      len += str.charCodeAt(i) > 127 || str.charCodeAt(i) === 94 ? 1 : 0.5;
    }
    return Math.ceil(len);
  };
  if (getStringLength(name) <= 5) return name;
  let result = '', currentLength = 0;
  for (let i = 0; i < name.length; i++) {
    const charLength = name.charCodeAt(i) > 127 || name.charCodeAt(i) === 94 ? 1 : 0.5;
    if (currentLength + charLength <= 4.5) {
      result += name[i];
      currentLength += charLength;
    } else break;
  }
  return result + '...';
}
</script>

<style scoped>
.cate2-container {
  padding: 12px 16px;
  display: flex;
  flex-direction: column;
  flex: 1;
  position: relative;
}

.cate2-content {
  position: relative;
  height: 0;
  overflow: hidden;
  transition: height 0.4s cubic-bezier(0.16, 1, 0.3, 1);
  will-change: height;
}

.cate2-scroll-wrapper {
  height: 100%;
  overflow: hidden;
}

.cate2-content.is-expanded .cate2-scroll-wrapper.allow-scroll {
  overflow-y: auto !important;
  scrollbar-width: thin;
}

.cate2-scroll-wrapper.allow-scroll::-webkit-scrollbar {
  width: 4px;
}

.cate2-scroll-wrapper.allow-scroll::-webkit-scrollbar-thumb {
  background: var(--border-color);
  border-radius: 10px;
}

.cate2-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(120px, 1fr));
  gap: 10px;
  padding: 4px;
}

.cate2-card {
  height: 40px; 
  padding: 0 12px; 
  border-radius: var(--radius-md);
  cursor: pointer;
  display: flex; 
  align-items: center; 
  gap: 10px; 
  background: var(--secondary-bg);
  border: 1px solid var(--border-color);
  color: var(--secondary-text);
  position: relative;
  overflow: hidden;
  transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
}

.cate2-card:hover {
  background: var(--tertiary-bg);
  border-color: var(--accent-color);
  color: var(--primary-text);
  transform: translateY(-1px);
}

.cate2-card.active { 
  background: var(--accent-gradient);
  border-color: transparent;
  color: #fff;
  font-weight: 600;
  box-shadow: 0 4px 10px rgba(139, 92, 246, 0.2);
}

.cate2-icon {
  width: 20px; 
  height: 20px; 
  flex-shrink: 0; 
}

.cate2-icon img {
  width: 100%; 
  height: 100%; 
  object-fit: contain; 
  border-radius: 4px; 
}

.cate2-card.active .cate2-icon img {
  filter: brightness(0) invert(1);
}

.cate2-info {
  flex: 1;
  overflow: hidden;
}

.cate2-name {
  font-size: 13px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.expand-button {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 10px;
  font-size: 12px;
  cursor: pointer;
  color: var(--secondary-text);
  transition: all 0.2s ease;
  margin: 0 16px;
  background: var(--tertiary-bg);
  border-radius: 20px;
  width: fit-content;
  align-self: center;
  margin-top: 8px;
  border: 1px solid var(--border-color);
}

.expand-button:hover {
  color: var(--primary-text);
  background: var(--hover-bg);
  border-color: var(--accent-color);
}

.expand-icon {
  margin-left: 6px;
  width: 14px;
  height: 14px;
}
</style>
