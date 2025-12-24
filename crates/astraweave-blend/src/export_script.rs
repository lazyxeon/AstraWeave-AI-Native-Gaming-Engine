//! Blender Python export script generation.
//!
//! This module generates Python scripts that Blender executes to export
//! .blend files to glTF/GLB format with all configured options.

use crate::options::{
    AnimationOptions, ConversionOptions, GltfExportOptions, LinkedLibraryOptions,
    MaterialOptions, MeshOptions, TextureOptions,
};
use std::path::Path;

/// Generates the complete Python export script.
pub fn generate_export_script(
    blend_path: &Path,
    output_path: &Path,
    options: &ConversionOptions,
    blend_hash: &str,
) -> String {
    let mut script = String::with_capacity(8192);

    // Script header
    script.push_str(&generate_header());

    // Import statements
    script.push_str(&generate_imports());

    // Configuration variables
    script.push_str(&generate_config(blend_path, output_path, options, blend_hash));

    // Texture unpacking function
    script.push_str(&generate_texture_unpacking(&options.textures, blend_hash));

    // Linked library handling
    if options.linked_libraries.process_recursively {
        script.push_str(&generate_linked_library_handler(&options.linked_libraries));
    }

    // Main export function
    script.push_str(&generate_main_export(&options.gltf, &options.mesh, &options.animation, &options.materials));

    // Script entry point
    script.push_str(&generate_entry_point());

    script
}

