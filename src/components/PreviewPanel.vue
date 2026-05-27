<script setup lang="ts">
import {
  computed,
  inject,
  onBeforeUnmount,
  onMounted,
  ref,
  useTemplateRef,
  watch,
  type Ref,
} from "vue";
import { open, save } from "@tauri-apps/plugin-dialog";
import { previewTv } from "@/components/preview/theme";
import { shellTv } from "@/components/app/theme";
import { useConverter } from "@/composables/useConverter";
import { useHistoryStore } from "@/composables/useHistory";
import { useImagePreview } from "@/composables/useImagePreview";
import {
  clientToNormalizedCrop,
  getDisplayedImageLayout,
  normalizeDragRect,
  normalizedToPixelCrop,
  type NormalizedCrop,
} from "@/lib/previewCrop";
import type {
  CompareViewMode,
  CropRect,
  PreviewResponse,
  VtracerSettings,
} from "@/types/vtracer";
import {
  formatBytes,
  IMAGE_EXTENSIONS,
  isSupportedImage,
} from "@/types/vtracer";

const settings = inject<Ref<VtracerSettings>>("settings")!;
const selectedInputPath = inject<Ref<string>>("selectedInputPath")!;
const lastOutputPath = inject<Ref<string>>("lastOutputPath", ref(""));

const props = defineProps<{
  isDragActive?: boolean;
}>();

const emit = defineEmits<{
  pickFile: [path: string];
}>();

const { convertPreview, convertToSvg } = useConverter();
const historyStore = useHistoryStore();
const imagePreview = useImagePreview();
const ui = previewTv();
const shell = shellTv();

const viewMode = ref<CompareViewMode>("slider");
const sliderPos = ref(50);
const sliderLocked = ref(false);
const overlayOpacity = ref(70);
const preview = ref<PreviewResponse | null>(null);
const previewing = ref(false);
const converting = ref(false);
const message = ref("");
const zoom = ref(100);
const panX = ref(0);
const panY = ref(0);
const spaceHeld = ref(false);
const isPanning = ref(false);
const cropMode = ref(false);
const cropNorm = ref<NormalizedCrop | null>(null);
const cropCommitted = ref(false);
const isDrawingCrop = ref(false);
const layoutTick = ref(0);

const viewportRef = useTemplateRef<HTMLElement>("viewportRef");

let debounceTimer: ReturnType<typeof setTimeout> | null = null;
let panStart = { x: 0, y: 0, panX: 0, panY: 0 };
let cropAnchor: { x: number; y: number } | null = null;

const svgDataUrl = computed(() => {
  if (!preview.value?.svgContent) return "";
  return `data:image/svg+xml;charset=utf-8,${encodeURIComponent(preview.value.svgContent)}`;
});

const metrics = computed(() => {
  if (!preview.value && !imagePreview.width.value) return null;
  const outBytes = preview.value ? new Blob([preview.value.svgContent]).size : 0;
  const inBytes = imagePreview.fileSizeBytes.value;
  const ratio = inBytes > 0 && outBytes > 0 ? ((outBytes / inBytes) * 100).toFixed(0) : "—";
  return {
    width: preview.value?.width ?? imagePreview.width.value,
    height: preview.value?.height ?? imagePreview.height.value,
    paths: preview.value?.pathCount ?? "—",
    elapsed: preview.value?.elapsedMs ?? "—",
    outBytes,
    inBytes,
    ratio,
  };
});

function getImageLayout() {
  const el = viewportRef.value;
  if (!el || !imagePreview.width.value || !imagePreview.height.value) return null;
  const rect = el.getBoundingClientRect();
  return getDisplayedImageLayout(
    rect.width,
    rect.height,
    imagePreview.width.value,
    imagePreview.height.value,
  );
}

function getActiveCropRect(): CropRect | null {
  if (!cropNorm.value) return null;
  return normalizedToPixelCrop(
    cropNorm.value,
    imagePreview.width.value,
    imagePreview.height.value,
  );
}

async function loadPreview(): Promise<void> {
  if (!selectedInputPath.value) return;
  previewing.value = true;
  message.value = "";
  try {
    preview.value = await convertPreview(selectedInputPath.value, settings.value, null);
  } catch (e) {
    message.value = String(e);
    preview.value = null;
  } finally {
    previewing.value = false;
  }
}

