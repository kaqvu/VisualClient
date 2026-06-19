<script setup lang="ts">
import { currentTheme, setTheme, t } from '../../../composables/useI18n';
</script>

<template>
  <div class="settings-page">
    <h2>{{ t('appearance.title') }}</h2>
    <p class="subtitle">{{ t('appearance.subtitle') }}</p>
    
    <div class="theme-grid">
      <!-- Dark Theme Tile -->
      <div 
        class="theme-tile" 
        :class="{ active: currentTheme === 'dark' }"
        @click="setTheme('dark')"
      >
        <div class="theme-preview dark-preview">
          <div class="mock-card">
            <div class="mock-square"></div>
            <div class="mock-lines">
              <div class="mock-line wide"></div>
              <div class="mock-line narrow"></div>
            </div>
          </div>
        </div>
        <div class="theme-footer">
          <div class="footer-left">
            <div class="radio-circle">
              <div class="radio-inner" v-if="currentTheme === 'dark'"></div>
            </div>
            <span class="theme-name">{{ t('appearance.dark') }}</span>
          </div>
          <svg class="theme-icon" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z"></path>
          </svg>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.subtitle {
  color: var(--text-muted);
  font-size: 0.95rem;
  margin-top: -4px;
  margin-bottom: 24px;
}

.theme-grid {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 20px;
}

.theme-tile {
  background-color: transparent;
  border-radius: 16px;
  border: none;
  cursor: pointer;
  overflow: hidden;
  display: flex;
  flex-direction: column;
  transition: transform 0.2s cubic-bezier(0.34, 1.56, 0.64, 1);
}

.theme-tile:active {
  transform: scale(0.96);
}

.theme-preview {
  height: 150px;
  padding: 16px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.mock-card {
  width: 100%;
  padding: 16px;
  border-radius: 16px;
  display: flex;
  align-items: center;
  gap: 16px;
  box-shadow: 0 4px 12px var(--overlay-light);
}

.mock-square {
  width: 40px;
  height: 40px;
  border-radius: 10px;
}

.mock-lines {
  display: flex;
  flex-direction: column;
  gap: 8px;
  flex: 1;
}

.mock-line {
  height: 8px;
  border-radius: 4px;
}

.mock-line.wide {
  width: 80%;
}

.mock-line.narrow {
  width: 40%;
}

.dark-preview {
  background-color: var(--bg-content);
}
.dark-preview .mock-card {
  background-color: var(--bg-shell);
}
.dark-preview .mock-square {
  background-color: var(--border-line);
}
.dark-preview .mock-line.wide {
  background-color: var(--text-main);
}
.dark-preview .mock-line.narrow {
  background-color: var(--text-muted);
}



.theme-footer {
  padding: 12px 16px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  background-color: var(--surface-1);
  transition: background-color 0.2s;
}

.theme-tile:hover .theme-footer {
  background-color: var(--surface-hover);
}

.theme-tile.active .theme-footer {
  background-color: var(--surface-active);
}

.footer-left {
  display: flex;
  align-items: center;
  gap: 12px;
}

.radio-circle {
  width: 20px;
  height: 20px;
  border-radius: 50%;
  border: 2px solid var(--border-line);
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.2s;
  background-color: var(--bg-content);
}

.theme-tile.active .radio-circle {
  border-color: var(--text-main);
}

.radio-inner {
  width: 10px;
  height: 10px;
  border-radius: 50%;
  background-color: var(--text-main);
}

.theme-name {
  font-weight: 600;
  color: var(--text-main);
  font-size: 0.95rem;
}

.theme-icon {
  color: var(--text-muted);
}
</style>
