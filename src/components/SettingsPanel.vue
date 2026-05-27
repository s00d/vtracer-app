<script setup lang="ts">
import { ref } from "vue";
import { panelTv, shellTv } from "@/components/app/theme";
import {
  applyVtracerPreset,
  type NamedPreset,
  type VtracerPreset,
  type VtracerSettings,
} from "@/types/vtracer";

const settings = defineModel<VtracerSettings>("settings", { required: true });

defineProps<{
  converting?: boolean;
  namedPresets?: NamedPreset[];
}>();

const emit = defineEmits<{
  convert: [];
  saveAs: [];
  createPreset: [];
  deletePreset: [id: string];
}>();

const ui = panelTv();
const shell = shellTv();

const openSections = ref<Record<string, boolean>>({
  preset: true,
  color: true,
  curves: false,
  output: false,
});

const PARAM_HINTS: Record<string, string> = {
  preset: "Quick vtracer profiles: bw, poster, photo",
  colormode: "color — full color; bw — binary (black/white)",
  hierarchical: "stacked — layers on top of each other; cutout — separate cutouts",
  mode: "pixel — no smoothing; polygon — lines; spline — Bézier curves",
  filterSpeckle: "Drops speckles smaller than N pixels [0–16]",
  colorPrecision: "Significant RGB bits [1–8], higher — more colors",
  gradientStep: "Gradient step between layers [0–255]",
  cornerThreshold: "Angle threshold for corners [0–180°]",
  segmentLength: "Max smoothing segment length [3.5–10]",
  spliceThreshold: "Angle for splitting splines [0–180°]",
  pathPrecision: "Decimal places in path output [0–12]",
  maxIterations: "Spline fitting iterations [1–32]",
  trimEmptySpace: "Trim transparent margins on PNG/WebP before conversion",
  svgoOptimize: "Optimize SVG on save (SVGO or built-in minifier)",
};

function toggleSection(id: string): void {
  openSections.value[id] = !openSections.value[id];
}

function onPresetChange(value: string): void {
  const next = applyVtracerPreset(settings.value, value as VtracerPreset);
  // Keep the same object: some fields are mutated via v-model on the panel.
  Object.assign(settings.value, next);
}

function applyNamedPreset(preset: NamedPreset): void {
  Object.assign(settings.value, preset.settings);
}

function onDeletePreset(id: string, event: MouseEvent): void {
  event.stopPropagation();
  emit("deletePreset", id);
}
</script>

