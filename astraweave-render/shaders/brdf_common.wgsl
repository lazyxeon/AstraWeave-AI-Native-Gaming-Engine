// brdf_common.wgsl — Canonical BRDF functions (single source of truth)
// All PBR shaders must use these functions for consistent shading.
//
// Model: Cook-Torrance specular + Disney/Burley diffuse
// NDF: GGX/Trowbridge-Reitz
// Geometry: Height-correlated Smith-GGX (Heitz 2014)
// Fresnel: Schlick with saturate for numerical safety
// Diffuse: Disney/Burley (energy-conserving, unlike Lambertian)
// PI, TWO_PI, HALF_PI, INV_PI provided by constants.wgsl (prepended on Rust side).

fn fresnel_schlick(cos_theta: f32, F0: vec3<f32>) -> vec3<f32> {
    return F0 + (1.0 - F0) * pow(saturate(1.0 - cos_theta), 5.0);
}

fn fresnel_schlick_roughness(cos_theta: f32, F0: vec3<f32>, roughness: f32) -> vec3<f32> {
    let one_minus_rough = vec3<f32>(1.0 - roughness);
    return F0 + (max(one_minus_rough, F0) - F0) * pow(saturate(1.0 - cos_theta), 5.0);
}

fn distribution_ggx(NdotH: f32, roughness: f32) -> f32 {
    let a = roughness * roughness;
    let a2 = a * a;
    let NdotH2 = NdotH * NdotH;
    let denom = NdotH2 * (a2 - 1.0) + 1.0;
    return a2 / (PI * denom * denom + 1e-7);
}

// Height-correlated Smith-GGX visibility (Heitz 2014).
// Returns V = G / (4 * NdotV * NdotL), canceling the Cook-Torrance denominator.
// More physically accurate than the uncorrelated Schlick-GGX approximation.
fn visibility_smith_ggx(NdotV: f32, NdotL: f32, roughness: f32) -> f32 {
    let a = roughness * roughness;
    let a2 = a * a;
    let lambda_v = NdotL * sqrt(NdotV * NdotV * (1.0 - a2) + a2);
    let lambda_l = NdotV * sqrt(NdotL * NdotL * (1.0 - a2) + a2);
    return 0.5 / (lambda_v + lambda_l + 1e-7);
}

// Disney/Burley diffuse (energy-conserving at grazing angles).
// Schlick Fresnel-weighted retroreflection; more accurate than Lambertian.
fn diffuse_burley(NdotV: f32, NdotL: f32, VdotH: f32, roughness: f32) -> f32 {
    let fd90 = 0.5 + 2.0 * roughness * VdotH * VdotH;
    let light_scatter = 1.0 + (fd90 - 1.0) * pow(1.0 - NdotL, 5.0);
    let view_scatter = 1.0 + (fd90 - 1.0) * pow(1.0 - NdotV, 5.0);
    return light_scatter * view_scatter / PI;
}

// ======================================================================
// Unified PBR BRDF
// ======================================================================

// T.2d.F (2026-07-25, director-ratified): the material-LOD tiers are RETIRED.
// The LOD1|2 threshold (pixel footprint 2.0) was a visible camera-anchored
// detail boundary, and because fwidth(world_pos) includes the height
// derivative the per-pixel tier selection dithered over rough ground (40-55%
// of far-field high-frequency energy was tier flicker). Every fragment now
// shades with the one full BRDF below, so appearance judgments no longer
// depend on where the camera was. Do NOT reintroduce a stepped shading tier
// (`renderer.rs::material_lod_tiers_are_retired` enforces this); the ratified
// fallback for any future perf need is a falloff CONTINUOUS in footprint.
// Full trail: docs/audits/T2D_CAMERA_LIGHT.md §10, docs/audits/T2DF_OUTCOME.md.

// Unified PBR BRDF: Cook-Torrance specular + Burley diffuse + Kulla-Conty
// multiscatter energy compensation.
// Returns (diffuse + specular + multiscatter) * NdotL — ready to multiply by
// radiance and shadow.
fn evaluate_brdf(
    N: vec3<f32>, V: vec3<f32>, L: vec3<f32>,
    base_color: vec3<f32>, metallic: f32, roughness: f32, F0: vec3<f32>
) -> vec3<f32> {
    let H = normalize(V + L);
    let NdotL = max(dot(N, L), 0.0);
    let NdotV = max(dot(N, V), 0.0);
    let NdotH = max(dot(N, H), 0.0);
    let VdotH = max(dot(V, H), 0.0);

    let D = distribution_ggx(NdotH, roughness);
    let Vis = visibility_smith_ggx(NdotV, NdotL, roughness);
    let F = fresnel_schlick(VdotH, F0);
    let specular = D * Vis * F;

    let kd = (vec3<f32>(1.0) - F) * (1.0 - metallic);
    let diffuse = kd * base_color * diffuse_burley(NdotV, NdotL, VdotH, roughness);

    // Kulla-Conty multiscatter energy compensation (Turquin 2019 analytical
    // approximation). Single-scatter BRDF loses 20-40% energy at
    // roughness > 0.5; this recovers the inter-reflection energy.
    let a = roughness * roughness;
    let E = 1.0 - 1.4594 * a * NdotV + 0.8868 * a * a * NdotV * NdotV
          + 0.5716 * a * NdotV - 0.0159 * a * a;
    let E_l = 1.0 - 1.4594 * a * NdotL + 0.8868 * a * a * NdotL * NdotL
            + 0.5716 * a * NdotL - 0.0159 * a * a;
    let E_clamp = saturate(E);
    let E_l_clamp = saturate(E_l);
    // Average Fresnel: integrated Fresnel over hemisphere (Lagarde 2014)
    let F_avg = F0 + (1.0 - F0) / 21.0;
    // Multiscatter compensation: Fms * Favg, normalized
    let Fms = (1.0 - E_clamp) * (1.0 - E_l_clamp) / (PI * (1.0 - E_clamp) + 1e-7);
    let multiscatter = Fms * F_avg / (1.0 - F_avg * (1.0 - E_clamp) + 1e-7);

    return (diffuse + specular + multiscatter) * NdotL;
}
