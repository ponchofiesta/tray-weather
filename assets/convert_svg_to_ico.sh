#!/bin/bash -e

# Size recommended here: https://learn.microsoft.com/de-de/windows/win32/shell/notification-area?redirectedfrom=MSDN#install_icon
declare -a res=(16 32)
for f in *.svg
do
    declare -a png_files=()
    for r in "${res[@]}"
    do
        inkscape -w $r -h $r "$f" -o "$r".png
        png_files+=("${r}".png)
    done
    magick "${png_files[@]}" -compress jpeg "${f%.*}".ico
    rm -f "${png_files[@]}"
done
