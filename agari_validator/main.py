import json
import gzip
import random
from pathlib import Path

def extract_sample_hands(file_path: str, n: int) -> list[dict]:
    winning_samples: list[dict] = []
    player_hands: list[list[str]] = [[], [], [], []]

    with gzip.open(file_path, "rt", encoding="utf-8") as f:
        for line in f:
            try:
                event = json.loads(line)
                e_type = event.get("type")
                actor = event.get("actor")

                if e_type == "start_kyoku":
                    player_hands = event.get("tehais", [[], [], [], []])
                elif e_type == "tsumo":
                    player_hands[actor].append(event.get("pai"))
                elif e_type == "dahai":
                    if event.get("pai") in player_hands[actor]:
                        player_hands[actor].remove(event.get("pai"))
                elif e_type in ["chi", "pon", "daikan", "minkan"]:
                    for p in event.get("consumed", []):
                        if p in player_hands[actor]:
                            player_hands[actor].remove(p)
                
                elif e_type == "hora":
                    # Deep dive into the yaku and score structure
                    # MJAI usually stores yaku as: "yaku": [["Riichi", 1], ["Pinfu", 1]]
                    yaku_list = event.get("yaku", [])
                    
                    # Extract Han and Fu - sometimes they are in 'han'/'fu' keys, 
                    # sometimes we sum the yaku list.
                    han = event.get("han", sum(y[1] for y in yaku_list if len(y) > 1))
                    fu = event.get("fu", 0)
                    
                    win_tile = event.get("hora_pai")
                    
                    # If hora_pai is missing, it's often the last tile in the tehai for Tsumo
                    # or we have to look back at the last 'dahai' for Ron.
                    
                    win_data = {
                        "tehai": list(player_hands[actor]),
                        "win_tile": win_tile,
                        "han": han,
                        "fu": fu,
                        "yaku": [y[0] for y in yaku_list],
                        "is_tsumo": event.get("actor") == event.get("target")
                    }
                    winning_samples.append(win_data)
                    
            except (json.JSONDecodeError, KeyError, IndexError):
                continue
    
    return random.sample(winning_samples, min(len(winning_samples), n))

def validate_agari(sample_hands: list[dict]):
    for hand in sample_hands:
        # Filter out empty or broken records
        if not hand["yaku"] and hand["han"] == 0:
            continue
            
        print(f"Hand: {' '.join(hand['tehai'])}")
        print(f"Win Tile: {hand['win_tile']} | {'Tsumo' if hand['is_tsumo'] else 'Ron'}")
        print(f"Expected: {hand['han']} Han, {hand['fu']} Fu")
        print(f"Yaku: {', '.join(hand['yaku'])}")
        
        # This is where you'd call your Rust binary
        # result = subprocess.run(["./agari", "--hand", ...])
        
        print("-" * 50)

if __name__ == "__main__":
    log_file = "/Users/rsullenb/Downloads/tenhou-to-mjai/2024/2024/2024010100gm-00a9-0000-0d9240dd.mjson"
    if Path(log_file).exists():
        samples = extract_sample_hands(log_file, 10)
        validate_agari(samples)
