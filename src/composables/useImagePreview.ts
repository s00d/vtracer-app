import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { ImageDataUrlResponse } from "../types/vtracer";

const cache = new Map<string, ImageDataUrlResponse>();

function cacheKey(path: string, trimEmpty: boolean): string {
  return `${path}::trim=${trimEmpty}`;
}

export function useImagePreview() {
  const dataUrl = ref<string | null>(null);
  const width = ref(0);
  const height = ref(0);
  const fileSizeBytes = ref(0);
  const loading = ref(false);
  const error = ref<string | null>(null);
  let currentPath = "";
  let currentTrim = false;

  async function loadPreview(path: string, trimEmpty = false): Promise<void> {
    const trimmed = path.trim();
    if (!trimmed) {
      clear();
      return;
    }

    currentPath = trimmed;
    currentTrim = trimEmpty;
    loading.value = true;
    error.value = null;

    const key = cacheKey(trimmed, trimEmpty);

    try {
      const cached = cache.get(key);
      const result =
        cached ??
        (await invoke<ImageDataUrlResponse>("read_image_data_url", {
          inputPath: trimmed,
          trimEmpty,
        }));
      if (!cached) cache.set(key, result);
      if (currentPath !== trimmed || currentTrim !== trimEmpty) return;

      dataUrl.value = result.dataUrl;
      width.value = result.width;
      height.value = result.height;
      fileSizeBytes.value = result.fileSizeBytes;
    } catch (e) {
      if (currentPath === trimmed && currentTrim === trimEmpty) {
        dataUrl.value = null;
        error.value = e instanceof Error ? e.message : String(e);
      }
    } finally {
      if (currentPath === trimmed && currentTrim === trimEmpty) loading.value = false;
    }
  }

  function clear() {
    currentPath = "";
    currentTrim = false;
    dataUrl.value = null;
    width.value = 0;
    height.value = 0;
    fileSizeBytes.value = 0;
    error.value = null;
    loading.value = false;
  }

  function invalidateCache(path?: string) {
    if (!path) {
      cache.clear();
      return;
    }
    const trimmed = path.trim();
    for (const key of [...cache.keys()]) {
      if (key.startsWith(`${trimmed}::`)) cache.delete(key);
    }
  }

  return {
    dataUrl,
    width,
    height,
    fileSizeBytes,
    loading,
    error,
    loadPreview,
    clear,
    invalidateCache,
  };
}
