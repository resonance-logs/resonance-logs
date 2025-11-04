/**
 * Utility function for transforming player name display in the DPS meter
 * based on user preferences for showing/hiding names and classes.
 */

/**
 * Player object containing basic player information
 */
export interface Player {
  /** Unique identifier for the player */
  uid: number;
  /** Player's name */
  name: string;
  /** Player's class name */
  className: string;
}

/**
 * Settings for name display preferences
 */
export type NameDisplaySetting = 
  | "Show Your Name" 
  | "Show Your Class" 
  | "Hide Your Name"
  | "Show Others' Name" 
  | "Show Others' Class" 
  | "Hide Others' Name";

/**
 * Parameters for the name display transformation function
 */
export interface NameDisplayParams {
  /** Player object with name, className, and uid properties */
  player: Player;
  /** Setting for how to display the local player's name */
  showYourNameSetting: string;
  /** Setting for how to display other players' names */
  showOthersNameSetting: string;
  /** Boolean indicating if this is the current user */
  isLocalPlayer: boolean;
}

/**
 * Transforms player name display based on user preferences
 * 
 * @param params - Object containing player information and display settings
 * @returns The appropriate display name based on the settings
 * 
 * @example
 * ```typescript
 * const displayName = getDisplayName({
 *   player: { uid: 123, name: "PlayerName", className: "Frost Mage" },
 *   showYourNameSetting: "Show Your Class",
 *   showOthersNameSetting: "Show Others' Name",
 *   isLocalPlayer: true
 * });
 * // Returns "Frost Mage"
 * ```
 */
export default function getDisplayName(params: NameDisplayParams): string {
  const { player, showYourNameSetting, showOthersNameSetting, isLocalPlayer } = params;
  
  // Determine which setting to use based on whether this is the local player
  const setting = isLocalPlayer ? showYourNameSetting : showOthersNameSetting;
  
  // Apply the appropriate setting
  switch (setting) {
    case "Show Your Name":
    case "Show Others' Name":
      return player.name || player.uid.toString();
    
    case "Show Your Class":
    case "Show Others' Class":
      return player.className || player.name || player.uid.toString();
    
    case "Hide Your Name":
    case "Hide Others' Name":
      return player.name ? "" : player.uid.toString();
    
    default:
      // Fallback to name if setting is unrecognized
      return player.name || player.uid.toString();
  }
}