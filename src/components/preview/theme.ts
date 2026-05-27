import { tv } from "@/lib/tv";

export const previewTv = tv({
  slots: {
    root: "flex min-h-0 flex-1 flex-col gap-2 p-4",
    toolbar: "flex flex-wrap items-center gap-2",
    segmented:
      "inline-flex rounded-lg border border-neutral-800 bg-neutral-900 p-0.5",
    segment:
      "cursor-pointer rounded-md px-2.5 py-1 text-xs transition-colors",
    viewportWrap: "relative flex min-h-0 flex-1 flex-col",
    viewport:
      "checkerboard relative min-h-0 flex-1 overflow-hidden rounded-xl border border-neutral-800 bg-neutral-900",
    panLayer: "relative h-full min-h-0 w-full",
    panInner: "flex h-full w-full items-center justify-center",
    empty:
      "flex h-full min-h-[320px] flex-col items-center justify-center gap-4 rounded-xl border border-dashed border-neutral-700 bg-neutral-900/60 p-8 text-center",
    emptyIcon: "text-4xl text-neutral-600",
    emptyTitle: "text-base font-medium text-neutral-200",
    emptyHint: "max-w-sm text-sm text-neutral-500",
    media: "max-h-full max-w-full object-contain",
    placeholder: "flex h-full items-center justify-center p-6 text-sm text-neutral-500",
    sliderRoot: "absolute inset-0 min-h-0 select-none touch-none",
    sliderLayer: "absolute inset-0 overflow-hidden",
    sliderImageWrap: "absolute inset-0 flex items-center justify-center",
    sliderBeforeClip: "absolute inset-0 z-[1] overflow-hidden",
    sliderMedia: "block h-full w-full object-contain",
    sliderHandle:
      "absolute top-0 z-10 h-full w-0.5 -translate-x-1/2 bg-blue-500 transition-colors",
    sliderKnob:
      "absolute top-1/2 left-1/2 size-7 -translate-x-1/2 -translate-y-1/2 rounded-full border-2 border-white bg-blue-500 shadow-lg transition-colors",
    sliderHint:
      "pointer-events-none absolute bottom-2 left-1/2 z-20 -translate-x-1/2 rounded bg-black/60 px-2 py-1 text-[10px] text-neutral-200",
    cropOverlay: "pointer-events-none absolute inset-0 z-30",
    cropStage: "absolute overflow-hidden rounded-sm border border-blue-500/30",
    cropBox:
      "pointer-events-none absolute border-2 border-blue-400 bg-blue-500/10 shadow-[0_0_0_9999px_rgba(0,0,0,0.62)]",
    cropBadge:
      "pointer-events-none absolute left-1 top-1 z-10 rounded bg-blue-600/90 px-1.5 py-0.5 text-[10px] font-medium text-white",
    cropHint:
      "pointer-events-none absolute bottom-1 left-1/2 z-10 max-w-[90%] -translate-x-1/2 rounded bg-black/70 px-2 py-1 text-center text-[10px] text-neutral-200",
    split: "grid h-full min-h-0 grid-cols-2 gap-2",
    pane: "relative flex h-full min-h-0 items-center justify-center overflow-hidden rounded-lg border border-neutral-800",
    paneLabel:
      "absolute left-2 top-2 z-10 rounded bg-black/60 px-2 py-0.5 text-[10px] font-medium uppercase tracking-wide text-white",
    overlaySvg: "pointer-events-none absolute inset-0 flex items-center justify-center",
    metrics:
      "flex flex-wrap gap-x-6 gap-y-1 rounded-lg border border-neutral-800 bg-neutral-900/80 px-3 py-2 text-xs",
    metric: "flex items-baseline gap-1.5",
    metricLabel: "text-neutral-500",
    metricValue: "font-medium text-neutral-200",
    actions: "flex flex-wrap gap-2",
    message: "text-xs text-neutral-400",
    loading: "absolute inset-0 z-20 flex items-center justify-center bg-neutral-950/60",
  },
  variants: {
    segmentActive: {
      true: { segment: "bg-neutral-800 text-white" },
      false: { segment: "text-neutral-400 hover:text-neutral-200" },
    },
    dragActive: {
      true: { empty: "border-blue-500/60 bg-blue-500/5" },
    },
    sliderLocked: {
      true: {
        sliderRoot: "cursor-pointer",
        sliderHandle: "cursor-pointer bg-amber-400",
        sliderKnob: "bg-amber-400",
      },
      false: {
        sliderRoot: "cursor-crosshair",
        sliderHandle: "cursor-ew-resize",
      },
    },
  },
});

export const batchBarTv = tv({
  slots: {
    root: "flex h-[200px] shrink-0 flex-col border-t border-neutral-800 bg-neutral-900/90",
    header: "flex shrink-0 items-center justify-between gap-2 border-b border-neutral-800 px-4 py-2",
    title: "text-sm font-medium",
    toolbar: "flex flex-wrap items-center gap-2",
    body: "min-h-0 flex-1 overflow-y-auto px-2 py-1",
    row: "flex cursor-pointer items-center justify-between gap-2 rounded-md px-2 py-1.5 text-xs hover:bg-neutral-800/80",
    rowActive: "bg-blue-500/10 ring-1 ring-blue-500/30",
    status: "shrink-0 rounded px-1.5 py-0.5 text-[10px] font-medium uppercase",
    footer: "flex shrink-0 items-center gap-3 border-t border-neutral-800 px-4 py-2",
    progressTrack: "h-1.5 flex-1 overflow-hidden rounded-full bg-neutral-800",
    progressFill: "h-full bg-blue-500 transition-all",
  },
});
