import { onBeforeUnmount, onMounted, ref } from "vue";
import { TauriEvent, listen, type UnlistenFn } from "@tauri-apps/api/event";

export function useDragDrop(onPaths: (paths: string[]) => void) {
  const isDragActive = ref(false);
  const unlisteners: UnlistenFn[] = [];

  onMounted(async () => {
    unlisteners.push(
      await listen(TauriEvent.DRAG_ENTER, () => {
        isDragActive.value = true;
      }),
      await listen(TauriEvent.DRAG_OVER, () => {
        isDragActive.value = true;
      }),
      await listen(TauriEvent.DRAG_LEAVE, () => {
        isDragActive.value = false;
      }),
      await listen<{ paths?: string[] }>(TauriEvent.DRAG_DROP, (event) => {
        isDragActive.value = false;
        const paths = event.payload?.paths ?? [];
        if (paths.length > 0) onPaths(paths);
      }),
    );
  });

  onBeforeUnmount(() => {
    unlisteners.forEach((unlisten) => unlisten());
  });

  return { isDragActive };
}
