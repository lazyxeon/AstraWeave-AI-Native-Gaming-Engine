#!/bin/bash

# Centralized exclusion list for CI workflows
# This file defines the problematic crates that should be excluded from workspace operations

# Export the standardized exclusion pattern
export PROBLEMATIC_CRATES=(
    "astraweave-author"       # rhai sync/send trait issues
    "visual_3d"               # egui/winit API mismatches  
    "ui_controls_demo"        # egui API compatibility issues
    "npc_town_demo"           # Multiple API mismatches
    "rhai_authoring"          # depends on broken astraweave-author
    "cutscene_render_demo"    # graphics API issues
    "weaving_playground"      # dependency issues
    "combat_physics_demo"     # API mismatches
    "navmesh_demo"            # API mismatches
    "physics_demo3d"          # API mismatches
    "debug_toolkit_demo"      # rand version conflicts and egui API issues
    "aw_editor"               # eframe API issues and Send/Sync trait problems
)

# Convert array to cargo exclusion format
export CARGO_EXCLUSIONS=""
for crate in "${PROBLEMATIC_CRATES[@]}"; do
    CARGO_EXCLUSIONS="$CARGO_EXCLUSIONS --exclude $crate"
done

# Core working crates for targeted builds
export CORE_CRATES=(
    "astraweave-core"
    "astraweave-ai"
    "astraweave-physics"
    "astraweave-nav"
    "astraweave-render"
    "astraweave-gameplay"
    "astraweave-audio"
    "astraweave-input"
    "astraweave-ui"
    "astraweave-net"
    "astraweave-director"
    "astraweave-memory"
    "astraweave-persona"
    "astraweave-ipc"
    "astraweave-llm"
    "astraweave-sdk"
    "hello_companion"
)

# Convert core crates to cargo package format
export CORE_PACKAGES=""
for crate in "${CORE_CRATES[@]}"; do
    CORE_PACKAGES="$CORE_PACKAGES -p $crate"
done

# Working examples for demo testing
export WORKING_EXAMPLES=(
    "hello_companion"
    "adaptive_boss"
    "companion_profile"
    "ipc_loopback"
    "llm_toolcall"
    "llm_integration"
    "phase_director"
    "persona_loader"
    "coop_server"
    "coop_client"
    "audio_spatial_demo"
    "dialogue_voice_demo"
)

# Convert working examples to cargo package format
export EXAMPLE_PACKAGES=""
for example in "${WORKING_EXAMPLES[@]}"; do
    EXAMPLE_PACKAGES="$EXAMPLE_PACKAGES -p $example"
done

# Function to print exclusions (for debugging)
print_exclusions() {
    echo "Problematic crates excluded from builds:"
    printf '%s\n' "${PROBLEMATIC_CRATES[@]}"
    echo ""
    echo "Cargo exclusion flags: $CARGO_EXCLUSIONS"
}

# Function to print core crates (for debugging)
print_core_crates() {
    echo "Core crates for targeted builds:"
    printf '%s\n' "${CORE_CRATES[@]}"
    echo ""
    echo "Cargo package flags: $CORE_PACKAGES"
}

# Export functions for use in workflows
export -f print_exclusions
export -f print_core_crates