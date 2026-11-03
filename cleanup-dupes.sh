#!/usr/bin/env bash
set -euo pipefail

# Remove lower-quality duplicate movies and fix mismatched files.
# Jellyfin mount paths: /media/ = /data1/media/, /media2/ = /data2/media/
#
# Usage:
#   ./cleanup-dupes.sh              # dry run
#   ./cleanup-dupes.sh --apply      # actually execute

APPLY=false
if [[ "${1:-}" == "--apply" ]]; then
    APPLY=true
fi

echo "=== Movie Library Cleanup ==="
if $APPLY; then
    echo "*** APPLY MODE ***"
else
    echo "(dry run — pass --apply to execute)"
fi
echo ""

run() {
    echo "  $*"
    if $APPLY; then
        "$@"
    fi
}

# ============================================================
# PART 1: DUPLICATE REMOVAL (keep better quality)
# ============================================================
echo "========== DUPLICATES =========="
echo ""

# --- Akira: keep data2 multi-audio, remove data1 720p English-only ---
echo "--- Akira (1988) ---"
echo "  KEEP: /data2 DVDRip multi-audio (JPN + 2xENG + FRA + subs)"
echo "  REMOVE: /data1 720p BluRay English-only"
run rm -rf "/data1/media/Movies/Akira (1988) [BluRay] [720p] [YTS.AM]"
echo ""

# --- Harry Potter: keep data2 1080p HEVC 5.1, remove data1 720p stereo ---
HP="/data1/media/Movies/Harry.Potter"
echo "--- Harry Potter (all 8 films) ---"
echo "  KEEP: /data2 1080p HEVC 5.1 surround collection"
echo "  REMOVE: /data1 720p stereo copies"
for num in 1 2 3 4 5 6 7 8; do
    dir=$(find "$HP" -maxdepth 1 -type d -name "${num}.*" 2>/dev/null | head -1)
    if [[ -n "$dir" ]]; then
        run rm -rf "$dir"
    fi
done
# Remove extras (images, info, samples) and parent dir
run rm -rf "$HP/Harry-Potter-Complete-8-Film-Collection-.jpg" 2>/dev/null || true
run rm -rf "$HP/Info.txt" 2>/dev/null || true
run rm -rf "$HP/Samples & Screenshots" 2>/dev/null || true
run rmdir "$HP" 2>/dev/null || true
echo ""

# --- Pokemon: keep multi-audio, remove English-only ---
echo "--- Pokémon: The First Movie (1998) ---"
echo "  KEEP: /data1 Pokemon The First Movie - Mewtwo Strikes Back/ (ENG + JPN audio, 3 sub tracks)"
echo "  REMOVE: /data1 Pokémon the First Movie (1998)/ (English-only)"
run rm -rf "/data1/media/Movies/Pokémon the First Movie (1998)"
echo ""

# --- Narnia: identical file duplicated in same folder ---
echo "--- The Chronicles of Narnia (2005) ---"
echo "  KEEP: clean-named copy"
echo "  REMOVE: YIFY duplicate (identical file)"
NARNIA="/data1/media/Movies/The Chronicles of Narnia - The Lion, The Witch, and The Wardrobe (2005)"
run rm -f "$NARNIA/The.Chronicles.of.Narnia.The.Lion.The Witch.And.The.Wardrobe.2005.720p.Brrip.x264.Deceit.YIFY.mp4"
run rm -f "$NARNIA/The.Chronicles.of.Narnia.The.Lion.The Witch.And.The.Wardrobe.2005.720p.Brrip.x264.Deceit.YIFY.srt"
run rm -f "$NARNIA/The.Chronicles.of.Narnia.The.Lion.The Witch.And.The.Wardrobe.2005.720p.Brrip.x264.Deceit.YIFY.nfo"
run rm -f "$NARNIA/The.Chronicles.of.Narnia.The.Lion.The Witch.And.The.Wardrobe.2005.720p.Brrip.x264.Deceit.YIFY-logo.png"
run rm -f "$NARNIA/The.Chronicles.of.Narnia.The.Lion.The Witch.And.The.Wardrobe.2005.720p.Brrip.x264.Deceit.YIFY-poster.jpg"
run rm -f "$NARNIA/The.Chronicles.of.Narnia.The.Lion.The Witch.And.The.Wardrobe.2005.720p.Brrip.x264.Deceit.YIFY-backdrop.jpg"
run rm -f "$NARNIA/The.Chronicles.of.Narnia.The.Lion.The Witch.And.The.Wardrobe.2005.720p.Brrip.x264.Deceit.YIFY-landscape.jpg"
echo ""

# ============================================================
# PART 2: MISLABELED FILES (content matches folder, not filename)
# Verified via ffprobe: runtimes match the folder name, not the filename.
# ============================================================
echo "========== MISLABELED FILES (rename file to match actual content) =========="
echo ""

# --- The Dark Knight: file IS The Dark Knight (152min matches), just named wrong ---
DK="/data1/media/Movies/The Dark Knight (2008)"
echo "--- The Dark Knight (2008) ---"
echo "  152min runtime = The Dark Knight. (Batman DKR Part 1 is only 76min)"
echo "  Renaming file to match folder"
run mv "$DK/Batman the Dark Knight Returns, Part 1 (2012).mp4" \
       "$DK/The Dark Knight (2008).mp4"
# Also fix the .srt companion
run mv "$DK/Batman.The.Dark.Knight.srt" \
       "$DK/The Dark Knight (2008).srt" 2>/dev/null || true
echo ""

# --- Hitler: file IS The Last Ten Days (106min ≈ 108min), not "My Part" (102min comedy) ---
HITLER="/data1/media/Movies/Hitler - The Last Ten Days [1973 - UK] WWII drama"
echo "--- Hitler: The Last Ten Days (1973) ---"
echo "  106min runtime + drama subs = The Last Ten Days. (My Part in His Downfall is a 102min comedy)"
echo "  Renaming file to match folder"
run mv "$HITLER/Adolf Hitler - My Part in His Downfall (1973).mkv" \
       "$HITLER/Hitler - The Last Ten Days (1973).mkv"
# Fix companion srt files that reference the correct title (leave as-is, they're already correct)
echo ""

# --- WALL-E: file IS WALL-E (98min matches), not L'eclisse (126min) ---
WALLE="/data1/media/Movies/Wall_e_2008"
echo "--- WALL-E (2008) ---"
echo "  98min runtime = WALL-E. (L'eclisse is 126min)"
echo "  Renaming file to match folder"
run mv "$WALLE/L'eclisse (1962).mp4" \
       "$WALLE/WALL-E (2008).mp4"
echo ""

# ============================================================
echo "========== DONE =========="
if ! $APPLY; then
    echo ""
    echo "Dry run complete. Run with --apply to execute."
    echo "After cleanup, trigger a Jellyfin library rescan."
fi
