import tippy from 'tippy.js';
import 'tippy.js/dist/tippy.css'; // optional for styling
import type { Attachment } from 'svelte/attachments';

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