fn generate_header() -> String {
    r#"#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
AstraWeave Blender Export Script
Generated automatically - do not edit manually.

This script exports a .blend file to glTF/GLB format with
engine-optimized settings.
"""

"#
    .to_string()
}

fn generate_imports() -> String {
    r#"import bpy
import os
import sys
import json
import hashlib
import traceback
from pathlib import Path

"#
    .to_string()
}

fn generate_config(
    blend_path: &Path,
    output_path: &Path,
    options: &ConversionOptions,
    blend_hash: &str,
) -> String {
    let blend_path_escaped = blend_path.display().to_string().replace('\\', "\\\\");
    let output_path_escaped = output_path.display().to_string().replace('\\', "\\\\");
    let output_format = options.format.blender_format();

    format!(
        r#"# Configuration
BLEND_FILE = r"{blend_path_escaped}"
OUTPUT_FILE = r"{output_path_escaped}"
OUTPUT_FORMAT = "{output_format}"
BLEND_HASH = "{blend_hash}"
DRACO_COMPRESSION = {draco}
DRACO_LEVEL = {draco_level}
UNPACK_TEXTURES = {unpack}
MAX_TEXTURE_RESOLUTION = {max_res}

"#,
        draco = python_bool(options.gltf.draco_compression),
        draco_level = options.gltf.draco_compression_level,
        unpack = python_bool(options.textures.unpack_embedded),
        max_res = options.textures.max_resolution.map_or("None".to_string(), |r| r.to_string()),
    )
}

fn generate_texture_unpacking(options: &TextureOptions, blend_hash: &str) -> String {
    let texture_format = match options.format {
        crate::options::TextureFormat::Png => "PNG",
        crate::options::TextureFormat::Jpeg => "JPEG",
        crate::options::TextureFormat::WebP => "WEBP",
        crate::options::TextureFormat::Original => "NONE",
    };

    format!(
        r#"
def unpack_and_process_textures(output_dir):
    """Unpack embedded textures with deterministic naming."""
    texture_map = {{}}
    
    for image in bpy.data.images:
        if image.packed_file is None:
            continue
            
        # Generate deterministic filename: {{blend_hash}}_{{texture_name}}.ext
        original_name = Path(image.name).stem
        safe_name = "".join(c if c.isalnum() or c in "._-" else "_" for c in original_name)
        texture_filename = f"{blend_hash}_{{safe_name}}.{texture_format_lower}"
        texture_path = output_dir / texture_filename
        
        try:
            # Unpack to file
            image.unpack(method='WRITE_ORIGINAL')
            
            # Process resolution limit if needed
            if MAX_TEXTURE_RESOLUTION and (image.size[0] > MAX_TEXTURE_RESOLUTION or image.size[1] > MAX_TEXTURE_RESOLUTION):
                scale = MAX_TEXTURE_RESOLUTION / max(image.size[0], image.size[1])
                new_width = int(image.size[0] * scale)
                new_height = int(image.size[1] * scale)
                image.scale(new_width, new_height)
            
            # Save with desired format
            if "{texture_format}" != "NONE":
                scene = bpy.context.scene
                scene.render.image_settings.file_format = "{texture_format}"
                if "{texture_format}" == "JPEG":
                    scene.render.image_settings.quality = {jpeg_quality}
                image.save_render(str(texture_path))
                
            texture_map[image.name] = str(texture_path)
            print(f"Unpacked texture: {{image.name}} -> {{texture_path}}")
            
        except Exception as e:
            print(f"Warning: Failed to unpack texture {{image.name}}: {{e}}")
            
    return texture_map

"#,
        blend_hash = blend_hash,
        texture_format = texture_format,
        texture_format_lower = texture_format.to_lowercase(),
        jpeg_quality = options.jpeg_quality,
    )
}

fn generate_linked_library_handler(options: &LinkedLibraryOptions) -> String {
    let search_paths: Vec<String> = options
        .search_paths
        .iter()
        .map(|p| format!("r\"{}\"", p.display().to_string().replace('\\', "\\\\")))
        .collect();
    let search_paths_str = if search_paths.is_empty() {
        "[]".to_string()
    } else {
        format!("[{}]", search_paths.join(", "))
    };

    format!(
        r#"
# Linked library tracking for circular reference detection
_processed_libraries = set()

def process_linked_libraries(max_depth={max_depth}):
    """Recursively process linked .blend libraries."""
    search_paths = {search_paths}
    
    def process_library(lib_path, depth=0):
        if depth > max_depth:
            print(f"Warning: Max library depth exceeded for {{lib_path}}")
            return False
            
        # Normalize path for comparison
        normalized = os.path.normpath(os.path.abspath(lib_path))
        
        # Circular reference detection
        if normalized in _processed_libraries:
            {circular_handling}
            return True
            
        _processed_libraries.add(normalized)
        
        if not os.path.exists(lib_path):
            # Try search paths
            found = False
            for search_dir in search_paths:
                candidate = os.path.join(search_dir, os.path.basename(lib_path))
                if os.path.exists(candidate):
                    lib_path = candidate
                    found = True
                    break
            
            if not found:
                {missing_handling}
                return False
        
        print(f"Processing linked library (depth {{depth}}): {{lib_path}}")
        
        # Link the library data
        try:
            with bpy.data.libraries.load(lib_path, link=False) as (data_from, data_to):
                # Import all objects and meshes
                data_to.objects = data_from.objects
                data_to.meshes = data_from.meshes
                data_to.materials = data_from.materials
                data_to.textures = data_from.textures
                data_to.images = data_from.images
                
        except Exception as e:
            print(f"Error loading library {{lib_path}}: {{e}}")
            return False
            
        return True
    
    # Process all libraries in the current file
    for lib in bpy.data.libraries:
        if lib.filepath:
            abs_path = bpy.path.abspath(lib.filepath)
            process_library(abs_path, depth=1)
    
    print(f"Processed {{len(_processed_libraries)}} linked libraries")

"#,
        max_depth = options.max_recursion_depth,
        search_paths = search_paths_str,
        circular_handling = if options.detect_circular_references {
            r#"print(f"Skipping circular reference: {normalized}")
            return True"#
        } else {
            "pass"
        },
        missing_handling = match options.missing_library_action {
            crate::options::MissingLibraryAction::Skip => {
                r#"print(f"Skipping missing library: {lib_path}")
                return True"#
            }
            crate::options::MissingLibraryAction::Warn => {
                r#"print(f"Warning: Missing linked library: {lib_path}")
                return True"#
            }
            crate::options::MissingLibraryAction::Fail => {
                r#"raise FileNotFoundError(f"Missing linked library: {lib_path}")"#
            }
        },
    )
}

fn generate_main_export(
    gltf: &GltfExportOptions,
    mesh: &MeshOptions,
    animation: &AnimationOptions,
    materials: &MaterialOptions,
) -> String {
    format!(
        r#"
def export_gltf():
    """Perform the glTF export with configured options."""
    
    # Prepare output directory
    output_path = Path(OUTPUT_FILE)
    output_dir = output_path.parent
    output_dir.mkdir(parents=True, exist_ok=True)
    
    # Unpack textures if enabled
    if UNPACK_TEXTURES:
        unpack_and_process_textures(output_dir)
    
    # Export parameters
    export_params = {{
        'filepath': OUTPUT_FILE,
        'export_format': OUTPUT_FORMAT,
        
        # Mesh options
        'export_apply': {apply_modifiers},
        'export_tangents': {export_tangents},
        'export_normals': {export_normals},
        'export_colors': {export_vertex_colors},
        'use_mesh_edges': {export_loose_edges},
        'use_mesh_vertices': {export_loose_points},
        
        # Material options
        'export_materials': '{export_materials}',
        
        # Animation options
        'export_animations': {export_animations},
        'export_morph': {export_shape_keys},
        'export_morph_normal': {export_shape_keys},
        'export_morph_tangent': False,
        'export_nla_strips': {export_nla_strips},
        'optimize_animation_size': {optimize_animation},
        
        # Armature options  
        'export_skins': {export_skins},
        
        # Other options
        'export_cameras': {export_cameras},
        'export_lights': {export_lights},
        'export_extras': {export_extras},
        'export_yup': {y_up},
        
        # Selection
        'use_selection': {selected_only},
        'use_visible': {visible_only},
        'use_active_collection': {active_collection_only},
    }}
    
    # Add Draco compression if enabled
    if DRACO_COMPRESSION:
        export_params['export_draco_mesh_compression_enable'] = True
        export_params['export_draco_mesh_compression_level'] = DRACO_LEVEL
        export_params['export_draco_position_quantization'] = 14
        export_params['export_draco_normal_quantization'] = 10
        export_params['export_draco_texcoord_quantization'] = 12
        export_params['export_draco_color_quantization'] = 10
        export_params['export_draco_generic_quantization'] = 12
    
    # Add copyright if specified
    copyright_text = {copyright}
    if copyright_text:
        export_params['export_copyright'] = copyright_text
    
    print(f"Exporting to: {{OUTPUT_FILE}}")
    print(f"Format: {{OUTPUT_FORMAT}}")
    print(f"Draco: {{DRACO_COMPRESSION}}")
    
    # Perform export
    bpy.ops.export_scene.gltf(**export_params)
    
    # Verify output
    if not output_path.exists():
        raise RuntimeError(f"Export failed: output file not created at {{OUTPUT_FILE}}")
    
    file_size = output_path.stat().st_size
    print(f"Export complete: {{file_size}} bytes")
    
    return {{
        'output_file': OUTPUT_FILE,
        'file_size': file_size,
        'format': OUTPUT_FORMAT,
    }}

"#,
        apply_modifiers = python_bool(mesh.apply_modifiers),
        export_tangents = python_bool(mesh.export_tangents),
        export_normals = python_bool(mesh.export_normals),
        export_vertex_colors = python_bool(mesh.export_vertex_colors),
        export_loose_edges = python_bool(mesh.export_loose_edges),
        export_loose_points = python_bool(mesh.export_loose_points),
        export_materials = if materials.export_materials { "EXPORT" } else { "NONE" },
        export_animations = python_bool(animation.export_animations),
        export_shape_keys = python_bool(animation.export_shape_keys),
        export_nla_strips = python_bool(animation.export_nla_strips),
        optimize_animation = python_bool(animation.optimize_animation_size),
        export_skins = python_bool(gltf.export_skins),
        export_cameras = python_bool(gltf.export_cameras),
        export_lights = python_bool(gltf.export_lights),
        export_extras = python_bool(gltf.export_extras),
        y_up = python_bool(gltf.y_up),
        selected_only = python_bool(gltf.selected_only),
        visible_only = python_bool(gltf.visible_only),
        active_collection_only = python_bool(gltf.active_collection_only),
        copyright = gltf.copyright.as_ref().map_or("None".to_string(), |c| format!("\"{}\"", c.replace('"', "\\\""))),
    )
}

fn generate_entry_point() -> String {
    r#"
def main():
    """Main entry point."""
    result = {'success': False, 'error': None, 'output': None}
    
    try:
        print(f"Loading: {BLEND_FILE}")
        
        # Open the blend file
        bpy.ops.wm.open_mainfile(filepath=BLEND_FILE)
        
        # Process linked libraries if function exists
        if 'process_linked_libraries' in dir():
            process_linked_libraries()
        
        # Perform export
        export_result = export_gltf()
        
        result['success'] = True
        result['output'] = export_result
        
    except Exception as e:
        result['error'] = str(e)
        result['traceback'] = traceback.format_exc()
        print(f"Export failed: {e}", file=sys.stderr)
        traceback.print_exc()
        sys.exit(1)
    
    # Write result JSON for parsing by Rust
    result_file = OUTPUT_FILE + '.result.json'
    with open(result_file, 'w') as f:
        json.dump(result, f, indent=2)
    
    print(f"Result written to: {result_file}")
    print("Export completed successfully!")
    
if __name__ == '__main__':
    main()
"#
    .to_string()
}

/// Converts a bool to Python boolean string.
fn python_bool(b: bool) -> &'static str {
    if b { "True" } else { "False" }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::options::OutputFormat;
    use std::path::PathBuf;

    #[test]
    fn test_generate_script_basic() {
        let blend_path = PathBuf::from("/test/model.blend");
        let output_path = PathBuf::from("/output/model.glb");
        let options = ConversionOptions::default();
        
        let script = generate_export_script(&blend_path, &output_path, &options, "abc123");
        
        assert!(script.contains("BLEND_FILE"));
        assert!(script.contains("OUTPUT_FILE"));
        assert!(script.contains("def export_gltf"));
        assert!(script.contains("def main"));
        assert!(script.contains("abc123"));
    }

    #[test]
    fn test_generate_script_with_draco() {
        let blend_path = PathBuf::from("/test/model.blend");
        let output_path = PathBuf::from("/output/model.glb");
        let mut options = ConversionOptions::default();
        options.gltf.draco_compression = true;
        
        let script = generate_export_script(&blend_path, &output_path, &options, "hash");
        
        assert!(script.contains("DRACO_COMPRESSION = True"));
        assert!(script.contains("export_draco_mesh_compression_enable"));
    }

    #[test]
    fn test_generate_script_linked_libraries() {
        let blend_path = PathBuf::from("/test/model.blend");
        let output_path = PathBuf::from("/output/model.glb");
        let mut options = ConversionOptions::default();
        options.linked_libraries.process_recursively = true;
        
        let script = generate_export_script(&blend_path, &output_path, &options, "hash");
        
        assert!(script.contains("process_linked_libraries"));
        assert!(script.contains("_processed_libraries"));
    }

    #[test]
    fn test_python_bool() {
        assert_eq!(python_bool(true), "True");
        assert_eq!(python_bool(false), "False");
    }

    #[test]
    fn test_output_format_blender_string() {
        assert_eq!(OutputFormat::GlbBinary.blender_format(), "GLB");
        assert_eq!(OutputFormat::GltfEmbedded.blender_format(), "GLTF_EMBEDDED");
        assert_eq!(OutputFormat::GltfSeparate.blender_format(), "GLTF_SEPARATE");
    }
}
