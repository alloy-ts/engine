import type { MacroContext } from "bun";

export function transformUI(ctx: MacroContext, element: any): string {
  // Simple placeholder for JSX transform logic
  if (element.type === "Window") {
    return `alloy.ui.createWindow(${JSON.stringify(element.props)}, capId)`;
  }
  return "";
}
