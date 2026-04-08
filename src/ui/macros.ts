/**
 * Requirement 14.1: Bun macro for JSX transform
 */

export function transformUI(element: any): string {
    // Transform JSX elements into alloy.ui.* calls
    // Simulation logic
    return `alloy.ui.createWindow(${JSON.stringify(element.props)})`;
}
