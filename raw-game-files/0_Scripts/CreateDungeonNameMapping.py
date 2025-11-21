import json
import os
import re

def create_dungeon_name_mapping():
    """
    Extracts SceneID and Name from DungeonsTable.json and creates a mapping
    similar to MonsterNameBoss.json structure
    """
    # Use absolute paths relative to this script
    script_dir = os.path.dirname(os.path.abspath(__file__))
    input_path = os.path.join(script_dir, "../1_Dirty/DungeonsTable.json")
    output_path = os.path.join(script_dir, "../../src-tauri/meter-data/SceneName.json")

    # Load the DungeonsTable.json
    with open(input_path, "r", encoding="utf-8") as f:
        dungeons_data = json.load(f)

    # Create the mapping: SceneID -> Name
    scene_name_mapping = {}

    for dungeon_id, dungeon_info in dungeons_data.items():
        scene_id = str(dungeon_info.get("SceneID", ""))
        name = dungeon_info.get("Name", "")

        if name.lower() == "content mechanism test scene":
            name = "Overworld"

        dungeon_type = dungeon_info.get("DungeonTypeName", "")

        if scene_id and name:
            # Append DungeonTypeName if it exists and is a short string (likely a difficulty)
            if dungeon_type and len(dungeon_type) < 50 and "<br>" not in dungeon_type:
                # Normalize 'Master 1', 'Master 2' -> 'Master'
                m = re.match(r"^(Master)\s*\d+$", dungeon_type.strip(), re.IGNORECASE)
                if m:
                    normalized = m.group(1).title()
                else:
                    normalized = dungeon_type

                name = f"{name} - {normalized}"

            scene_name_mapping[scene_id] = name

    # Ensure output directory exists
    os.makedirs(os.path.dirname(output_path), exist_ok=True)

    # Write the mapping to the output file
    with open(output_path, "w", encoding="utf-8") as f:
        json.dump(scene_name_mapping, f, ensure_ascii=False, indent=4)

    print(f"✅ Created SceneName.json with {len(scene_name_mapping)} mappings")
    print(f"Output saved to: {output_path}")

    return scene_name_mapping

if __name__ == "__main__":
    create_dungeon_name_mapping()