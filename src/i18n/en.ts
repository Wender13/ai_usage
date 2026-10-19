/** English texts; the reference shape that every other language must match. */
const en = {
  app: {
    lastCheck: "Last checked at {time}",
    waiting: "Waiting for the first check…",
    refresh: "Refresh",
    refreshing: "Refreshing…",
  },
  sections: {
    rules: "Rules",
    settings: "Settings",
    overlay: "Overlay",
  },
  providers: {
    claude: "Claude",
  },
  windows: {
    rolling_5h: "Session (5h)",
    rolling_7d: "Weekly (7 days)",
  },
  windowsShort: {
    rolling_5h: "5h",
    rolling_7d: "7d",
  },
  status: {
    loading: "Loading…",
    stale: "Values from the last successful check, at {time}.",
  },
  time: {
    lessThanMinute: "less than 1 min",
    resetUnknown: "reset time unknown",
    resetting: "resetting…",
    resetsIn: "resets in {duration} ({when})",
  },
  rules: {
    new: "New rule",
    empty: "No rules yet.",
    enable: "Enable rule",
    disable: "Disable rule",
    edit: "Edit",
    delete: "Delete",
    anyProvider: "any provider",
    anyWindow: "any window",
    types: {
      threshold: "Usage threshold",
      interval: "Periodic summary",
      beforeReset: "Before reset",
    },
    channels: {
      notification: "Notification",
      overlay: "Overlay",
      both: "Notification and overlay",
    },
    describe: {
      threshold: "When {target} reaches {percent}%",
      interval: "Usage summary every {minutes} min",
      beforeReset: "{minutes} min before {target} resets",
    },
    form: {
      type: "Type",
      provider: "Provider",
      window: "Window",
      any: "Any",
      threshold: "Threshold (%)",
      every: "Every (min)",
      minutesBefore: "Minutes before reset",
      channel: "Channel",
      hint: "Checked on every usage check, so it is as precise as the check interval.",
      cancel: "Cancel",
      save: "Save",
      create: "Create rule",
    },
  },
  settings: {
    checkEvery: "Check usage every",
    minutes: "min",
    save: "Save",
    saved: "Saved.",
    language: "Language",
    languageAuto: "Automatic (system)",
  },
  overlay: {
    show: "Show overlay",
    position: "Position",
    custom: "Custom (dragged)",
    opacity: "Background opacity: {value}%",
    lock: "Lock position (prevents dragging the overlay)",
    fieldsLegend: "Information shown",
    fields: {
      showProvider: "Provider name",
      showLabel: "Window name (5h, 7d)",
      showPercent: "Percentage",
      showReset: "Time until reset",
    },
    windowsLegend: "Windows shown",
    xwayland: "Use XWayland",
    xwaylandHint:
      'On GNOME with Wayland, position, lock and "always on top" only work through XWayland. Takes effect after restarting the app.',
    dismiss: "Click to dismiss",
    moreAlerts: "+{count} more alert(s)",
    lastCheckFailed: "The last check failed",
  },
  anchors: {
    top_left: "Top left",
    top_center: "Top center",
    top_right: "Top right",
    bottom_left: "Bottom left",
    bottom_center: "Bottom center",
    bottom_right: "Bottom right",
  },
};

export type Messages = typeof en;
export default en;
