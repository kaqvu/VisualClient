<script setup lang="ts">
import { ref } from 'vue';
import { t } from '../../../composables/useI18n';
import { useInstances, Instance } from '../../../composables/useInstances';
import IconTrash from '../../icons/IconTrash.vue';

const props = defineProps<{ instance: Instance }>();
const emit = defineEmits(['requestDelete']);

const { renameInstance } = useInstances();
const instanceNameInput = ref(props.instance.name);

const handleNameUpdate = () => {
  renameInstance(props.instance.id, instanceNameInput.value);
};
</script>

<template>
  <div class="settings-section">
    <div class="setting-item">
      <label class="setting-label"><strong>{{ t('instance_settings.name') }}</strong></label>
      <input 
        type="text" 
        v-model="instanceNameInput" 
        class="setting-input" 
        @input="handleNameUpdate"
      />
    </div>

    <div class="setting-item">
      <div class="setting-info">
        <span class="setting-name">{{ t('instance_settings.delete_title') }}</span>
        <span class="setting-desc">{{ t('instance_settings.delete_desc') }}</span>
      </div>
      <div>
        <button class="btn-delete" @click="emit('requestDelete')">
          <IconTrash class="btn-icon" />
          {{ t('instance_settings.delete_btn') }}
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.settings-section {
  display: flex;
  flex-direction: column;
  gap: 24px;
}

.setting-item {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.setting-info {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.setting-name {
  font-weight: 600;
  font-size: 1rem;
  color: var(--text-main);
}

.setting-desc {
  font-size: 0.85rem;
  color: var(--text-muted);
}

.setting-label {
  color: var(--text-main);
}

.setting-input {
  width: 100%;
  padding: 12px 16px;
  background-color: var(--surface-1);
  border: none;
  color: var(--text-main);
  border-radius: 8px;
  font-family: inherit;
  font-size: 1rem;
  transition: border-color 0.2s;
}

.setting-input:focus {
  outline: none;
  background-color: var(--surface-hover);
}

.btn-delete {
  background-color: var(--danger-hover);
  color: var(--color-black);
  border: none;
  border-radius: 12px;
  padding: 10px 24px;
  font-size: 1rem;
  font-weight: 600;
  cursor: pointer;
  display: flex;
  align-items: center;
  gap: 8px;
  transition: all 0.2s cubic-bezier(0.34, 1.56, 0.64, 1);
}

.btn-icon {
  width: 18px;
  height: 18px;
}

.btn-delete:hover {
  background-color: var(--danger-active);
}

.btn-delete:active {
  transform: scale(0.95);
}
</style>
