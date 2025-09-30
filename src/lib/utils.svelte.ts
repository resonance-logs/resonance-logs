import tippy from 'tippy.js';
import 'tippy.js/dist/tippy.css'; // optional for styling
import type { Attachment } from 'svelte/attachments';
import html2canvas from "html2canvas-pro";
import { writeText, writeImage } from '@tauri-apps/plugin-clipboard-manager';
import { image } from '@tauri-apps/api';

export const classColors: Record<string, string> = {
  "Stormblade": "#674598",
  "Frost Mage": "#4de3d1",
  "Wind Knight": "#0099c6",
  "Verdant Oracle": "#66aa00",
  "Heavy Guardian": "#b38915",
  "Marksman": "#ffee00",
  "Shield Knight": "#7b9aa2",
  "Beat Performer": "#ee2e48",
};

export function getClassColor(className: string): string {
  return `rgb(from ${classColors[className] ?? "#ffc9ed"} r g b / 0.6)`;
}

export function getClassIcon(class_name: string): string {
  if (class_name == "") {
    return "/images/classes/blank.png";
  }
  return "/images/classes/" + class_name + ".png";
}

// https://svelte.dev/docs/svelte/@attach#Attachment-factories
export function tooltip(getContent: () => string): Attachment {
  return (element: Element) => {
    const tooltip = tippy(element, { 
      content: "",
    });
    $effect(() => {
      tooltip.setContent(getContent())
    })
    return tooltip.destroy;
  };
}

export async function copyToClipboard(error: MouseEvent & { currentTarget: EventTarget & HTMLTableCellElement }, content: string) {
  // TODO: add a way to simulate a "click" animation
  error.stopPropagation();
  await writeText(content);
}

export function takeScreenshot(target?: HTMLElement) {
  if (!target) return;
  setTimeout(async () => {
    const canvas = await html2canvas(target, { backgroundColor: "#27272A" });
    canvas.toBlob(async (blob) => {
      if (!blob) return;
      try {
        // TODO: is there a way to avoid image.Image.fromBytes()?
        await writeImage(await image.Image.fromBytes(await blob.arrayBuffer()));
      } catch (error) {
        console.error("Failed to take a screenshot", error)
      }
    });
  }, 100);
} 