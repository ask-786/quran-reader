/** Below this width the sidebar becomes an overlay drawer instead of a docked column. */
export const MOBILE_BREAKPOINT = 900;

export function isNarrowViewport(): boolean {
  return typeof window !== 'undefined' && window.innerWidth <= MOBILE_BREAKPOINT;
}
