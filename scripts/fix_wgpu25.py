#!/usr/bin/env python3
"""Fix WGPU 25.0 breaking changes across the codebase."""

import re
import os
from pathlib import Path

def fix_entry_point(content):
    """Wrap entry_point strings in Some()."""
    # Match entry_point: "string" and wrap in Some()
    return re.sub(
        r'(entry_point:)\s*"([^"]+)"',
        r'\1 Some("\2")',
        content
    )

def fix_texture_view_descriptor(content):
    """Add usage field to TextureViewDescriptor."""
    # Match TextureViewDescriptor { \n spaces label:
    # Insert usage: None, between { and label
    return re.sub(
        r'(wgpu::TextureViewDescriptor\s*\{\s*\n)(\s+)(label:)',
        r'\1\2usage: None,\n\2\3',
        content
    )

def fix_device_descriptor(content):
    """Add trace field to DeviceDescriptor."""
    # Find memory_hints line and add trace: None, after it
    content = re.sub(
        r'(memory_hints:\s*wgpu::MemoryHints::default\(\),)',
        r'\1\n                trace: None,',
        content
    )
    
    # Also handle cases where DeviceDescriptor is missing both fields
    # Look for required_limits: ... }, and add memory_hints + trace before the closing brace
    content = re.sub(
        r'(required_limits:\s*wgpu::Limits::[^,]+,)\s*\n(\s*)\}',
        r'\1\n\2    memory_hints: wgpu::MemoryHints::default(),\n\2    trace: None,\n\2}',
        content
    )
    
    return content

def fix_request_adapter(content):
    """Remove ok_or_else from request_adapter (now returns Result)."""
    # request_adapter().await.ok_or_else(...) -> request_adapter().await?
    return re.sub(
        r'(\.request_adapter\([^)]+\))\s*\.await\s*\.ok_or_else\([^)]+\)\?',
        r'\1.await?',
        content
    )

def fix_render_pipeline_descriptor(content):
    """Add cache: None to RenderPipelineDescriptor."""
    # Find RenderPipelineDescriptor { ... multiview: ... } and add cache before closing
    # Look for multiview field (last field in most cases)
    content = re.sub(
        r'(multiview:\s*[^,\n]+,)\s*\n(\s*)\}',
        r'\1\n\2    cache: None,\n\2}',
        content
    )
    
    # Also handle cases where fragment is the last field
    content = re.sub(
        r'(?s)(fragment:\s*Some\(\s*[^\{]+\{[^{}]*(?:\{[^{}]*\}[^{}]*)*\}\s*\),)\s*\n(\s*)\}\s*\)(?!,)',
        r'\1\n\2    cache: None,\n\2})',
        content
    )
    
    return content

def fix_compute_pipeline_descriptor(content):
    """Add cache: None to ComputePipelineDescriptor."""
    # Find ComputePipelineDescriptor { ... and add cache before closing }
    content = re.sub(
        r'(wgpu::ComputePipelineDescriptor\s*\{[^}]*)(compile_options:[^,\n]+,)\s*\n(\s*)\}',
        r'\1\2\n\3    cache: None,\n\3}',
        content
    )
    
    # Handle cases where compilation_options or entry_point is last
    content = re.sub(
        r'(wgpu::ComputePipelineDescriptor\s*\{[^}]*module:[^,\n]+,)\s*\n(\s*)\}',
        r'\1\n\2    cache: None,\n\2}',
        content
    )
    
    return content

def fix_wgpu_maintain(content):
    """Replace wgpu::Maintain with wgpu::MaintainBase."""
    return re.sub(
        r'wgpu::Maintain::',
        r'wgpu::MaintainBase::',
        content
    )

def process_file(filepath):
    """Apply all fixes to a single file."""
    try:
        with open(filepath, 'r', encoding='utf-8') as f:
            content = f.read()
        
        original = content
        content = fix_entry_point(content)
        content = fix_texture_view_descriptor(content)
        content = fix_device_descriptor(content)
        content = fix_request_adapter(content)
        content = fix_render_pipeline_descriptor(content)
        content = fix_compute_pipeline_descriptor(content)
        content = fix_wgpu_maintain(content)
        
        if content != original:
            with open(filepath, 'w', encoding='utf-8', newline='\n') as f:
                f.write(content)
            print(f"Fixed: {filepath}")
            return True
        return False
    except Exception as e:
        print(f"Error processing {filepath}: {e}")
        return False

def main():
    """Process all Rust files in the workspace."""
    root = Path(".")
    rust_files = list(root.rglob("*.rs"))
    
    fixed_count = 0
    for filepath in rust_files:
        if process_file(filepath):
            fixed_count += 1
    
    print(f"\nProcessed {len(rust_files)} files, fixed {fixed_count} files")

if __name__ == "__main__":
    main()