<template>
  <section :class="ui.panel()">
    <div :class="ui.scroll()">
      <div :class="ui.section()">
        <button type="button" :class="ui.sectionHeader()" @click="toggleSection('preset')">
          <span>Preset</span>
          <span class="text-neutral-500">{{ openSections.preset ? "−" : "+" }}</span>
        </button>
        <div v-show="openSections.preset" :class="ui.sectionBody()">
          <label :class="ui.field()" :title="PARAM_HINTS.preset">
            <span :class="ui.fieldLabel()">VTracer preset</span>
            <select
              :class="ui.control()"
              :value="settings.preset"
              @change="onPresetChange(($event.target as HTMLSelectElement).value)"
            >
              <option value="">Custom</option>
              <option value="bw">bw</option>
              <option value="poster">poster</option>
              <option value="photo">photo</option>
            </select>
          </label>
          <div :class="ui.field()">
            <span :class="ui.fieldLabel()">My presets</span>
            <div :class="ui.presetList()">
              <div
                v-for="preset in namedPresets"
                :key="preset.id"
                role="button"
                tabindex="0"
                :class="ui.presetItem()"
                :title="`Apply “${preset.name}”`"
                @click="applyNamedPreset(preset)"
                @keydown.enter="applyNamedPreset(preset)"
              >
                <span :class="ui.presetItemName()">{{ preset.name }}</span>
                <button
                  type="button"
                  :class="ui.presetDelete()"
                  title="Delete"
                  @click="onDeletePreset(preset.id, $event)"
                >
                  ✕
                </button>
              </div>
              <p v-if="!namedPresets?.length" :class="ui.hint()" class="px-2 py-2">
                No saved presets
              </p>
            </div>
            <button type="button" :class="shell.button()" class="w-full" @click="emit('createPreset')">
              Create preset
            </button>
          </div>
        </div>
      </div>

      <div :class="ui.section()">
        <button type="button" :class="ui.sectionHeader()" @click="toggleSection('color')">
          <span>Color</span>
          <span class="text-neutral-500">{{ openSections.color ? "−" : "+" }}</span>
        </button>
        <div v-show="openSections.color" :class="ui.sectionBody()">
          <label :class="ui.field()" :title="PARAM_HINTS.colormode">
            <span :class="ui.fieldLabel()">Colormode</span>
            <select v-model="settings.colormode" :class="ui.control()">
              <option value="color">color</option>
              <option value="bw">bw</option>
            </select>
          </label>
          <label :class="ui.field()" :title="PARAM_HINTS.hierarchical">
            <span :class="ui.fieldLabel()">Hierarchical</span>
            <select v-model="settings.hierarchical" :class="ui.control()">
              <option value="stacked">stacked</option>
              <option value="cutout">cutout</option>
            </select>
          </label>
          <label :class="ui.field()" :title="PARAM_HINTS.filterSpeckle">
            <span :class="ui.fieldLabel()">
              filterSpeckle
              <span :class="ui.fieldValue()">{{ settings.filterSpeckle }}</span>
            </span>
            <input v-model.number="settings.filterSpeckle" :class="ui.range()" type="range" min="0" max="16" />
          </label>
          <label :class="ui.field()" :title="PARAM_HINTS.colorPrecision">
            <span :class="ui.fieldLabel()">
              colorPrecision
              <span :class="ui.fieldValue()">{{ settings.colorPrecision }}</span>
            </span>
            <input v-model.number="settings.colorPrecision" :class="ui.range()" type="range" min="1" max="8" />
          </label>
          <label :class="ui.field()" :title="PARAM_HINTS.gradientStep">
            <span :class="ui.fieldLabel()">
              gradientStep
              <span :class="ui.fieldValue()">{{ settings.gradientStep }}</span>
            </span>
            <input v-model.number="settings.gradientStep" :class="ui.range()" type="range" min="0" max="255" />
          </label>
        </div>
      </div>

      <div :class="ui.section()">
        <button type="button" :class="ui.sectionHeader()" @click="toggleSection('curves')">
          <span>Curves</span>
          <span class="text-neutral-500">{{ openSections.curves ? "−" : "+" }}</span>
        </button>
        <div v-show="openSections.curves" :class="ui.sectionBody()">
          <label :class="ui.field()" :title="PARAM_HINTS.mode">
            <span :class="ui.fieldLabel()">Mode</span>
            <select v-model="settings.mode" :class="ui.control()">
              <option value="pixel">pixel</option>
              <option value="polygon">polygon</option>
              <option value="spline">spline</option>
            </select>
          </label>
          <label :class="ui.field()" :title="PARAM_HINTS.cornerThreshold">
            <span :class="ui.fieldLabel()">
              cornerThreshold
              <span :class="ui.fieldValue()">{{ settings.cornerThreshold }}°</span>
            </span>
            <input v-model.number="settings.cornerThreshold" :class="ui.range()" type="range" min="0" max="180" />
          </label>
          <label :class="ui.field()" :title="PARAM_HINTS.segmentLength">
            <span :class="ui.fieldLabel()">
              segmentLength
              <span :class="ui.fieldValue()">{{ settings.segmentLength }}</span>
            </span>
            <input
              v-model.number="settings.segmentLength"
              :class="ui.range()"
              type="range"
              min="3.5"
              max="10"
              step="0.1"
            />
          </label>
          <label :class="ui.field()" :title="PARAM_HINTS.spliceThreshold">
            <span :class="ui.fieldLabel()">
              spliceThreshold
              <span :class="ui.fieldValue()">{{ settings.spliceThreshold }}°</span>
            </span>
            <input v-model.number="settings.spliceThreshold" :class="ui.range()" type="range" min="0" max="180" />
          </label>
          <label :class="ui.field()" :title="PARAM_HINTS.maxIterations">
            <span :class="ui.fieldLabel()">
              maxIterations
              <span :class="ui.fieldValue()">{{ settings.maxIterations }}</span>
            </span>
            <input v-model.number="settings.maxIterations" :class="ui.range()" type="range" min="1" max="32" />
          </label>
        </div>
      </div>

      <div :class="ui.section()">
        <button type="button" :class="ui.sectionHeader()" @click="toggleSection('output')">
          <span>Output</span>
          <span class="text-neutral-500">{{ openSections.output ? "−" : "+" }}</span>
        </button>
        <div v-show="openSections.output" :class="ui.sectionBody()">
          <label :class="ui.field()" :title="PARAM_HINTS.pathPrecision">
            <span :class="ui.fieldLabel()">
              pathPrecision
              <span :class="ui.fieldValue()">{{ settings.pathPrecision }}</span>
            </span>
            <input v-model.number="settings.pathPrecision" :class="ui.range()" type="range" min="0" max="12" />
          </label>
          <label :class="ui.field()" :title="PARAM_HINTS.trimEmptySpace">
            <span class="flex items-center gap-2">
              <input v-model="settings.trimEmptySpace" type="checkbox" class="accent-blue-500" />
              Trim empty transparent margins
            </span>
          </label>
          <label :class="ui.field()" :title="PARAM_HINTS.svgoOptimize">
            <span class="flex items-center gap-2">
              <input v-model="settings.svgoOptimize" type="checkbox" class="accent-blue-500" />
              SVGO on save
            </span>
          </label>
        </div>
      </div>
    </div>

    <div :class="ui.stickyFooter()">
      <div :class="ui.actions()">
        <button
          type="button"
          :class="shell.buttonPrimary()"
          :disabled="converting"
          @click="emit('convert')"
        >
          {{ converting ? "Converting…" : "Convert" }}
        </button>
        <button
          type="button"
          :class="shell.button()"
          :disabled="converting"
          @click="emit('saveAs')"
        >
          Save as…
        </button>
      </div>
    </div>
  </section>
</template>
