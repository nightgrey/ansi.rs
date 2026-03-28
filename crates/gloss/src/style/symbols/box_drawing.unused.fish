#!/usr/bin/env fish

set temp_file (mktemp)

for line in (cat "box_drawing.rs")
    if string match -qr '".*"' -- "$line"

        set str (string match -r '"(.*)"' -- $line)[2]

        if command rg --glob "!box_drawing.*" -q --fixed-strings -- $str ./
        else
            echo "Unused: $str"
            echo "$line" >> $temp_file
        end
    else
        echo $line >> $temp_file
    end
end

mv $temp_file ./box_drawing.unused.rs