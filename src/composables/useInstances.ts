import { ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';

export interface Instance {
  id: string;
  name: string;
  loader: string;
  version: string;
  java_path: string;
}

const instances = ref<Instance[]>([]);
const isLoaded = ref(false);

export function useInstances() {
  const fetchInstances = async () => {
    try {
      instances.value = await invoke<Instance[]>('get_instances');
      isLoaded.value = true;
    } catch (e) {
      console.error('Failed to fetch instances:', e);
    }
  };

  const renameInstance = async (id: string, newName: string) => {
    try {
      await invoke('rename_instance', { id, newName });
      const inst = instances.value.find(i => i.id === id);
      if (inst) {
        inst.name = newName;
      }
    } catch (e) {
      console.error('Failed to rename instance:', e);
    }
  };

  const deleteInstance = async (id: string) => {
    try {
      await invoke('delete_instance', { id });
      instances.value = instances.value.filter(i => i.id !== id);
    } catch (e) {
      console.error('Failed to delete instance:', e);
    }
  };

  if (!isLoaded.value) {
    fetchInstances();
  }

  return {
    instances,
    isLoaded,
    fetchInstances,
    renameInstance,
    deleteInstance
  };
}
