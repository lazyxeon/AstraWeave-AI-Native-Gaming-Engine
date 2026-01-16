
import os

file_path = r"tools\aw_editor\src\main.rs"

with open(file_path, "r", encoding="utf-8") as f:
    lines = f.readlines()

start_index = -1
end_index = -1

# Look for the specific TopBottomPanel call in main loop
start_marker = 'egui::TopBottomPanel::top("top")'

# We track occurrences because there might be redundant ones (though unlikely inside update if unique)
# Actually, I added the method definition above, so looking for start_marker will find the DEFINITION too.
# I need to find the one inside 'fn update'. 'fn update' is typically later in file?
# Actually 'fn update' is at line 4400. The newly added method is at line 2500.
# BUT I edited the file just now.

# We need to target the block around line 5600+ (now shifted).
# Best way is to search relative to surrounding code in update.

# Context:
#         self.animation_panel.update(frame_time);
# 
#         egui::TopBottomPanel::top("top")

search_context = "self.animation_panel.update(frame_time);"

lines_iter = enumerate(lines)
for i, line in lines_iter:
    if search_context in line:
        # found context, next non-empty line should be start
        # Let's peek forward
        for j in range(i+1, len(lines)):
            if start_marker in lines[j]:
                start_index = j
                break
        break

if start_index == -1:
    print("Start marker not found after context.")
    exit(1)

# Now find matching });
# It has 8 spaces indent
indent = "        " # 8 spaces
closing_brace = indent + "});"

for k in range(start_index + 1, len(lines)):
    if lines[k].rstrip() == closing_brace:
        end_index = k
        break

if end_index == -1:
    print("End marker not found.")
    exit(1)

print(f"Replacing block from line {start_index + 1} to {end_index + 1}")

new_code = ["        self.show_top_panel(ctx);\n"]

new_lines = lines[:start_index] + new_code + lines[end_index+1:]

with open(file_path, "w", encoding="utf-8") as f:
    f.writelines(new_lines)

print("Successfully replaced top panel in update")
