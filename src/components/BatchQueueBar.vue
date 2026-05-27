<script setup lang="ts">
import { computed, inject, onBeforeUnmount, onMounted, ref, watch, type Ref } from "vue";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { revealItemInDir } from "@tauri-apps/plugin-opener";
import { batchBarTv } from "@/components/preview/theme";
import { shellTv } from "@/components/app/theme";
import { useConverter } from "@/composables/useConverter";
import { useHistoryStore } from "@/composables/useHistory";
import type {
  BatchProgressPayload,
  BatchQueueItem,
  OutputMode,
  VtracerSettings,
} from "@/types/vtracer";
import { fileName, IMAGE_EXTENSIONS, isSupportedImage } from "@/types/vtracer";

const settings = inject<Ref<VtracerSettings>>("settings")!;
const queuePaths = inject<Ref<string[]>>("queuePaths")!;
const selectedInputPath = inject<Ref<string>>("selectedInputPath")!;

const emit = defineEmits<{
  selectPath: [path: string];
}>();

const { convertBatch, cancelBatch } = useConverter();
const historyStore = useHistoryStore();
const ui = batchBarTv();
const shell = shellTv();

const outputMode = ref<OutputMode>("same_dir");
const outputDir = ref("");
const running = ref(false);
const progress = ref({ current: 0, total: 0 });
const message = ref("");
const lastResultDir = ref("");

const items = ref<BatchQueueItem[]>([]);

const progressPercent = computed(() =>
  progress.value.total > 0
    ? Math.round((progress.value.current / progress.value.total) * 100)
    : 0,
);

let unlistenProgress: UnlistenFn | null = null;

function syncItemsFromPaths(): void {
  const existing = new Map(items.value.map((i) => [i.path, i]));
  items.value = queuePaths.value.map((path) => {
    const prev = existing.get(path);
    return prev ?? { path, status: "pending" as const };
  });
}

function addPaths(paths: string[]): void {
  const valid = paths.filter(isSupportedImage);
  const merged = new Set([...queuePaths.value, ...valid]);
  queuePaths.value = [...merged];
  syncItemsFromPaths();
  message.value = `Queued: ${queuePaths.value.length} files`;
}

async function pickFiles(): Promise<void> {
  const selected = await open({
    multiple: true,
    filters: [{ name: "Images", extensions: [...IMAGE_EXTENSIONS] }],
  });
  if (Array.isArray(selected)) addPaths(selected);
  else if (typeof selected === "string") addPaths([selected]);
}

async function pickFolder(): Promise<void> {
  const selected = await open({ directory: true, multiple: false });
  if (typeof selected !== "string") return;
  const scanned = await invoke<string[]>("scan_images_in_directory", { dirPath: selected });
  addPaths(scanned);
}

async function pickOutputDir(): Promise<void> {
  const selected = await open({ directory: true, multiple: false });
  if (typeof selected === "string") outputDir.value = selected;
}

function clearQueue(): void {
  queuePaths.value = [];
  items.value = [];
  message.value = "Queue cleared";
}

function selectItem(path: string): void {
  selectedInputPath.value = path;
  emit("selectPath", path);
}

async function runBatch(): Promise<void> {
  if (items.value.length === 0) return;
  if (outputMode.value === "output_dir" && !outputDir.value) {
    message.value = "Choose an output folder";
    return;
  }

  running.value = true;
  progress.value = { current: 0, total: items.value.length };
  items.value = items.value.map((i) => ({
    ...i,
    status: "pending" as const,
    error: undefined,
    outputPath: undefined,
  }));

  try {
    const result = await convertBatch(
      items.value.map((i) => i.path),
      settings.value,
      outputMode.value,
      outputDir.value || null,
    );

    for (const ok of result.succeeded) {
      const inputPath = ok.inputPath ?? "";
      const item = items.value.find((i) => i.path === inputPath);
      if (item) {
        item.status = "done";
        item.outputPath = ok.outputPath;
        item.elapsedMs = Number(ok.elapsedMs);
      }
    }

    for (const fail of result.failed) {
      const item = items.value.find((i) => i.path === fail.path);
      if (item) {
        item.status = "error";
        item.error = fail.error;
      }
    }

    if (result.succeeded.length > 0) {
      const first = result.succeeded[0].outputPath;
      lastResultDir.value = first.includes("/")
        ? first.substring(0, first.lastIndexOf("/"))
        : first.includes("\\")
          ? first.substring(0, first.lastIndexOf("\\"))
          : outputDir.value;
    }

    const status =
      result.failed.length === 0
        ? "success"
        : result.succeeded.length > 0
          ? "partial"
          : "error";
    message.value = `${result.succeeded.length} OK, ${result.failed.length} failed · ${result.totalElapsedMs} ms`;

    await historyStore.addEntry({
      id: crypto.randomUUID(),
      timestamp: Date.now(),
      mode: "batch",
      inputPaths: items.value.map((i) => i.path),
      outputPaths: result.succeeded.map((s) => s.outputPath),
      settings: settings.value,
      stats: { elapsedMs: Number(result.totalElapsedMs) },
      status,
    });
  } catch (e) {
    message.value = String(e);
  } finally {
    running.value = false;
  }
}

