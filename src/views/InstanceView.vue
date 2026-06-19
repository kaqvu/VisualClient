<script setup lang="ts">
import { ref, computed } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { t } from '../composables/useI18n';
import { useInstances, Instance } from '../composables/useInstances';
import { useAccounts } from '../composables/useAccounts';
import IconInstance from '../components/icons/IconInstance.vue';
import IconPlay from '../components/icons/IconPlay.vue';
import IconGamepad from '../components/icons/IconGamepad.vue';
import IconFolder from '../components/icons/IconFolder.vue';
import LoginRequiredModal from '../components/modals/LoginRequiredModal.vue';

const props = defineProps<{
  instanceId: string
}>();

const emit = defineEmits(['openSettings', 'openAccounts']);

const { instances } = useInstances();
const { accounts } = useAccounts();

const instance = computed(() => {
  return instances.value.find((i: Instance) => i.id === props.instanceId);
});

const activeTab = ref(0);

const showLoginModal = ref(false);

const handlePlay = async () => {
  const currentAccount = accounts.value.find(a => a.active)?.username;
  if (!currentAccount) {
    showLoginModal.value = true;
  } else {
    try {
      await invoke('launch_instance', { id: instance.value!.id, username: currentAccount, launchingText: t('instance.launching') });
    } catch (e) {
      console.error(e);
    }
  }
};

const handleAddAccountClick = () => {
  showLoginModal.value = false;
  emit('openAccounts');
};

const openFolder = async () => {
  if (instance.value) {
    try {
      await invoke('open_instance_folder', { id: instance.value.id });
    } catch (e) {
      console.error(e);
    }
  }
};
</script>

<template>
  <div v-if="instance" class="instance-view">
    <header class="instance-header">
      <div class="header-left">
        <div class="library-icon">
          <IconInstance class="library-icon-svg" />
        </div>
        <div class="instance-info">
          <h1 class="title">{{ instance.name }}</h1>
          <div class="instance-loader">
            <IconGamepad class="gamepad-icon" />
            <span class="subtitle">{{ instance.loader.charAt(0).toUpperCase() + instance.loader.slice(1) }} {{ instance.version }}</span>
          </div>
        </div>
      </div>
      <div class="actions">
        <button class="btn-play" @click="handlePlay">
          <IconPlay class="play-icon" />
          {{ t('instance.play') || 'Play' }}
        </button>
        <button class="btn-settings" @click="emit('openSettings', instance)">
          <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <circle cx="12" cy="12" r="3"></circle>
            <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z"></path>
          </svg>
        </button>
        <button class="btn-settings" @click="openFolder">
          <IconFolder class="folder-icon" />
        </button>
      </div>
    </header>

    <div class="tabs-container">
      <div class="tab-indicator" :style="{ transform: `translateX(${activeTab * 100}%)` }"></div>
      
      <div 
        class="tab-button" 
        :class="{ active: activeTab === 0 }"
        @click="activeTab = 0"
      >
        <svg class="tab-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <rect x="2" y="2" width="20" height="8" rx="2" ry="2"></rect>
          <rect x="2" y="14" width="20" height="8" rx="2" ry="2"></rect>
          <line x1="6" y1="6" x2="6.01" y2="6"></line>
          <line x1="6" y1="18" x2="6.01" y2="18"></line>
        </svg>
        <span>Servers</span>
      </div>

      <div 
        class="tab-button" 
        :class="{ active: activeTab === 1 }"
        @click="activeTab = 1"
      >
        <svg class="tab-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <circle cx="12" cy="12" r="10"></circle>
          <line x1="2" y1="12" x2="22" y2="12"></line>
          <path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z"></path>
        </svg>
        <span>Worlds</span>
      </div>
    </div>

    <Transition name="modal">
      <LoginRequiredModal 
        v-if="showLoginModal" 
        @close="showLoginModal = false" 
        @login="handleAddAccountClick" 
      />
    </Transition>
  </div>
</template>

<style scoped>
.instance-view {
  display: flex;
  flex-direction: column;
  height: 100%;
  padding: 0;
}

.instance-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 0;
  padding-bottom: 32px;
  border-bottom: 1px solid var(--border-line);
}

.tabs-container {
  display: flex;
  position: relative;
  background-color: var(--surface-hover);
  border-radius: 999px;
  padding: 4px;
  width: fit-content;
  margin-top: 10px;
}

.tab-indicator {
  position: absolute;
  top: 4px;
  bottom: 4px;
  left: 4px;
  width: calc((100% - 8px) / 2);
  background-color: color-mix(in srgb, var(--accent) 15%, transparent);
  border-radius: 999px;
  transition: transform 0.35s cubic-bezier(0.25, 1, 0.5, 1);
}

.tab-button {
  position: relative;
  z-index: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  padding: 6px 20px;
  cursor: pointer;
  border-radius: 999px;
  transition: transform 0.2s cubic-bezier(0.34, 1.56, 0.64, 1);
}

.tab-button:active {
  transform: scale(0.92);
}

.tab-button span {
  font-weight: 700;
  color: var(--color-white);
  font-size: 0.9rem;
  transition: color 0.2s ease;
}

.tab-icon {
  width: 14px;
  height: 14px;
  color: var(--text-muted);
  transition: color 0.2s ease;
}

.tab-button.active span,
.tab-button.active .tab-icon {
  color: var(--accent);
}

.header-left {
  display: flex;
  align-items: center;
  gap: 20px;
}

.library-icon {
  width: 72px;
  height: 72px;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  border: 4px solid var(--surface-2);
  border-radius: 16px;
  background-color: var(--surface-1);
}

.library-icon-svg {
  width: 72px;
  height: 72px;
}

.instance-info {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.title {
  margin: 0;
  font-size: 1.8rem;
  font-weight: 700;
  color: var(--text-main);
}

.instance-loader {
  display: flex;
  align-items: center;
  gap: 6px;
  color: var(--text-muted);
}

.gamepad-icon {
  width: 18px;
  height: 18px;
  color: var(--text-muted);
}

.subtitle {
  font-size: 1rem;
  color: var(--text-muted);
  font-weight: 600;
}

.actions {
  display: flex;
  align-items: center;
  gap: 12px;
}

.btn-play {
  padding: 0 24px;
  height: 44px;
  background-color: var(--accent);
  color: var(--color-black);
  border: none;
  border-radius: 16px;
  font-size: 1.1rem;
  font-weight: 600;
  cursor: pointer;
  display: flex;
  align-items: center;
  gap: 8px;
  transition: background-color 0.2s ease, transform 0.2s cubic-bezier(0.34, 1.56, 0.64, 1);
}

.play-icon {
  width: 20px;
  height: 20px;
}

.btn-play:hover {
  background-color: var(--accent-hover);
}

.btn-play:active {
  transform: scale(0.96);
}

.btn-settings {
  width: 48px;
  height: 48px;
  background-color: var(--surface-2);
  color: var(--text-secondary);
  border: none;
  border-radius: 50%;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: background-color 0.2s ease, transform 0.2s cubic-bezier(0.34, 1.56, 0.64, 1);
}

.folder-icon {
  width: 20px;
  height: 20px;
}

.btn-settings:hover {
  background-color: var(--surface-3);
}

.btn-settings:active {
  transform: scale(0.85);
}
</style>
