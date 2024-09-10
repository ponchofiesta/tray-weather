#!/bin/bash
inkscape=$(cygpath -m 'C:\Users\AJHMF7Y\AppData\Local\Programs\inkscape-1.2.1_2022-07-14_9c6d41e410-x64\bin\inkscape')
mkdir -p export_dir
for file in ico/*.ico; do
    filename=$(basename "$file")
    #"$inkscape" "$file" --export-type=png --export-width=16 --export-height=16 -o "export_ico/${filename%.svg}.png"
    echo "${filename%.ico} ICON \"assets/weathericons/ico/${filename%.ico}.ico\"" >> weathericons.res
done
