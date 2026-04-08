// src/gui/gui_shims.ts
// Installed by gui_plugin.c on_init via eval_fn
// Thin wrappers over __alloy_ui_* C bridge globals

declare const __alloy_ui_request_cap: (resource: string, perms: string, scopes: string) => string;
declare const __alloy_ui_create_window: (opts: string, capId: string) => string;
declare const __alloy_ui_close_window: (windowId: number, capId: string) => string;
declare const __alloy_ui_add_control: (windowId: number, spec: string, capId: string) => string;
declare const __alloy_ui_set_layout: (controlId: number, layout: string) => string;
declare const __alloy_ui_on_event: (handler: Function, windowId: number) => string;
declare const __alloy_ui_set_menu_bar: (template: string, capId: string) => string;
declare const __alloy_ui_set_tray_icon: (iconPath: string, tooltip: string, capId: string) => string;
declare const Alloy: any;

(function(global: any) {
  const ui = {
    requestCapability: (resource: string, perms: string[], scopes?: string[]) =>
      __alloy_ui_request_cap(resource, JSON.stringify(perms), JSON.stringify(scopes ?? [])),

    createWindow: (opts: object, capId: string) =>
      JSON.parse(__alloy_ui_create_window(JSON.stringify(opts), capId)),

    closeWindow: (windowId: number, capId: string) =>
      JSON.parse(__alloy_ui_close_window(windowId, capId)),

    addControl: (windowId: number, spec: object, capId: string) =>
      JSON.parse(__alloy_ui_add_control(windowId, JSON.stringify(spec), capId)),

    setLayout: (controlId: number, layout: object) =>
      JSON.parse(__alloy_ui_set_layout(controlId, JSON.stringify(layout))),

    onEvent: (handler: Function, windowId?: number) =>
      __alloy_ui_on_event(handler, windowId ?? -1),

    setMenuBar: (template: object[], capId: string) =>
      JSON.parse(__alloy_ui_set_menu_bar(JSON.stringify(template), capId)),

    setTrayIcon: (iconPath: string, tooltip: string, capId: string) =>
      JSON.parse(__alloy_ui_set_tray_icon(iconPath, tooltip, capId)),
  };

  global.alloy = global.alloy || {};
  global.alloy.ui = ui;

  if (typeof Alloy !== "undefined" && Alloy.plugins) {
    Alloy.plugins["alloy:gui"] = { name: "alloy:gui", version: "0.1.0", ui };
  }
})(globalThis);
