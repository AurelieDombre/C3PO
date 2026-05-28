from pathlib import Path

# =========================================================
# #. CALCUL DU SCORE (pour les correspondances mots clés -> dossiers et fichiers)
# =========================================================
def _is_dir_safe(item: Path) -> bool:
    try:
        return item.is_dir()
    except OSError:
        return False


def compute_score(item: Path, keywords: list[str]) -> int:
    # Nom du fichier en minuscule
    # pour comparaison insensible à la casse
    try:
      name_lower = item.name.lower()
      full_path_lower = str(item).lower()
    except OSError:
      return 0
    score = 0

    for kw in keywords:
        # Ignore mots trop courts
        if len(kw) <= 2:
            continue
        # Correspondance exacte sur le nom → priorité absolue
        if name_lower == kw:
            score += 100
        # Nom commence par le mot-clé
        if name_lower.startswith(kw):
            score += 40
        # Mot-clé dans le nom
        if kw in name_lower:
            score += 20
        # Mot-clé est un dossier parent dans le chemin
        if f"\\{kw}\\" in full_path_lower or full_path_lower.endswith(f"\\{kw}"):
            score += 5
    # Les dossiers remontent avant les fichiers à score égal
    if _is_dir_safe(item) and score > 0:
        score += 10

    return score
