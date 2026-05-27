<script setup lang="ts">
import { computed, inject, onMounted, ref, type Ref } from "vue";
import { revealItemInDir } from "@tauri-apps/plugin-opener";
import { drawerTv, panelTv, shellTv } from "@/components/app/theme";
import { useHistoryStore } from "@/composables/useHistory";
import type { AppTab, HistoryEntry, VtracerSettings } from "@/types/vtracer";
import { fileName, formatBytes } from "@/types/vtracer";

const open = defineModel<boolean>("open", { default: false });

const settings = inject<Ref<VtracerSettings>>("settings")!;
const selectedInputPath = inject<Ref<string>>("selectedInputPath")!;
const queuePaths = inject<Ref<string[]>>("queuePaths")!;

const emit = defineEmits<{
  switchMode: [mode: AppTab];
}>();

const historyStore = useHistoryStore();
const ui = drawerTv();
const panel = panelTv();
const shell = shellTv();

const filterMode = ref<"" | "single" | "batch">("");
const filterStatus = ref<"" | "success" | "partial" | "error">("");

onMounted(() => {
  void historyStore.refresh();
});

const filteredEntries = computed(() =>
  historyStore.entries.value.filter((e) => {
    if (filterMode.value && e.mode !== filterMode.value) return false;
    if (filterStatus.value && e.status !== filterStatus.value) return false;
    return true;
  }),
);

function applyPresetSettings(s: VtracerSettings): void {
  settings.value = { ...s };
}

function rerunEntry(entry: HistoryEntry): void {
  applyPresetSettings(entry.settings);
  if (entry.mode === "batch") {
    queuePaths.value = [...entry.inputPaths];
    emit("switchMode", "batch");
  } else if (entry.inputPaths[0]) {
    selectedInputPath.value = entry.inputPaths[0];
    emit("switchMode", "file");
  }
  open.value = false;
}

async function openOutput(path: string): Promise<void> {
  try {
    await revealItemInDir(path);
  } catch {
    /* ignore */
  }
}

function close(): void {
  open.value = false;
}
</script>

<template>
  <Teleport to="body">
    <div v-if="open" :class="ui.backdrop()" @click="close" />
    <aside v-if="open" :class="ui.panel()">
      <div :class="ui.header()">
        <h2 :class="ui.title()">History</h2>
        <button type="button" :class="shell.buttonGhost()" @click="close">✕</button>
      </div>

      <div :class="ui.body()">
        <section>
          <div class="mb-3 flex flex-wrap items-center justify-between gap-2">
            <h3 :class="panel.panelTitle()">Conversion log</h3>
            <button type="button" :class="shell.button()" @click="historyStore.clearHistory()">
              Clear
            </button>
          </div>
          <div :class="panel.actions()" class="mb-3">
            <select v-model="filterMode" :class="panel.control()">
              <option value="">All modes</option>
              <option value="single">File</option>
              <option value="batch">Batch</option>
            </select>
            <select v-model="filterStatus" :class="panel.control()">
              <option value="">All statuses</option>
              <option value="success">Success</option>
              <option value="partial">Partial</option>
              <option value="error">Error</option>
            </select>
          </div>

          <div class="space-y-2">
            <article
              v-for="entry in filteredEntries"
              :key="entry.id"
              class="rounded-lg border border-neutral-800 p-2 text-xs"
            >
              <div class="flex justify-between gap-2">
                <span class="font-medium text-neutral-200">
                  {{ entry.mode === "single" ? "File" : "Batch" }} · {{ entry.status }}
                </span>
                <span class="text-neutral-500">{{ new Date(entry.timestamp).toLocaleString() }}</span>
              </div>
              <p :class="panel.pathLine()">
                {{ entry.inputPaths.length }} in → {{ entry.outputPaths.length }} out
                <template v-if="entry.stats.elapsedMs"> · {{ entry.stats.elapsedMs }} ms</template>
                <template v-if="entry.stats.outputBytes">
                  · {{ formatBytes(entry.stats.outputBytes) }}
                </template>
              </p>
              <p v-if="entry.outputPaths[0]" :class="panel.pathLine()">
                {{ fileName(entry.outputPaths[0]) }}
              </p>
              <div :class="panel.actions()" class="mt-2">
                <button type="button" :class="shell.button()" @click="applyPresetSettings(entry.settings)">
                  Settings
                </button>
                <button type="button" :class="shell.button()" @click="rerunEntry(entry)">Rerun</button>
                <button
                  v-if="entry.outputPaths[0]"
                  type="button"
                  :class="shell.button()"
                  @click="openOutput(entry.outputPaths[0])"
                >
                  Open
                </button>
                <button type="button" :class="shell.button()" @click="historyStore.removeEntry(entry.id)">
                  Delete
                </button>
              </div>
            </article>
            <p v-if="filteredEntries.length === 0" :class="panel.hint()">No history yet</p>
          </div>
        </section>
      </div>
    </aside>
  </Teleport>
</template>
