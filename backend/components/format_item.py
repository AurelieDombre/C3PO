from pathlib import Path

def rglob_safe(path):
    try:
        for item in path.iterdir():
            yield item
            if item.is_dir():
                yield from rglob_safe(item)
    except PermissionError:
        pass
    except OSError:
        pass
    

# =========================================================
# #. FORMAT D'UN ITEM 
# =========================================================
def format_item(item: Path, score: int) -> dict:
    
    return {
        "name": item.name,
        "path": str(item),
        "type": "folder" if item.is_dir() else "file",
        "score": score
    }