async function stopBatch(): Promise<void> {
  await cancelBatch();
  message.value = "Cancelling…";
}

async function openResultsFolder(): Promise<void> {
  const path =
    outputMode.value === "output_dir" && outputDir.value
      ? outputDir.value
      : (items.value.find((i) => i.outputPath)?.outputPath ?? lastResultDir.value);
  if (!path) return;
  try {
    await revealItemInDir(path);
  } catch {
    message.value = "Could not reveal in file manager";
  }
}

function statusClass(status: BatchQueueItem["status"]): string {
  const map: Record<BatchQueueItem["status"], string> = {
    pending: "bg-neutral-700 text-neutral-300",
    running: "bg-blue-500/20 text-blue-300",
    done: "bg-green-500/20 text-green-300",
    error: "bg-red-500/20 text-red-300",
  };
  return map[status];
}

onMounted(async () => {
  unlistenProgress = await listen<BatchProgressPayload>("batch-progress", (event) => {
    const p = event.payload;
    progress.value = { current: p.index + 1, total: p.total };
    const item = items.value.find((i) => i.path === p.path);
    if (item) {
      item.status = p.status;
      if (p.message) item.error = p.message;
    }
  });
  syncItemsFromPaths();
});

onBeforeUnmount(() => {
  unlistenProgress?.();
});

watch(queuePaths, syncItemsFromPaths, { deep: true });

defineExpose({ addPaths });
</script>

<template>
  <div :class="ui.root()">
    <div :class="ui.header()">
      <span :class="ui.title()">Batch queue ({{ items.length }})</span>
      <div :class="ui.toolbar()">
        <button type="button" :class="shell.button()" :disabled="running" @click="pickFiles">
          Files
        </button>
        <button type="button" :class="shell.button()" :disabled="running" @click="pickFolder">
          Folder
        </button>
        <button type="button" :class="shell.button()" :disabled="running" @click="clearQueue">
          Clear
        </button>
        <label class="flex items-center gap-1 text-xs text-neutral-400">
          <input v-model="outputMode" type="radio" value="same_dir" :disabled="running" />
          Next to source
        </label>
        <label class="flex items-center gap-1 text-xs text-neutral-400">
          <input v-model="outputMode" type="radio" value="output_dir" :disabled="running" />
          Into folder
        </label>
        <button
          v-if="outputMode === 'output_dir'"
          type="button"
          :class="shell.button()"
          :disabled="running"
          @click="pickOutputDir"
        >
          {{ outputDir ? fileName(outputDir) : "Folder…" }}
        </button>
      </div>
    </div>

    <div :class="ui.body()">
      <div
        v-for="item in items"
        :key="item.path"
        :class="[ui.row(), selectedInputPath === item.path ? ui.rowActive() : '']"
        :title="item.error ?? item.path"
        @click="selectItem(item.path)"
      >
        <span class="min-w-0 flex-1 truncate">{{ fileName(item.path) }}</span>
        <span :class="[ui.status(), statusClass(item.status)]">
          {{
            item.status === "pending"
              ? "pending"
              : item.status === "running"
                ? "running"
                : item.status === "done"
                  ? "done"
                  : "error"
          }}
        </span>
      </div>
      <p v-if="items.length === 0" class="p-3 text-center text-xs text-neutral-500">
        Add files or drop multiple images here
      </p>
    </div>

    <div :class="ui.footer()">
      <div v-if="running" :class="ui.progressTrack()">
        <div :class="ui.progressFill()" :style="{ width: `${progressPercent}%` }" />
      </div>
      <span v-if="running" class="shrink-0 text-xs text-neutral-400">
        {{ progress.current }}/{{ progress.total }}
      </span>
      <button
        type="button"
        :class="shell.buttonPrimary()"
        :disabled="running || items.length === 0"
        @click="runBatch"
      >
        Convert all
      </button>
      <button type="button" :class="shell.button()" :disabled="!running" @click="stopBatch">
        Cancel
      </button>
      <button type="button" :class="shell.button()" @click="openResultsFolder">
        Open folder
      </button>
      <span v-if="message" class="min-w-0 truncate text-xs text-neutral-500">{{ message }}</span>
    </div>
  </div>
</template>
