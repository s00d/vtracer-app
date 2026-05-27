import { tv } from "@/lib/tv";

export const shellTv = tv({
  slots: {
    root: "flex h-screen flex-col bg-neutral-950 text-neutral-100",
    header:
      "flex shrink-0 items-center justify-between gap-4 border-b border-neutral-800 px-5 py-3",
    headerLeft: "min-w-0",
    title: "text-lg font-semibold tracking-tight",
    subtitle: "text-xs text-neutral-400",
    headerActions: "flex shrink-0 items-center gap-2",
    modeTabs: "flex rounded-lg border border-neutral-800 bg-neutral-900/80 p-0.5",
    modeTab:
      "cursor-pointer rounded-md px-3 py-1.5 text-sm transition-colors",
    body: "flex min-h-0 flex-1 flex-col overflow-hidden",
    workbench: "flex min-h-0 flex-1 flex-col overflow-hidden",
    split: "flex min-h-0 flex-1 overflow-hidden",
    previewCol: "flex min-w-0 flex-[65] flex-col overflow-hidden",
    settingsCol:
      "flex w-[min(420px,35%)] min-w-[300px] shrink-0 flex-col border-l border-neutral-800 bg-neutral-900/50",
    button:
      "cursor-pointer rounded-lg border border-neutral-700 bg-neutral-800 px-3 py-1.5 text-sm text-neutral-100 transition-colors hover:bg-neutral-700 disabled:cursor-not-allowed disabled:opacity-40",
    buttonPrimary:
      "cursor-pointer rounded-lg border border-blue-600 bg-blue-600 px-3 py-1.5 text-sm font-medium text-white transition-colors hover:bg-blue-500 disabled:cursor-not-allowed disabled:opacity-40",
    buttonGhost:
      "cursor-pointer rounded-lg px-3 py-1.5 text-sm text-neutral-300 transition-colors hover:bg-neutral-800 hover:text-white",
  },
  variants: {
    modeTabActive: {
      true: { modeTab: "bg-neutral-800 font-medium text-white shadow-sm" },
      false: { modeTab: "text-neutral-400 hover:text-neutral-200" },
    },
    dragActive: {
      true: { previewCol: "ring-2 ring-inset ring-blue-500/50" },
    },
  },
});

export const panelTv = tv({
  slots: {
    panel: "flex min-h-0 flex-1 flex-col",
    scroll: "min-h-0 flex-1 overflow-y-auto px-4 py-3",
    panelTitle: "text-sm font-medium text-neutral-200",
    section: "border-b border-neutral-800 last:border-0",
    sectionHeader:
      "flex w-full cursor-pointer items-center justify-between gap-2 py-3 text-left text-sm font-medium text-neutral-200",
    sectionBody: "space-y-3 pb-4",
    field: "flex flex-col gap-1.5 text-xs text-neutral-400",
    fieldLabel: "flex items-center justify-between gap-2",
    fieldValue: "tabular-nums text-neutral-200",
    control:
      "w-full rounded-lg border border-neutral-700 bg-neutral-950 px-2.5 py-1.5 text-sm text-neutral-100 focus:border-blue-500 focus:outline-none",
    range: "h-1.5 w-full cursor-pointer accent-blue-500",
    grid: "grid grid-cols-1 gap-3",
    hint: "text-xs text-neutral-500",
    stickyFooter:
      "shrink-0 border-t border-neutral-800 bg-neutral-900/95 p-4 backdrop-blur-sm",
    actions: "flex flex-wrap gap-2",
    pathLine: "break-all text-xs text-neutral-400",
    presetList:
      "max-h-36 space-y-1 overflow-y-auto rounded-lg border border-neutral-800 bg-neutral-950/60 p-1",
    presetItem:
      "flex w-full cursor-pointer items-center gap-2 rounded-md px-2 py-1.5 text-left text-sm text-neutral-200 transition-colors hover:bg-neutral-800",
    presetItemName: "min-w-0 flex-1 truncate",
    presetDelete:
      "shrink-0 cursor-pointer rounded px-1.5 py-0.5 text-xs text-neutral-500 transition-colors hover:bg-neutral-700 hover:text-red-400",
  },
});

export const drawerTv = tv({
  slots: {
    backdrop: "fixed inset-0 z-40 bg-black/50 backdrop-blur-sm",
    panel:
      "fixed right-0 top-0 z-50 flex h-full w-full max-w-md flex-col border-l border-neutral-800 bg-neutral-900 shadow-2xl",
    header:
      "flex shrink-0 items-center justify-between border-b border-neutral-800 px-4 py-3",
    title: "text-base font-semibold",
    body: "min-h-0 flex-1 overflow-y-auto p-4",
  },
});
