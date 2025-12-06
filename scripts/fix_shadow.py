import
re
  
with open('astraweave-render-bevy/src/render/shadow.rs', 'r', encoding='utf-8') as f:  
    content = f.read()  
  
old = "        for i in 1..CASCADE_COUNT {\n            let ratio = i as f32 / CASCADE_COUNT as f32;\n            // Logarithmic distribution (better quality near camera)\n            split_distances[i] = camera_near * (camera_far / camera_near).powf(ratio);\n        }"  
new = "        for (i, split_distance) in split_distances.iter_mut().enumerate().take(CASCADE_COUNT).skip(1) {\n            let ratio = i as f32 / CASCADE_COUNT as f32;\n            // Logarithmic distribution (better quality near camera)\n            *split_distance = camera_near * (camera_far / camera_near).powf(ratio);\n        }"  
content = content.replace(old, new)  
  
with open('astraweave-render-bevy/src/render/shadow.rs', 'w', encoding='utf-8') as f:  
    f.write(content)  
print('Fixed')  
