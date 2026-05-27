<script setup lang="ts">
import { computed, provide, ref, useTemplateRef } from "vue";
import { shellTv } from "@/components/app/theme";
import Workbench from "@/components/Workbench.vue";
import HistoryDrawer from "@/components/HistoryDrawer.vue";
import { useDragDrop } from "@/composables/useDragDrop";
import { useHistoryStore } from "@/composables/useHistory";
import { usePersistedSettings } from "@/composables/usePersistedSettings";
import type { AppTab } from "@/types/vtracer";
import { isSupportedImage } from "@/types/vtracer";

defineOptions({ name: "App" });

const activeMode = ref<AppTab>("file");
const { settings } = usePersistedSettings();
const selectedInputPath = ref("");
const lastOutputPath = ref("");
const queuePaths = ref<string[]>([]);
const historyOpen = ref(false);

const workbenchRef = useTemplateRef<InstanceType<typeof Workbench>>("workbenchRef");

provide("settings", settings);
provide("selectedInputPath", selectedInputPath);
provide("lastOutputPath", lastOutputPath);
provide("queuePaths", queuePaths);

const historyStore = useHistoryStore();
void historyStore.refresh();

function handleDroppedPaths(paths: string[]): void {
  const images = paths.filter(isSupportedImage);
  if (images.length === 0) return;

  if (activeMode.value === "batch" || images.length > 1) {
    activeMode.value = "batch";
    workbenchRef.value?.addBatchPaths(images);
    if (images.length === 1) selectedInputPath.value = images[0];
    else if (!selectedInputPath.value && images[0]) selectedInputPath.value = images[0];
    return;
  }

  selectedInputPath.value = images[0];
}

const { isDragActive } = useDragDrop(handleDroppedPaths);

const ui = computed(() => shellTv({ dragActive: isDragActive.value }));

const modes: { id: AppTab; label: string }[] = [
  { id: "file", label: "File" },
  { id: "batch", label: "Batch" },
];
</script>

<template>
  <div :class="ui.root()">
    <header :class="ui.header()">
      <div :class="ui.headerLeft()">
        <h1 :class="ui.title()">VTracer</h1>
        <p :class="ui.subtitle()">Raster to SVG · before/after preview · batch · history</p>
      </div>
      <div :class="ui.headerActions()">
        <nav :class="ui.modeTabs()">
          <button
            v-for="m in modes"
            :key="m.id"
            type="button"
            :class="shellTv({ modeTabActive: activeMode === m.id }).modeTab()"
            @click="activeMode = m.id"
          >
            {{ m.label }}
          </button>
        </nav>
        <button type="button" :class="ui.button()" @click="historyOpen = true">
          History
        </button>
      </div>
    </header>

    <div :class="ui.body()">
      <Workbench
        ref="workbenchRef"
        :mode="activeMode"
        :is-drag-active="isDragActive"
      />
    </div>

    <HistoryDrawer v-model:open="historyOpen" @switch-mode="activeMode = $event" />
  </div>
</template>
