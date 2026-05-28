from pathlib import Path

def _is_dir_safe(item: Path) -> bool:
    try:
        return item.is_dir()
    except OSError:
        return False


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
        "type": "folder" if _is_dir_safe(item) else "file",
        "score": score
    }
