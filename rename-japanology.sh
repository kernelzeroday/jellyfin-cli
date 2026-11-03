#!/usr/bin/env bash
set -euo pipefail

# Rename Japanology files into proper Jellyfin directory structure.
# TVDB/TMDB list these as 3 separate shows:
#   - Begin Japanology (2008)
#   - Japanology Plus (2014)
#   - Begin Japanology mini (2009)
#
# Usage:
#   ./rename-japanology.sh              # dry run (default)
#   ./rename-japanology.sh --apply      # actually rename

SRC="/data2/media/TV/japanology/Begin Japanology/Begin Japanology"
TV="/data2/media/TV"

APPLY=false
if [[ "${1:-}" == "--apply" ]]; then
    APPLY=true
fi

# Counters for sequential episode numbering
ep_begin=0
ep_plus=0
ep_mini=0
ep_special=0

do_move() {
    local src="$1"
    local dst="$2"
    local dstdir
    dstdir="$(dirname "$dst")"

    if $APPLY; then
        mkdir -p "$dstdir"
        mv -v "$src" "$dst"
    else
        echo "  $src"
        echo "    -> $dst"
        echo ""
    fi
}

rename_pair() {
    local media_src="$1"
    local media_dst="$2"

    do_move "$media_src" "$media_dst"

    # Also move companion .nfo if it exists
    local base_src="${media_src%.*}"
    local base_dst="${media_dst%.*}"
    if [[ -f "${base_src}.nfo" ]]; then
        do_move "${base_src}.nfo" "${base_dst}.nfo"
    fi
}

echo "=== Japanology Rename Script ==="
if $APPLY; then
    echo "*** APPLY MODE — files will be moved ***"
else
    echo "(dry run — pass --apply to execute)"
fi
echo ""

# --- BEGIN Japanology (numbered files: NNN BEGIN Japanology - Title.ext) ---
echo "--- Begin Japanology (2008) ---"
show_dir="$TV/Begin Japanology (2008)/Season 01"

while IFS= read -r -d '' filepath; do
    filename="$(basename "$filepath")"
    ext="${filename##*.}"

    # Extract title: strip "NNN BEGIN Japanology - " prefix
    title="${filename#*BEGIN Japanology - }"
    title="${title%.*}"  # strip extension

    ep_begin=$((ep_begin + 1))
    ep_num=$(printf "%03d" "$ep_begin")

    newname="Begin Japanology - S01E${ep_num} - ${title}.${ext}"
    rename_pair "$filepath" "$show_dir/$newname"
done < <(find "$SRC" -maxdepth 1 -type f \( -name "*.mp4" -o -name "*.avi" -o -name "*.mkv" \) -regex '.*/[0-9]+ BEGIN Japanology - .*' -print0 | sort -z)

echo "  Total: $ep_begin episodes"
echo ""

# --- Japanology Plus (numbered files: NNN Japanology Plus - Title.ext) ---
echo "--- Japanology Plus (2014) ---"
show_dir="$TV/Japanology Plus (2014)/Season 01"

while IFS= read -r -d '' filepath; do
    filename="$(basename "$filepath")"
    ext="${filename##*.}"

    # Extract title: strip "NNN Japanology Plus - " or "NNN Japanology Plus  - " (double space variant)
    title="${filename#*Japanology Plus}"
    title="${title# - }"
    title="${title#  - }"
    title="${title%.*}"

    ep_plus=$((ep_plus + 1))
    ep_num=$(printf "%03d" "$ep_plus")

    newname="Japanology Plus - S01E${ep_num} - ${title}.${ext}"
    rename_pair "$filepath" "$show_dir/$newname"
done < <(find "$SRC" -maxdepth 1 -type f \( -name "*.mp4" -o -name "*.avi" -o -name "*.mkv" \) -regex '.*/[0-9]+ Japanology Plus.*' -print0 | sort -z)

echo "  Total: $ep_plus episodes"
echo ""

# --- BEGIN Japanology Mini (unnumbered: BEGIN Japanology Mini - Title.ext) ---
echo "--- Begin Japanology mini (2009) ---"
show_dir="$TV/Begin Japanology mini (2009)/Season 01"

while IFS= read -r -d '' filepath; do
    filename="$(basename "$filepath")"
    ext="${filename##*.}"

    title="${filename#BEGIN Japanology Mini - }"
    title="${title%.*}"

    ep_mini=$((ep_mini + 1))
    ep_num=$(printf "%03d" "$ep_mini")

    newname="Begin Japanology mini - S01E${ep_num} - ${title}.${ext}"
    rename_pair "$filepath" "$show_dir/$newname"
done < <(find "$SRC" -maxdepth 1 -type f \( -name "*.mp4" -o -name "*.avi" -o -name "*.mkv" \) -name "BEGIN Japanology Mini*" -print0 | sort -z)

echo "  Total: $ep_mini episodes"
echo ""

# --- Specials (NHK Special, etc.) ---
echo "--- Specials ---"
show_dir="$TV/Begin Japanology (2008)/Season 00"

while IFS= read -r -d '' filepath; do
    filename="$(basename "$filepath")"
    ext="${filename##*.}"

    title="${filename#NHK Special - }"
    title="${title%.*}"

    ep_special=$((ep_special + 1))
    ep_num=$(printf "%03d" "$ep_special")

    newname="Begin Japanology - S00E${ep_num} - ${title}.${ext}"
    rename_pair "$filepath" "$show_dir/$newname"
done < <(find "$SRC" -maxdepth 1 -type f \( -name "*.mp4" -o -name "*.avi" -o -name "*.mkv" \) -name "NHK Special*" -print0 | sort -z)

echo "  Total: $ep_special specials"
echo ""

# --- Summary ---
total=$((ep_begin + ep_plus + ep_mini + ep_special))
echo "=== Summary ==="
echo "  Begin Japanology:      $ep_begin episodes"
echo "  Japanology Plus:       $ep_plus episodes"
echo "  Begin Japanology mini: $ep_mini episodes"
echo "  Specials:              $ep_special"
echo "  Total:                 $total files"

if ! $APPLY; then
    echo ""
    echo "This was a dry run. Run with --apply to execute."
    echo ""
    echo "After renaming, you may want to:"
    echo "  1. Remove the old empty directory and tvshow.nfo"
    echo "  2. Rescan the library in Jellyfin"
fi