async function convertAndSave(): Promise<void> {
  if (!selectedInputPath.value) return;
  converting.value = true;
  try {
    const res = await convertToSvg(
      selectedInputPath.value,
      settings.value,
      null,
      getActiveCropRect(),
    );
    lastOutputPath.value = res.outputPath;
    message.value = `SVG saved · ${res.pathCount} paths · ${formatBytes(res.outputSizeBytes)}`;

    await historyStore.addEntry({
      id: crypto.randomUUID(),
      timestamp: Date.now(),
      mode: "single",
      inputPaths: [selectedInputPath.value],
      outputPaths: [res.outputPath],
      settings: settings.value,
      stats: {
        elapsedMs: Number(res.elapsedMs),
        pathCount: res.pathCount,
        outputBytes: res.outputSizeBytes,
      },
      status: "success",
    });

    await loadPreview();
  } catch (e) {
    message.value = String(e);
  } finally {
    converting.value = false;
  }
}

async function saveAs(): Promise<void> {
  if (!selectedInputPath.value) return;
  const selected = await save({
    title: "Save SVG",
    defaultPath: lastOutputPath.value || undefined,
    filters: [{ name: "SVG", extensions: ["svg"] }],
  });
  if (typeof selected !== "string") return;
  try {
    const res = await convertToSvg(
      selectedInputPath.value,
      settings.value,
      selected,
      getActiveCropRect(),
    );
    lastOutputPath.value = res.outputPath;
    message.value = `Saved: ${res.outputPath}`;
  } catch (e) {
    message.value = String(e);
  }
}

async function chooseFile(): Promise<void> {
  const selected = await open({
    multiple: false,
    filters: [{ name: "Images", extensions: [...IMAGE_EXTENSIONS] }],
  });
  if (typeof selected === "string" && isSupportedImage(selected)) {
    emit("pickFile", selected);
  }
}

function clamp(value: number, min: number, max: number): number {
  return Math.min(Math.max(value, min), max);
}

function setZoom(next: number): void {
  zoom.value = clamp(Math.round(next), 25, 400);
}

function zoomIn(): void {
  setZoom(zoom.value + 25);
}

function zoomOut(): void {
  setZoom(zoom.value - 25);
}

function resetView(): void {
  zoom.value = 100;
  panX.value = 0;
  panY.value = 0;
}

function clearCrop(): void {
  cropNorm.value = null;
  cropAnchor = null;
  cropCommitted.value = false;
  isDrawingCrop.value = false;
}

function startCropRedraw(): void {
  cropNorm.value = null;
  cropAnchor = null;
  cropCommitted.value = false;
  isDrawingCrop.value = false;
}

function resetZoom(): void {
  resetView();
}

function onViewportWheel(event: WheelEvent): void {
  const direction = event.deltaY < 0 ? 1 : -1;
  setZoom(zoom.value + direction * 10);
}

const viewportTransform = computed(() => ({
  transform: `translate(${panX.value}px, ${panY.value}px) scale(${zoom.value / 100})`,
  transformOrigin: "center center",
}));

function shouldStartPan(event: PointerEvent): boolean {
  return event.button === 1 || (event.button === 0 && (spaceHeld.value || event.altKey));
}

const imageLayoutStyle = computed(() => {
  void layoutTick.value;
  const layout = getImageLayout();
  if (!layout) return null;
  return {
    left: `${layout.left}px`,
    top: `${layout.top}px`,
    width: `${layout.width}px`,
    height: `${layout.height}px`,
  };
});

const cropBoxPercentStyle = computed(() => {
  if (!cropNorm.value) return null;
  if (
    !isDrawingCrop.value &&
    (cropNorm.value.w < 0.005 || cropNorm.value.h < 0.005)
  ) {
    return null;
  }
  return {
    left: `${cropNorm.value.x * 100}%`,
    top: `${cropNorm.value.y * 100}%`,
    width: `${cropNorm.value.w * 100}%`,
    height: `${cropNorm.value.h * 100}%`,
  };
});

const cropSizeLabel = computed(() => {
  const rect = getActiveCropRect();
  if (!rect) return "";
  return `${rect.width}×${rect.height} px`;
});

const cropHintText = computed(() => {
  if (isDrawingCrop.value) return "Release mouse to fix crop for save";
  if (cropCommitted.value) {
    return `SVG preview is full image · on save: ${cropSizeLabel.value || "crop region"}`;
  }
  return "Preview always uses the full image · crop applies on save only";
});

function bumpLayout(): void {
  layoutTick.value += 1;
}

function onCropPointerDown(event: PointerEvent): void {
  if (!cropMode.value || spaceHeld.value || shouldStartPan(event)) return;
  if (cropCommitted.value) return;

  const el = viewportRef.value;
  const layout = getImageLayout();
  if (!el || !layout) return;

  event.preventDefault();
  event.stopPropagation();
  const pt = clientToNormalizedCrop(event.clientX, event.clientY, el.getBoundingClientRect(), layout);
  if (!pt) return;

  isDrawingCrop.value = true;
  cropAnchor = pt;
  cropNorm.value = { x: pt.x, y: pt.y, w: 0, h: 0 };
  (event.currentTarget as HTMLElement).setPointerCapture(event.pointerId);
}

