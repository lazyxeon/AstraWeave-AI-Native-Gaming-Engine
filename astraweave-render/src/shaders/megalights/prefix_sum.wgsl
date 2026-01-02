// MegaLights: Prefix Sum Pass
// Calculates global offsets for the light index list
//
// NOTE: For the number of clusters we have (~10k-50k), a serial scan on a single GPU thread
// is actually faster (<50us) than a complex multi-stage parallel scan due to synchronization overhead.
// We dispatch (1, 1, 1) for this pass.

struct PrefixSumParams {
    element_count: u32,
    workgroup_size: u32,
    pad1: u32,
    pad2: u32,
}

@group(0) @binding(0) var<storage, read> input: array<u32>;       // light_counts
@group(0) @binding(1) var<storage, read_write> output: array<u32>; // light_offsets
@group(0) @binding(2) var<uniform> params: PrefixSumParams;

@compute @workgroup_size(1, 1, 1)
fn prefix_sum(@builtin(global_invocation_id) global_id: vec3<u32>) {
    // Serial scan
    var current_offset = 0u;
    
    for (var i = 0u; i < params.element_count; i = i + 1u) {
        output[i] = current_offset;
        current_offset = current_offset + input[i];
    }
    
    // Optionally store total count at the end if buffer allows, 
    // but for now we just need the offsets for the write pass.
}
