import { ref, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { VtracerSettings } from "@/types/vtracer";
import { DEFAULT_SETTINGS, settingsFromBackend, settingsToBackend } from "@/types/vtracer";

const SAVE_DEBOUNCE_MS = 400;

export function usePersistedSettings() {
  const settings = ref<VtracerSettings>({ ...DEFAULT_SETTINGS });
  let hydrated = false;
  let saveTimer: ReturnType<typeof setTimeout> | null = null;

  async function hydrate(): Promise<void> {
    try {
      const raw = await invoke<Record<string, unknown> | null>("app_settings_load");
      if (raw) {
        Object.assign(settings.value, settingsFromBackend(raw));
      }
    } catch {
      // keep defaults
    } finally {
      hydrated = true;
    }
  }

  function scheduleSave(): void {
    if (!hydrated) return;
    if (saveTimer) clearTimeout(saveTimer);
    saveTimer = setTimeout(() => {
      void invoke("app_settings_save", { settings: settingsToBackend(settings.value) });
    }, SAVE_DEBOUNCE_MS);
  }

  watch(settings, scheduleSave, { deep: true });

  void hydrate();

  return { settings };
}
