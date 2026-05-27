<script setup lang="ts">
import { inject, ref, useTemplateRef, type Ref } from "vue";
import { shellTv } from "@/components/app/theme";
import PreviewPanel from "@/components/PreviewPanel.vue";
import SettingsPanel from "@/components/SettingsPanel.vue";
import BatchQueueBar from "@/components/BatchQueueBar.vue";
import { useHistoryStore } from "@/composables/useHistory";
import type { AppTab, VtracerSettings } from "@/types/vtracer";
import { isSupportedImage } from "@/types/vtracer";

defineProps<{
  mode: AppTab;
  isDragActive?: boolean;
}>();

const settings = inject<Ref<VtracerSettings>>("settings")!;
const selectedInputPath = inject<Ref<string>>("selectedInputPath")!;

const historyStore = useHistoryStore();
const shellUi = shellTv();

const previewRef = useTemplateRef<InstanceType<typeof PreviewPanel>>("previewRef");
const batchRef = useTemplateRef<InstanceType<typeof BatchQueueBar>>("batchRef");
const converting = ref(false);

function onPickFile(path: string): void {
  if (!isSupportedImage(path)) return;
  selectedInputPath.value = path;
}

function onConvert(): void {
  converting.value = true;
  previewRef.value
    ?.convertAndSave()
    .finally(() => {
      converting.value = false;
    });
}

function onSaveAs(): void {
  previewRef.value?.saveAs();
}

async function onCreatePreset(): Promise<void> {
  const name = window.prompt("Preset name:");
  if (!name?.trim()) return;
  await historyStore.savePreset(name.trim(), settings.value);
}

async function onDeletePreset(id: string): Promise<void> {
  await historyStore.deletePreset(id);
}

defineExpose({
  addBatchPaths(paths: string[]) {
    batchRef.value?.addPaths(paths);
  },
});
</script>

<template>
  <div :class="shellUi.workbench()">
    <div :class="shellUi.split()">
      <div :class="shellTv({ dragActive: isDragActive }).previewCol()">
        <PreviewPanel
          ref="previewRef"
          :is-drag-active="isDragActive"
          @pick-file="onPickFile"
        />
      </div>
      <aside :class="shellUi.settingsCol()">
        <SettingsPanel
          v-model:settings="settings"
          :converting="converting"
          :named-presets="historyStore.presets.value"
          @convert="onConvert"
          @save-as="onSaveAs"
          @create-preset="onCreatePreset"
          @delete-preset="onDeletePreset"
        />
      </aside>
    </div>
    <BatchQueueBar v-if="mode === 'batch'" ref="batchRef" />
  </div>
</template>