function onCropPointerMove(event: PointerEvent): void {
  if (!isDrawingCrop.value || !cropAnchor) return;
  const el = viewportRef.value;
  const layout = getImageLayout();
  if (!el || !layout) return;

  const pt = clientToNormalizedCrop(
    event.clientX,
    event.clientY,
    el.getBoundingClientRect(),
    layout,
    true,
  );
  if (!pt) return;
  cropNorm.value = normalizeDragRect(cropAnchor.x, cropAnchor.y, pt.x, pt.y);
}

function onCropPointerUp(event: PointerEvent): void {
  if (!isDrawingCrop.value) return;
  isDrawingCrop.value = false;
  cropAnchor = null;
  const target = event.currentTarget as HTMLElement;
  if (target.hasPointerCapture(event.pointerId)) {
    target.releasePointerCapture(event.pointerId);
  }
  if (cropNorm.value && (cropNorm.value.w < 0.01 || cropNorm.value.h < 0.01)) {
    cropNorm.value = null;
    cropCommitted.value = false;
    return;
  }
  cropCommitted.value = true;
}

function onViewportPointerDown(event: PointerEvent): void {
  if (cropMode.value && !spaceHeld.value) return;
  if (!shouldStartPan(event)) return;
  event.preventDefault();
  isPanning.value = true;
  panStart = { x: event.clientX, y: event.clientY, panX: panX.value, panY: panY.value };
  (event.currentTarget as HTMLElement).setPointerCapture(event.pointerId);
}

function onViewportPointerMove(event: PointerEvent): void {
  if (!isPanning.value) return;
  panX.value = panStart.panX + (event.clientX - panStart.x);
  panY.value = panStart.panY + (event.clientY - panStart.y);
}

function onViewportPointerUp(event: PointerEvent): void {
  if (!isPanning.value) return;
  isPanning.value = false;
  if ((event.currentTarget as HTMLElement).hasPointerCapture(event.pointerId)) {
    (event.currentTarget as HTMLElement).releasePointerCapture(event.pointerId);
  }
}

function onKeyDown(event: KeyboardEvent): void {
  if (event.code === "Space" && !(event.target instanceof HTMLInputElement)) {
    spaceHeld.value = true;
    event.preventDefault();
  }
}

function onKeyUp(event: KeyboardEvent): void {
  if (event.code === "Space") spaceHeld.value = false;
}

let layoutObserver: ResizeObserver | null = null;

onMounted(() => {
  window.addEventListener("keydown", onKeyDown);
  window.addEventListener("keyup", onKeyUp);
});

onBeforeUnmount(() => {
  window.removeEventListener("keydown", onKeyDown);
  window.removeEventListener("keyup", onKeyUp);
  layoutObserver?.disconnect();
});

watch(
  viewportRef,
  (el) => {
    layoutObserver?.disconnect();
    layoutObserver = null;
    if (!el) return;
    layoutObserver = new ResizeObserver(() => bumpLayout());
    layoutObserver.observe(el);
    bumpLayout();
  },
  { flush: "post" },
);

watch([zoom, panX, panY, viewMode], () => bumpLayout());

function scheduleLivePreview(): void {
  if (debounceTimer) clearTimeout(debounceTimer);
  debounceTimer = setTimeout(() => {
    if (selectedInputPath.value) void loadPreview();
  }, 400);
}

watch(
  () => selectedInputPath.value,
  async (path) => {
    preview.value = null;
    if (!path) {
      imagePreview.clear();
      return;
    }
    cropNorm.value = null;
    cropCommitted.value = false;
    await imagePreview.loadPreview(path, settings.value.trimEmptySpace);
    resetView();
    scheduleLivePreview();
  },
  { immediate: true },
);

watch(settings, scheduleLivePreview, { deep: true });

watch(
  () => settings.value.trimEmptySpace,
  async () => {
    if (!selectedInputPath.value) return;
    imagePreview.invalidateCache(selectedInputPath.value);
    await imagePreview.loadPreview(selectedInputPath.value, settings.value.trimEmptySpace);
    scheduleLivePreview();
  },
);

watch(viewMode, (mode) => {
  if (mode !== "slider") sliderLocked.value = false;
});

watch(cropMode, (enabled) => {
  if (!enabled) clearCrop();
});

