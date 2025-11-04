/**
 * @file This file contains a hook for detecting whether the application is running on a mobile device.
 */
import { MediaQuery } from "svelte/reactivity";

const DEFAULT_MOBILE_BREAKPOINT = 768;

/**
 * A class that extends `MediaQuery` to detect whether the application is running on a mobile device.
 */
export class IsMobile extends MediaQuery {
  /**
   * Creates a new `IsMobile` instance.
   *
   * @param breakpoint - The breakpoint to use for detecting a mobile device.
   */
  constructor(breakpoint: number = DEFAULT_MOBILE_BREAKPOINT) {
    super(`max-width: ${breakpoint - 1}px`);
  }
}
