/**
 * @file This file defines the routes for the sidebar.
 */
import HourglassIcon from "virtual:icons/lucide/hourglass";
import SettingsIcon from "virtual:icons/lucide/settings";
import UploadIcon from "virtual:icons/lucide/upload-cloud";
import PaletteIcon from "virtual:icons/lucide/palette";

// Object keys maintain insertion order in JavaScript
export const SIDEBAR_ROUTES = {
  "/main/history": { label: "History", icon: HourglassIcon },
  "/main/uploading": { label: "Uploading", icon: UploadIcon },
  "/main/themes": { label: "Themes", icon: PaletteIcon },
  "/main/settings": { label: "Settings", icon: SettingsIcon },
};
