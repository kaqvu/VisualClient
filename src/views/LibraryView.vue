<script setup lang="ts">
import { ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { t } from '../composables/useI18n';
import { useInstances } from '../composables/useInstances';
import IconInstance from '../components/icons/IconInstance.vue';
import IconGamepad from '../components/icons/IconGamepad.vue';
import IconPlay from '../components/icons/IconPlay.vue';
import DeleteInstanceModal from '../components/modals/DeleteInstanceModal.vue';
import { useAccounts } from '../composables/useAccounts';

const emit = defineEmits(['createInstance', 'openInstance', 'openAccounts']);
const { instances, deleteInstance } = useInstances();
const { accounts } = useAccounts();

const instanceToDelete = ref<{id: string, name: string} | null>(null);
const showLoginModal = ref(false);
const playTargetInstance = ref<{id: string, name: string} | null>(null);

const confirmDelete = async () => {
  if (instanceToDelete.value) {
    await deleteInstance(instanceToDelete.value.id);
    instanceToDelete.value = null;
  }
};

const handleQuickPlay = async (instance: {id: string, name: string}) => {
  const currentAccount = accounts.value.find(a => a.active)?.username;
  if (!currentAccount) {
    playTargetInstance.value = instance;
    showLoginModal.value = true;
  } else {
    try {
      await invoke('launch_instance', { id: instance.id, username: currentAccount, launchingText: t('instance.launching') });
    } catch (e) {
      console.error(e);
    }
  }
};

const handleAddAccountClick = () => {
  showLoginModal.value = false;
  emit('openAccounts');
};
</script>

<template>
  <div class="library-view">
    <div v-if="instances.length === 0" class="empty-state">
      <svg class="empty-icon" width="64" height="64" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
        <rect x="2" y="3" width="20" height="14" rx="2" ry="2"></rect>
        <line x1="8" y1="21" x2="16" y2="21"></line>
        <line x1="12" y1="17" x2="12" y2="21"></line>
      </svg>
      <h2 class="empty-title">{{ t('library.empty') }}</h2>
      <button class="create-btn" @click="emit('createInstance')">
        <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="plus-icon">
          <path d="M5 12h14" />
          <path d="M12 5v14" />
        </svg>
        <span>{{ t('library.create') }}</span>
      </button>
    </div>
    
    <div v-else class="instances-list">
      <div 
        v-for="instance in instances" 
        :key="instance.id" 
        class="instance-card"
        @click="emit('openInstance', instance.id)"
      >
        <div class="instance-info-left">
          <div class="instance-avatar">
            <IconInstance class="library-icon-svg" />
            <div class="quick-play-overlay" @click.stop="handleQuickPlay(instance)">
              <IconPlay class="quick-play-icon" />
            </div>
          </div>
          <div class="instance-info">
            <h3 class="instance-name">{{ instance.name }}</h3>
            <div class="instance-loader">
              <IconGamepad class="gamepad-icon" />
              <span>{{ instance.loader.charAt(0).toUpperCase() + instance.loader.slice(1) }} {{ instance.version }}</span>
            </div>
          </div>
        </div>
        <div class="delete-action" @click.stop="instanceToDelete = { id: instance.id, name: instance.name }">
          <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <polyline points="3 6 5 6 21 6"></polyline>
            <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"></path>
          </svg>
        </div>
      </div>
    </div>
    
    <Transition name="modal">
      <DeleteInstanceModal 
        v-if="instanceToDelete"
        :name="instanceToDelete.name"
        @close="instanceToDelete = null"
        @confirm="confirmDelete"
      />
    </Transition>

    <Transition name="modal">
      <div v-if="showLoginModal" class="modal-backdrop" @click.self="showLoginModal = false">
        <div class="modal-container login-modal">
          <header class="login-modal-header">
            <span class="header-title">{{ t('instance.login_modal_title') }}</span>
            <div class="close-control" @click="showLoginModal = false">
              <svg width="16" height="16" viewBox="0 0 12 12" fill="none" xmlns="http://www.w3.org/2000/svg">
                <path d="M3 3L9 9M9 3L3 9" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>
              </svg>
            </div>
          </header>
          <div class="login-modal-body">
            <p class="login-text">{{ t('instance.login_modal_text') }}</p>
            <button class="btn-add-account" @click="handleAddAccountClick">
              <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2"></path>
                <circle cx="12" cy="7" r="4"></circle>
              </svg>
              {{ t('instance.login_modal_btn') }}
            </button>
          </div>
        </div>
      </div>
    </Transition>
  </div>
</template>

<style scoped>
.library-view {
  display: flex;
  flex-direction: column;
  height: 100%;
  padding: 32px;
  overflow-y: auto;
}

.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  flex: 1;
  gap: 12px;
  opacity: 0.8;
}

.empty-icon {
  color: var(--text-muted);
  margin-bottom: 8px;
}

