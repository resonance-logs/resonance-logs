/**
 * @file This file contains utility functions and constants for the application.
 */
import tippy from 'tippy.js';
import 'tippy.js/dist/tippy.css'; // optional for styling
import type { Attachment } from 'svelte/attachments';
// import html2canvas from "html2canvas-pro";
import { writeText } from '@tauri-apps/plugin-clipboard-manager';
// import { writeImage } from '@tauri-apps/plugin-clipboard-manager';
// import { image } from '@tauri-apps/api';
import { WebviewWindow } from '@tauri-apps/api/webviewWindow';

import { SETTINGS, DEFAULT_CLASS_COLORS } from '$lib/settings-store';

export const CLASS_MAP: Record<number, string> = {
  1: 'Stormblade',
  2: 'Frost Mage',
  4: 'Wind Knight',
  5: 'Verdant Oracle',
  9: 'Heavy Guardian',
  11: 'Marksman',
  12: 'Shield Knight',
  13: 'Beat Performer'
};

export const CLASS_NAMES = Object.values(CLASS_MAP);

export function getClassColor(className: string): string {
  const customColors = SETTINGS.accessibility.state.classColors ?? DEFAULT_CLASS_COLORS;
  const color = customColors[className] ?? DEFAULT_CLASS_COLORS[className] ?? "#ffc9ed";
  return `rgb(from ${color} r g b / 0.6)`;
}

export function getClassColorRaw(className: string): string {
  const customColors = SETTINGS.accessibility.state.classColors ?? DEFAULT_CLASS_COLORS;
  return customColors[className] ?? DEFAULT_CLASS_COLORS[className] ?? "#ffc9ed";
}

export function getClassIcon(class_name: string): string {
  if (class_name === "") {
    return "/images/classes/blank.png";
  }
  return "/images/classes/" + class_name + ".png";
}

// https://svelte.dev/docs/svelte/@attach#Attachment-factories
export function tooltip(getContent: () => string): Attachment {
  return (element: Element) => {
    const instance = tippy(element, {
      content: getContent(),
      theme: 'resonance',
      arrow: true,
      delay: [200, 80],
      duration: [120, 80],
      animation: 'fade',
      moveTransition: 'transform 120ms ease-out',
      placement: 'top',
    });

    // Keep content in sync with reactive source
    $effect(() => {
      instance.setContent(getContent());
    });

    return () => instance.destroy();
  };
}

export async function copyToClipboard(error: MouseEvent & { currentTarget: EventTarget & HTMLElement }, content: string) {
  // TODO: add a way to simulate a "click" animation
  error.stopPropagation();
  await writeText(content);
}

// export async function takeScreenshot(target?: HTMLElement): Promise<void> {
//   if (!target) return;
//   // Give the browser a paint frame (helps if caller just changed DOM)
//   await new Promise(requestAnimationFrame);

//   const canvas = await html2canvas(target, { backgroundColor: "#27272A" });

//   const blob: Blob | null = await new Promise((resolve) =>
//     canvas.toBlob(resolve)
//   );
//   if (!blob) return;

//   try {
//     await writeImage(await image.Image.fromBytes(await blob.arrayBuffer()));
//   } catch (error) {
//     console.error("Failed to take a screenshot", error);
//   }
// }

let isClickthrough = false;

export function getClickthroughState(): boolean {
  return isClickthrough;
}

export async function setClickthrough(bool: boolean) {
  const liveWindow = await WebviewWindow.getByLabel("live");
  await liveWindow?.setIgnoreCursorEvents(bool);
  isClickthrough = bool;
}

export async function toggleClickthrough() {
  const liveWindow = await WebviewWindow.getByLabel("live");
  await liveWindow?.setIgnoreCursorEvents(!isClickthrough);
  isClickthrough = !isClickthrough;
}
