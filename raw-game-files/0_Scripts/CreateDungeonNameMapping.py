import json
import os

def create_dungeon_name_mapping():
    """
    Extracts SceneID and Name from DungeonsTable.json and creates a mapping
    similar to MonsterNameBoss.json structure
    """
    input_path = "../1_Dirty/DungeonsTable.json"
    output_path = "../../src-tauri/meter-data/SceneName.json"

    # Load the DungeonsTable.json
    with open(input_path, "r", encoding="utf-8") as f:
        dungeons_data = json.load(f)

    # Create the mapping: SceneID -> { Name, DungeonTypeName }
    # We'll include both the human-readable Name and the DungeonTypeName value
    # if present. This makes the mapping more useful for consumers that need
    # both display names and type identifiers.
    scene_name_mapping = {}

    for dungeon_id, dungeon_info in dungeons_data.items():
        scene_id = str(dungeon_info.get("SceneID", ""))
        name = dungeon_info.get("Name", "")
        dungeon_type_name = dungeon_info.get("DungeonTypeName", "")

        # Only keep DungeonTypeName if it contains one of the keywords we care
        # about: 'normal', 'hard', or 'master' (case-insensitive). Otherwise
        # store an empty string so consumers can easily detect absence.
        dt = (dungeon_type_name or "").strip()
        dt_lower = dt.lower()
        if any(k in dt_lower for k in ("normal", "hard", "master")):
            filtered_dt = dungeon_type_name
        else:
            filtered_dt = ""

        # Include an entry for any scene_id present. DungeonTypeName will be
        # the filtered value (either the original string if it matched one of
        # the keywords, or empty string otherwise).
        if scene_id:
            scene_name_mapping[scene_id] = {
                "Name": name,
                "DungeonTypeName": filtered_dt
            }

    # Ensure output directory exists
    os.makedirs(os.path.dirname(output_path), exist_ok=True)

    # Write the mapping to the output file
    with open(output_path, "w", encoding="utf-8") as f:
        json.dump(scene_name_mapping, f, ensure_ascii=False, indent=4)

    print(f"✅ Created SceneName.json with {len(scene_name_mapping)} mappings (includes Name and DungeonTypeName)")
    print(f"Output saved to: {output_path}")

    return scene_name_mapping

if __name__ == "__main__":
    create_dungeon_name_mapping()