.empty-title {
  font-size: 1.25rem;
  font-weight: 600;
  color: var(--text-main);
}

.create-btn {
  background-color: var(--accent);
  color: var(--color-black);
  border: none;
  padding: 12px 24px;
  border-radius: 999px;
  font-size: 1.1rem;
  font-weight: 600;
  cursor: pointer;
  transition: transform 0.2s cubic-bezier(0.34, 1.56, 0.64, 1), background-color 0.2s;
  margin-top: 16px;
  display: flex;
  align-items: center;
  gap: 8px;
}

.create-btn:hover {
  background-color: var(--accent-hover);
}

.create-btn:active {
  transform: scale(0.94);
}

.instances-list {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
  gap: 16px;
  align-content: start;
}

.instance-card {
  background-color: var(--bg-shell);
  border: none;
  border-radius: 16px;
  padding: 14px 16px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  cursor: pointer;
  transition: background-color 0.2s ease, transform 0.2s ease;
}

.instance-card:hover {
  background-color: var(--surface-dark);
}

.instance-card:active:not(:has(.delete-action:active)):not(:has(.quick-play-overlay:active)) {
  transform: scale(0.98);
}

.instance-info-left {
  display: flex;
  align-items: center;
  gap: 14px;
  flex: 1;
  min-width: 0;
}

.instance-avatar {
  width: 44px;
  height: 44px;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  position: relative;
}

.library-icon-svg {
  width: 40px;
  height: 40px;
  transition: opacity 0.2s;
}

.quick-play-overlay {
  position: absolute;
  width: 32px;
  height: 32px;
  background-color: var(--accent);
  color: var(--color-black);
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  opacity: 0;
  transform: scale(0.8);
  transition: all 0.2s cubic-bezier(0.34, 1.56, 0.64, 1);
  cursor: pointer;
  z-index: 2;
}

.instance-card:hover .quick-play-overlay {
  opacity: 1;
  transform: scale(1);
}

.quick-play-overlay:hover {
  background-color: var(--accent-hover);
}

.quick-play-overlay:active {
  transform: scale(0.85) !important;
}

.quick-play-icon {
  width: 18px;
  height: 18px;
  margin-right: 1px;
  pointer-events: none;
}

.instance-info {
  display: flex;
  flex-direction: column;
  gap: 2px;
  overflow: hidden;
}

.instance-name {
  margin: 0;
  font-size: 1rem;
  font-weight: 700;
  color: var(--color-white);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.instance-loader {
  font-size: 0.85rem;
  color: var(--text-muted);
  display: flex;
  align-items: center;
  gap: 6px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.gamepad-icon {
  width: 14px;
  height: 14px;
  flex-shrink: 0;
  color: var(--text-muted);
}

.delete-action {
  width: 32px;
  height: 32px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--text-muted);
  transition: all 0.2s;
  flex-shrink: 0;
  opacity: 0;
}

.instance-card:hover .delete-action {
  opacity: 1;
}

.delete-action:hover {
  background-color: color-mix(in srgb, var(--danger) 15%, transparent);
  color: var(--danger);
}

.delete-action:active {
  transform: scale(0.85);
}


.modal-backdrop {
  position: fixed;
  top: 0;
  left: 0;
  width: 100vw;
  height: 100vh;
  background-color: var(--backdrop-dark);
  backdrop-filter: blur(8px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 999;
}

.login-modal {
  width: 480px;
  background-color: var(--bg-shell);
  border-radius: 16px;
  border: 1px solid var(--border-line);
  display: flex;
  flex-direction: column;
  overflow: hidden;
  box-shadow: 0 10px 40px color-mix(in srgb, var(--color-black) 50%, transparent);
}

.login-modal-header {
  padding: 0 24px;
  min-height: 80px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  border-bottom: 1px solid var(--border-line);
}

.header-title {
  font-size: 1.1rem;
  font-weight: 600;
  color: var(--text-main);
}

.close-control {
  width: 32px;
  height: 32px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--text-muted);
  cursor: pointer;
  transition: background-color 0.2s, color 0.2s, transform 0.2s;
}

.close-control:hover {
  background-color: var(--danger);
  color: var(--color-white);
}

.close-control:active {
  transform: scale(0.85);
}

.login-modal-body {
  padding: 32px 24px;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 24px;
}

.login-text {
  margin: 0;
  font-size: 1.05rem;
  color: var(--text-muted);
  text-align: center;
  font-weight: 600;
}

.btn-add-account {
  padding: 12px 24px;
  background-color: var(--accent);
  color: var(--color-black);
  border: none;
  border-radius: 12px;
  font-size: 1rem;
  cursor: pointer;
  display: flex;
  align-items: center;
  gap: 8px;
  transition: all 0.25s cubic-bezier(0.34, 1.56, 0.64, 1);
}

.btn-add-account:hover {
  background-color: var(--accent-hover);
}

.btn-add-account:active {
  transform: scale(0.96);
}
</style>
