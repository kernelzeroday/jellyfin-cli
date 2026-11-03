#!/usr/bin/env bash
set -euo pipefail

# Regenerate NFO files for Japanology episodes from filenames.
# The old NFOs had wrong metadata (title/episode number mismatch) from before
# the rename script reorganized the flat directory. This replaces each NFO
# with data derived from the filename, locked so TVDB can't override.
#
# Usage:
#   ./fix-japanology-nfo.sh              # dry run
#   ./fix-japanology-nfo.sh --apply      # overwrite NFOs

TV="/data2/media/TV"
APPLY=false
if [[ "${1:-}" == "--apply" ]]; then
    APPLY=true
fi

count=0

process_show() {
    local show_dir="$1"
    local show_name="$2"

    if [[ ! -d "$show_dir" ]]; then
        echo "SKIP (not found): $show_dir"
        return
    fi

    echo "--- $show_name ---"

    while IFS= read -r -d '' media_file; do
        local filename
        filename="$(basename "$media_file")"
        local nfo_file="${media_file%.*}.nfo"

        # Parse SxxExx from filename
        local se_match
        se_match=$(echo "$filename" | grep -oE 'S([0-9]+)E([0-9]+)' || true)
        if [[ -z "$se_match" ]]; then
            echo "  SKIP (no SxxExx): $filename"
            continue
        fi

        local season_num episode_num title ext
        season_num=$(echo "$se_match" | sed 's/S0*\([0-9]*\)E.*/\1/')
        episode_num=$(echo "$se_match" | sed 's/S[0-9]*E0*\([0-9]*\)/\1/')

        # Extract title: everything after "SxxExx - " and before the extension
        title=$(echo "$filename" | sed "s/.*${se_match} - //" | sed 's/\.[^.]*$//')

        if [[ -z "$title" ]]; then
            echo "  SKIP (no title): $filename"
            continue
        fi

        count=$((count + 1))

        if $APPLY; then
            rm -f "$nfo_file"
            cat > "$nfo_file" << NFOEOF
<?xml version="1.0" encoding="utf-8" standalone="yes"?>
<episodedetails>
  <title>${title}</title>
  <showtitle>${show_name}</showtitle>
  <season>${season_num}</season>
  <episode>${episode_num}</episode>
  <lockdata>true</lockdata>
</episodedetails>
NFOEOF
            echo "  WROTE: $(basename "$nfo_file") -> S${season_num}E${episode_num} \"${title}\""
        else
            echo "  $filename"
            echo "    -> S${season_num}E${episode_num} \"${title}\""
            if [[ -f "$nfo_file" ]]; then
                local old_title
                old_title=$(grep -oP '(?<=<title>).*(?=</title>)' "$nfo_file" 2>/dev/null || echo "?")
                local old_ep
                old_ep=$(grep -oP '(?<=<episode>).*(?=</episode>)' "$nfo_file" 2>/dev/null || echo "?")
                if [[ "$old_title" != "$title" || "$old_ep" != "$episode_num" ]]; then
                    echo "    OLD NFO: E${old_ep} \"${old_title}\" <- MISMATCH"
                fi
            fi
        fi
    done < <(find "$show_dir" -type f \( -name "*.mp4" -o -name "*.avi" -o -name "*.mkv" \) -print0 | sort -z)
}

echo "=== Japanology NFO Fix ==="
if $APPLY; then
    echo "*** APPLY MODE — NFO files will be overwritten ***"
else
    echo "(dry run — pass --apply to execute)"
fi
echo ""

process_show "$TV/Begin Japanology (2008)" "Begin Japanology"
echo ""
process_show "$TV/Japanology Plus (2014)" "Japanology Plus"
echo ""
process_show "$TV/Begin Japanology mini (2009)" "Begin Japanology mini"
echo ""

echo "=== Total: $count episodes ==="
if ! $APPLY; then
    echo ""
    echo "This was a dry run. Run with --apply to overwrite NFOs."
    echo "After applying, trigger a Jellyfin library scan."
fi
