<script setup lang="ts">
import { ref, computed } from 'vue';
import { t } from '../../composables/useI18n';
import IconChevronRight from '../icons/IconChevronRight.vue';
import IconMicrosoft from '../icons/IconMicrosoft.vue';

const emit = defineEmits(['close', 'create']);

const activeStep = ref<'choose' | 'offline' | 'microsoft'>('choose');
const nickname = ref('');

const isValid = computed(() => {
  const n = nickname.value;
  return n.length >= 3 && n.length <= 16 && /^[a-zA-Z0-9_]+$/.test(n);
});

import { invoke } from '@tauri-apps/api/core';

const handleClose = async () => {
  if (activeStep.value === 'microsoft') {
    await invoke('cancel_microsoft_login');
  }
  emit('close');
};

const goBackToChoose = async () => {
  if (activeStep.value === 'microsoft') {
    await invoke('cancel_microsoft_login');
  }
  activeStep.value = 'choose';
};

const handleCreate = () => {
  if (isValid.value) {
    emit('create', { type: 'offline', name: nickname.value });
  }
};

const loginError = ref('');

const handleMicrosoftLogin = async () => {
  activeStep.value = 'microsoft';
  loginError.value = '';
  try {
    const profile = await invoke('start_microsoft_login');
    emit('create', { type: 'microsoft', profile });
  } catch (e) {
    console.error("Microsoft login failed:", e);
    if (String(e) !== "Login window closed") {
      loginError.value = String(e);
    } else {
      activeStep.value = 'choose';
    }
  }
};
</script>