function getSliderRoot(event: Event): HTMLElement | null {
  return (event.currentTarget as HTMLElement).closest("[data-slider-root]");
}

function updateSliderPos(event: { clientX: number }, root: HTMLElement): void {
  const rect = root.getBoundingClientRect();
  const x = Math.min(Math.max(event.clientX - rect.left, 0), rect.width);
  sliderPos.value = Math.round((x / rect.width) * 100);
}

function onSliderMove(event: PointerEvent): void {
  if (sliderLocked.value) return;
  const root = getSliderRoot(event);
  if (!root) return;
  updateSliderPos(event, root);
}

function onSliderClick(event: MouseEvent): void {
  if (cropMode.value || spaceHeld.value || isPanning.value) return;

  const root = getSliderRoot(event);
  if (!root) return;

  if (sliderLocked.value) {
    sliderLocked.value = false;
    return;
  }

  updateSliderPos(event, root);
  sliderLocked.value = true;
}

const sliderUi = computed(() => previewTv({ sliderLocked: sliderLocked.value }));

defineExpose({ convertAndSave, chooseFile, saveAs });
</script>

<template>
  <div :class="ui.root()">
    <div v-if="selectedInputPath" :class="ui.toolbar()">
      <div :class="ui.segmented()">
        <button
          type="button"
          :class="previewTv({ segmentActive: viewMode === 'slider' }).segment()"
          @click="viewMode = 'slider'"
        >
          Slider
        </button>
        <button
          type="button"
          :class="previewTv({ segmentActive: viewMode === 'sideBySide' }).segment()"
          @click="viewMode = 'sideBySide'"
        >
          Side by side
        </button>
        <button
          type="button"
          :class="previewTv({ segmentActive: viewMode === 'overlay' }).segment()"
          @click="viewMode = 'overlay'"
        >
          Overlay
        </button>
      </div>
      <template v-if="viewMode === 'overlay'">
        <label class="flex items-center gap-2 text-xs text-neutral-400">
          Opacity
          <input v-model.number="overlayOpacity" type="range" min="0" max="100" class="w-24 accent-blue-500" />
        </label>
      </template>
      <span v-if="viewMode === 'slider'" class="text-xs text-neutral-500">
        Click preview to lock split · click again to unlock
      </span>
      <button
        type="button"
        :class="shell.button()"
        :title="cropMode ? 'Turn off crop selection' : 'Select crop region for conversion on save'"
        @click="cropMode = !cropMode"
      >
        {{ cropMode ? "Crop: on" : "Crop" }}
      </button>
      <button v-if="cropCommitted" type="button" :class="shell.button()" @click="startCropRedraw">
        Redraw
      </button>
      <button v-if="cropNorm" type="button" :class="shell.button()" @click="clearCrop">
        Reset crop
      </button>
      <span v-if="cropSizeLabel" class="text-xs text-blue-300">{{ cropSizeLabel }}</span>
      <span class="text-xs text-neutral-500">Space/Alt — pan · wheel — zoom</span>
      <div class="ml-auto flex items-center gap-1">
        <button type="button" :class="shell.button()" @click="zoomOut">-</button>
        <button type="button" :class="shell.button()" title="Reset zoom and pan" @click="resetZoom">
          {{ zoom }}%
        </button>
        <button type="button" :class="shell.button()" @click="zoomIn">+</button>
      </div>
    </div>

    <div :class="ui.viewportWrap()">
      <div
        v-if="!selectedInputPath"
        :class="previewTv({ dragActive: props.isDragActive }).empty()"
      >
        <span :class="ui.emptyIcon()">🖼</span>
        <p :class="ui.emptyTitle()">Drop an image here</p>
        <p :class="ui.emptyHint()">PNG, JPG, WebP, BMP, GIF, TIFF — see the original and live SVG preview</p>
        <button type="button" :class="shell.buttonPrimary()" @click="chooseFile">Choose file</button>
      </div>

      <div
        v-else
        ref="viewportRef"
        :class="ui.viewport()"
        @wheel.prevent="onViewportWheel"
        @pointerdown="onViewportPointerDown"
        @pointermove="onViewportPointerMove"
        @pointerup="onViewportPointerUp"
        @pointercancel="onViewportPointerUp"
      >
        <div v-if="previewing || imagePreview.loading.value" :class="ui.loading()">
          <span class="text-sm text-neutral-300">Loading…</span>
        </div>

        <div
          v-if="viewMode === 'slider'"
          :class="sliderUi.sliderRoot()"
          data-slider-root
          @pointermove="onSliderMove"
          @click="onSliderClick"
        >
          <div :class="ui.sliderLayer()">
            <div :class="ui.sliderImageWrap()" :style="viewportTransform">
              <img v-if="svgDataUrl" :src="svgDataUrl" alt="after" :class="ui.sliderMedia()" />
              <div v-else :class="ui.placeholder()">SVG preview</div>
            </div>
          </div>
          <div
            :class="ui.sliderBeforeClip()"
            :style="{ clipPath: `inset(0 ${100 - sliderPos}% 0 0)` }"
          >
            <div :class="ui.sliderImageWrap()" :style="viewportTransform">
              <img
                v-if="imagePreview.dataUrl.value"
                :src="imagePreview.dataUrl.value"
                alt="before"
                :class="ui.sliderMedia()"
              />
            </div>
          </div>
          <div :class="sliderUi.sliderHandle()" :style="{ left: `${sliderPos}%` }">
            <div :class="sliderUi.sliderKnob()" />
          </div>
          <p :class="ui.sliderHint()">
            {{
              sliderLocked
                ? "Locked · click to unlock"
                : "Hover or drag · click to lock"
            }}
          </p>
        </div>

        <div v-else :class="ui.panLayer()">
          <div :class="ui.panInner()" :style="viewportTransform">
            <div v-if="viewMode === 'sideBySide'" :class="ui.split()">
              <div :class="ui.pane()">
                <span :class="ui.paneLabel()">Before</span>
                <img
                  v-if="imagePreview.dataUrl.value"
                  :src="imagePreview.dataUrl.value"
                  alt="original"
                  :class="ui.media()"
                />
                <div v-else-if="imagePreview.error.value" :class="ui.placeholder()">
                  {{ imagePreview.error.value }}
                </div>
              </div>
              <div :class="ui.pane()">
                <span :class="ui.paneLabel()">After</span>
                <img v-if="svgDataUrl" :src="svgDataUrl" alt="svg" :class="ui.media()" />
                <div v-else :class="ui.placeholder()">Generating preview…</div>
              </div>
            </div>

            <div v-else class="relative flex h-full w-full items-center justify-center">
              <img
                v-if="imagePreview.dataUrl.value"
                :src="imagePreview.dataUrl.value"
                alt="original"
                :class="ui.media()"
              />
              <div
                v-if="svgDataUrl"
                :class="ui.overlaySvg()"
                :style="{ opacity: overlayOpacity / 100 }"
              >
                <img :src="svgDataUrl" alt="svg overlay" :class="ui.media()" />
              </div>
            </div>
          </div>
        </div>

        <div v-if="cropMode" :class="ui.cropOverlay()">
          <div
            v-if="imageLayoutStyle"
            :class="ui.cropStage()"
            :style="[
              imageLayoutStyle,
              {
                pointerEvents: spaceHeld ? 'none' : 'auto',
                cursor: cropCommitted ? 'default' : 'crosshair',
              },
            ]"
            @pointerdown="onCropPointerDown"
            @pointermove="onCropPointerMove"
            @pointerup="onCropPointerUp"
            @pointercancel="onCropPointerUp"
          >
            <div v-if="cropBoxPercentStyle" :class="ui.cropBox()" :style="cropBoxPercentStyle" />
            <span v-if="cropSizeLabel && cropCommitted" :class="ui.cropBadge()">
              {{ cropSizeLabel }}
            </span>
            <p :class="ui.cropHint()">{{ cropHintText }}</p>
          </div>
        </div>
      </div>
    </div>

    <div v-if="metrics" :class="ui.metrics()">
      <div :class="ui.metric()">
        <span :class="ui.metricLabel()">Dimensions</span>
        <span :class="ui.metricValue()">{{ metrics.width }}×{{ metrics.height }}</span>
      </div>
      <div :class="ui.metric()">
        <span :class="ui.metricLabel()">Paths</span>
        <span :class="ui.metricValue()">{{ metrics.paths }}</span>
      </div>
      <div :class="ui.metric()">
        <span :class="ui.metricLabel()">File: source → SVG</span>
        <span :class="ui.metricValue()">
          {{ formatBytes(metrics.inBytes) }} →
          {{ metrics.outBytes ? formatBytes(metrics.outBytes) : "—" }}
          <template v-if="metrics.ratio !== '—'"> ({{ metrics.ratio }}%)</template>
        </span>
      </div>
      <div :class="ui.metric()">
        <span :class="ui.metricLabel()">Time</span>
        <span :class="ui.metricValue()">{{ metrics.elapsed }} ms</span>
      </div>
    </div>

    <p v-if="message" :class="ui.message()">{{ message }}</p>
  </div>
</template>
