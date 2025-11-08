/**
 * @file This file defines the routes for the sidebar.
 */
import HourglassIcon from "virtual:icons/lucide/hourglass";
import ScrollTextIcon from "virtual:icons/lucide/scroll-text";
import SettingsIcon from "virtual:icons/lucide/settings";
import UploadIcon from "virtual:icons/lucide/upload-cloud";

// Object keys maintain insertion order in JavaScript
export const SIDEBAR_ROUTES = {
  "/main/history": { label: "History", icon: HourglassIcon },
  "/main/uploading": { label: "Uploading", icon: UploadIcon },
  "/main/settings": { label: "Settings", icon: SettingsIcon },
  "/main/changelog": { label: "Changelog (WIP)", icon: ScrollTextIcon },
};