<template>
  <div class="modal-backdrop" @click.self="handleClose">
    <div class="modal-container add-account-modal">
      <header class="modal-header">
        <div class="breadcrumb">
          <span class="breadcrumb-item static">{{ t('accounts.add_account') }}</span>
          <span class="breadcrumb-separator"><IconChevronRight /></span>
          <span 
            class="breadcrumb-item" 
            :class="{ active: activeStep === 'choose', clickable: activeStep !== 'choose' }" 
            @click="activeStep !== 'choose' ? goBackToChoose() : null"
          >
            {{ t('accounts.choose_type') }}
          </span>
          
          <template v-if="activeStep === 'offline' || activeStep === 'microsoft'">
            <span class="breadcrumb-separator"><IconChevronRight /></span>
            <span class="breadcrumb-item active">{{ activeStep === 'offline' ? t('accounts.offline') : t('accounts.microsoft') }}</span>
          </template>
        </div>
        
        <div class="close-control" @click="handleClose">
          <svg width="16" height="16" viewBox="0 0 12 12" fill="none" xmlns="http://www.w3.org/2000/svg">
            <path d="M3 3L9 9M9 3L3 9" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>
          </svg>
        </div>
      </header>
      
      <div class="modal-content-wrapper">
        <div class="sliding-container" :class="'step-' + activeStep">
          <div class="slide-pane choose-pane">
            <div class="account-type-card" @click="activeStep = 'offline'">
              <div class="card-icon offline-icon">
                <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                  <path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2"></path>
                  <circle cx="12" cy="7" r="4"></circle>
                </svg>
              </div>
              <div class="card-info">
                <span class="card-title">{{ t('accounts.offline') }}</span>
                <span class="card-desc">{{ t('accounts.offline_desc') }}</span>
              </div>
            </div>

            <div class="account-type-card" @click="handleMicrosoftLogin">
              <div class="card-icon">
                <IconMicrosoft />
              </div>
              <div class="card-info">
                <span class="card-title">{{ t('accounts.microsoft') }}</span>
                <span class="card-desc">{{ t('accounts.microsoft_desc') }}</span>
              </div>
            </div>
          </div>

          <div class="slide-pane offline-pane">
            <template v-if="activeStep === 'offline'">
              <div class="input-group">
                <label class="input-label">{{ t('accounts.nickname_label') }}</label>
                <input 
                  type="text" 
                  v-model="nickname" 
                  :placeholder="t('accounts.placeholder')" 
                  class="nickname-input"
                  @keyup.enter="handleCreate"
                  maxlength="16"
                />
                <span class="input-hint">{{ t('accounts.nickname_hint') }}</span>
              </div>
              <div class="modal-footer">
                <button class="btn-add" :disabled="!isValid" @click="handleCreate">
                  {{ t('accounts.add_account') }}
                </button>
              </div>
            </template>
            <template v-else-if="activeStep === 'microsoft'">
              <div class="microsoft-loading">
                <div v-if="!loginError" class="spinner"></div>
                <span v-if="!loginError">{{ t('accounts.waiting_for_login') }}</span>
                
                <div v-if="loginError" class="error-box">
                  <span class="error-title">Błąd logowania</span>
                  <span class="error-msg">{{ loginError }}</span>
                  <button class="btn-add" @click="activeStep = 'choose'">Wróć</button>
                </div>
              </div>
            </template>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.modal-backdrop {
  position: fixed;
  top: 0;
  left: 0;
  width: 100vw;
  height: 100vh;
  background-color: color-mix(in srgb, var(--color-black) 40%, transparent);
  backdrop-filter: blur(8px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 999;
}

.add-account-modal {
  width: 640px;
  background-color: var(--bg-shell);
  border-radius: 16px;
  border: 1px solid var(--border-line);
  display: flex;
  flex-direction: column;
  overflow: hidden;
  box-shadow: 0 10px 40px color-mix(in srgb, var(--color-black) 50%, transparent);
}

.modal-header {
  padding: 0 24px;
  min-height: 80px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  border-bottom: 1px solid var(--border-line);
}

.breadcrumb {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 1.1rem;
}

.breadcrumb-item {
  color: var(--text-muted);
  transition: color 0.2s;
}

.breadcrumb-item.active {
  color: var(--color-white);
  font-weight: 600;
}

.breadcrumb-item.clickable {
  cursor: pointer;
}

.breadcrumb-item.clickable:hover {
  color: var(--text-main);
}

.breadcrumb-separator {
  display: flex;
  align-items: center;
  color: var(--text-muted);
}

.breadcrumb-separator svg {
  width: 18px;
  height: 18px;
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

.modal-content-wrapper {
  position: relative;
  overflow: hidden;
  height: 280px;
}

.sliding-container {
  display: flex;
  width: 200%;
  height: 100%;
  transition: transform 0.4s cubic-bezier(0.34, 1.56, 0.64, 1);
}

.sliding-container.step-choose {
  transform: translateX(0);
}

.sliding-container.step-offline,
.sliding-container.step-microsoft {
  transform: translateX(-50%);
}

.slide-pane {
  width: 50%;
  height: 100%;
  padding: 32px 24px;
  display: flex;
  flex-direction: column;
}

.choose-pane {
  gap: 16px;
}

.account-type-card {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 20px;
  background-color: color-mix(in srgb, var(--color-white) 3%, transparent);
  border: 1px solid var(--border-line);
  border-radius: 16px;
  cursor: pointer;
  transition: background-color 0.2s ease, color 0.2s ease, transform 0.2s cubic-bezier(0.34, 1.56, 0.64, 1);
}

.account-type-card:not(.disabled):hover {
  background-color: color-mix(in srgb, var(--color-white) 6%, transparent);
  border-color: var(--text-muted);
}

.account-type-card:not(.disabled):active {
  transform: scale(0.85);
}

.account-type-card.disabled {
  opacity: 0.5;
  cursor: not-allowed;
  filter: grayscale(0.5);
}

.card-icon {
  width: 48px;
  height: 48px;
  background-color: var(--bg-shell);
  border-radius: 12px;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.offline-icon {
  color: var(--text-main);
}

.card-info {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.card-title {
  font-size: 1.1rem;
  font-weight: 600;
  color: var(--text-main);
}

.card-desc {
  font-size: 0.9rem;
  font-weight: 600;
  color: var(--text-muted);
}

.offline-pane {
  justify-content: space-between;
}

.input-group {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.input-label {
  font-weight: 600;
  color: var(--text-main);
  font-size: 0.95rem;
}

.nickname-input {
  width: 100%;
  height: 48px;
  background-color: color-mix(in srgb, var(--color-white) 3%, transparent);
  border: none;
  border-radius: 12px;
  padding: 0 16px;
  color: var(--text-main);
  font-size: 1.05rem;
  font-family: inherit;
  outline: none;
  transition: background-color 0.2s ease, color 0.2s ease, transform 0.2s cubic-bezier(0.34, 1.56, 0.64, 1);
}

.nickname-input:focus {
  background-color: color-mix(in srgb, var(--color-white) 6%, transparent);
  box-shadow: 0 0 0 4px var(--accent);
}

.input-hint {
  font-size: 0.85rem;
  color: var(--text-muted);
  font-weight: 500;
}

.modal-footer {
  display: flex;
  justify-content: flex-end;
}

.btn-add {
  padding: 12px 24px;
  background-color: var(--accent);
  color: var(--color-black);
  border: none;
  border-radius: 12px;
  font-size: 1rem;
  -webkit-font-smoothing: antialiased;
  cursor: pointer;
  transition: background-color 0.2s ease, color 0.2s ease, transform 0.2s cubic-bezier(0.34, 1.56, 0.64, 1);
}

.btn-add:not(:disabled):hover {
  background-color: color-mix(in srgb, var(--accent) 85%, var(--color-black));
}

.btn-add:not(:disabled):active {
  transform: scale(0.85);
}

.btn-add:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.microsoft-loading {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
  gap: 16px;
  color: var(--text-muted);
  font-size: 1.1rem;
  font-weight: 500;
}

.spinner {
  width: 32px;
  height: 32px;
  border: 3px solid color-mix(in srgb, var(--color-white) 6%, transparent);
  border-top-color: var(--accent);
  border-radius: 50%;
  animation: spin 1s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

.error-box {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
  text-align: center;
}

.error-title {
  color: var(--danger);
  font-weight: 600;
  font-size: 1.2rem;
}

.error-msg {
  color: var(--text-muted);
  font-size: 0.95rem;
  max-width: 80%;
}
</style>
