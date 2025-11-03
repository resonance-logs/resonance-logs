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

    # Create the mapping: SceneID -> Name
    scene_name_mapping = {}

    for dungeon_id, dungeon_info in dungeons_data.items():
        scene_id = str(dungeon_info.get("SceneID", ""))
        name = dungeon_info.get("Name", "")

        if scene_id and name:
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