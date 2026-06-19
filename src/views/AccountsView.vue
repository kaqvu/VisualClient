<script setup lang="ts">
import { ref } from 'vue';
import { t } from '../composables/useI18n';
import { useAccounts } from '../composables/useAccounts';
import AddAccountModal from '../components/modals/AddAccountModal.vue';
import DeleteAccountModal from '../components/modals/DeleteAccountModal.vue';
import { emit } from '@tauri-apps/api/event';

const { accounts, addAccount, addMicrosoftAccount, selectAccount, deleteAccount } = useAccounts();

const showAddAccountModal = ref(false);
const accountToDelete = ref<string | null>(null);

const confirmDelete = async () => {
  if (accountToDelete.value) {
    await deleteAccount(accountToDelete.value);
    accountToDelete.value = null;
  }
};

const handleSelect = (username: string) => {
  selectAccount(username);
};

const handleCreateAccount = async (payload: any) => {
  if (payload.type === 'offline') {
    if (payload.name && !accounts.value.some(a => a.username === payload.name)) {
      await addAccount(payload.name, 'Offline');
      showAddAccountModal.value = false;
      emit('show_toast', { message: t('accounts.added_offline', { name: payload.name }) });
    }
  } else if (payload.type === 'microsoft') {
    const profile = payload.profile;
    if (profile && profile.name) {
      await addMicrosoftAccount(profile);
      showAddAccountModal.value = false;
      emit('show_toast', { message: t('accounts.added_microsoft', { name: profile.name }) });
    }
  }
};
</script>

<template>
  <div class="accounts-view">
    <header class="accounts-header">
      <h1 class="view-title">{{ t('accounts.title') }}</h1>
      <button class="btn-add-account" @click="showAddAccountModal = true">
        <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <path d="M15 3h4a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2h-4"></path>
          <polyline points="10 17 15 12 10 7"></polyline>
          <line x1="15" y1="12" x2="3" y2="12"></line>
        </svg>
        {{ t('accounts.add') }}
      </button>
    </header>
    
    <div class="accounts-list">
      <div 
        v-for="acc in accounts" 
        :key="acc.username" 
        class="account-card"
        :class="{ active: acc.active }"
        @click="handleSelect(acc.username)"
      >
        <div class="account-info-left">
          <div class="account-avatar">
            <img :src="`https://mc-heads.net/head/${acc.username}`" :alt="acc.username" @error="(e) => (e.target as HTMLImageElement).src='https://mc-heads.net/head/MHF_Steve'" class="avatar-img" />
          </div>
          <div class="account-name-group">
            <span class="account-type">{{ acc.type || 'Offline' }}</span>
            <h3 class="account-name">{{ acc.username }}</h3>
          </div>
        </div>
        
        <div class="actions-right">
          <div class="delete-action" @click.stop="accountToDelete = acc.username">
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <polyline points="3 6 5 6 21 6"></polyline>
              <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"></path>
            </svg>
          </div>
          <div class="radio-circle">
            <div class="radio-inner" v-if="acc.active"></div>
          </div>
        </div>
      </div>
      
      <div v-if="accounts.length === 0" class="empty-accounts">
        {{ t('accounts.empty') }}
      </div>
    </div>

    <Transition name="modal">
      <AddAccountModal 
        v-if="showAddAccountModal" 
        @close="showAddAccountModal = false"
        @create="handleCreateAccount"
      />
    </Transition>

    <Transition name="modal">
      <DeleteAccountModal 
        v-if="accountToDelete"
        :name="accountToDelete"
        @close="accountToDelete = null"
        @confirm="confirmDelete"
      />
    </Transition>
  </div>
</template>

<style scoped>
.accounts-view {
  padding: 40px;
  display: flex;
  flex-direction: column;
  gap: 32px;
}

.accounts-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.view-title {
  margin: 0;
  font-size: 2rem;
  font-weight: 600;
  color: var(--text-main);
}

.btn-add-account {
  padding: 12px 24px;
  background-color: var(--accent);
  color: var(--color-black);
  border: none;
  border-radius: 12px;
  font-size: 1.1rem;
  font-weight: 600;
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

.accounts-list {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
  gap: 16px;
  align-content: start;
}

.account-card {
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
  position: relative;
  will-change: transform;
  transform: translateZ(0);
}

.account-card:hover {
  background-color: var(--surface-dark);
}

.account-card:active:not(:has(.delete-action:active)) {
  transform: scale(0.98);
}

.account-info-left {
  display: flex;
  align-items: center;
  gap: 14px;
  flex: 1;
  min-width: 0;
}

.account-name-group {
  display: flex;
  flex-direction: column;
  justify-content: center;
  gap: 2px;
  flex: 1;
  min-width: 0;
}

.account-type {
  font-size: 0.75rem;
  color: var(--text-muted);
  text-transform: uppercase;
  font-weight: 700;
  letter-spacing: 0.05em;
}

.account-name {
  margin: 0;
  font-size: 1rem;
  font-weight: 700;
  color: var(--color-white);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.account-avatar {
  width: 44px;
  height: 44px;
  border-radius: 12px;
  background-color: var(--bg-shell);
  display: flex;
  align-items: center;
  justify-content: center;
  overflow: hidden;
  flex-shrink: 0;
}

.avatar-img {
  width: 100%;
  height: 100%;
  object-fit: cover;
  image-rendering: pixelated;
}

.actions-right {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-shrink: 0;
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

.account-card:hover .delete-action {
  opacity: 1;
}

.delete-action:hover {
  background-color: color-mix(in srgb, var(--danger) 15%, transparent);
  color: var(--danger);
}

.delete-action:active {
  transform: scale(0.85);
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
  background-color: var(--surface-1);
}

.account-card.active .radio-circle {
  border-color: var(--text-main);
}

.radio-inner {
  width: 10px;
  height: 10px;
  border-radius: 50%;
  background-color: var(--text-main);
}

.empty-accounts {
  grid-column: 1 / -1;
  text-align: center;
  padding: 40px;
  color: var(--text-muted);
  font-style: italic;
}
</style>
