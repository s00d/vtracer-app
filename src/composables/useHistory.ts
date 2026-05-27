import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { HistoryEntry, NamedPreset, VtracerSettings } from "@/types/vtracer";
import { settingsFromBackend, settingsToBackend } from "@/types/vtracer";

export function useHistoryStore() {
  const entries = ref<HistoryEntry[]>([]);
  const presets = ref<NamedPreset[]>([]);
  const loading = ref(false);

  async function refresh(): Promise<void> {
    loading.value = true;
    try {
      const [history, profiles] = await Promise.all([
        invoke<HistoryEntry[]>("history_list"),
        invoke<NamedPreset[]>("presets_list"),
      ]);
      entries.value = history.map((e) => ({
        ...e,
        settings: settingsFromBackend(e.settings as unknown as Record<string, unknown>),
      }));
      presets.value = profiles.map((p) => ({
        ...p,
        settings: settingsFromBackend(p.settings as unknown as Record<string, unknown>),
      }));
    } finally {
      loading.value = false;
    }
  }

  async function addEntry(entry: HistoryEntry): Promise<void> {
    await invoke("history_add", {
      entry: {
        ...entry,
        settings: settingsToBackend(entry.settings),
      },
    });
    await refresh();
  }

  async function removeEntry(id: string): Promise<void> {
    await invoke("history_delete", { id });
    await refresh();
  }

  async function clearHistory(): Promise<void> {
    await invoke("history_clear");
    await refresh();
  }

  async function savePreset(name: string, settings: VtracerSettings): Promise<NamedPreset> {
    const preset = await invoke<NamedPreset>("presets_save", {
      request: { name, settings: settingsToBackend(settings) },
    });
    await refresh();
    return preset;
  }

  async function deletePreset(id: string): Promise<void> {
    await invoke("presets_delete", { id });
    await refresh();
  }

  return {
    entries,
    presets,
    loading,
    refresh,
    addEntry,
    removeEntry,
    clearHistory,
    savePreset,
    deletePreset,
  };
}
