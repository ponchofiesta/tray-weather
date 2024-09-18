#!/bin/bash -e
declare -a res=(16 20 24 32 40 48 64 256)
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
