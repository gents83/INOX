// Ray Type Constants for Wavefront Path Tracer

const RAY_TYPE_PRIMARY: u32 = 0u;
const RAY_TYPE_SHADOW: u32 = 1u;
const RAY_TYPE_DIFFUSE_BOUNCE: u32 = 2u;
const RAY_TYPE_SPECULAR_BOUNCE: u32 = 3u;
const RAY_TYPE_TRANSMISSION: u32 = 4u;
const RAY_TYPE_REFLECTION: u32 = 5u;
const RAY_TYPE_REFRACTION: u32 = 6u;
const RAY_TYPE_AO: u32 = 7u;

// Ray flags
const RAY_FLAG_ACTIVE: u32 = 1u << 0;
const RAY_FLAG_HIT: u32 = 1u << 1;
const RAY_FLAG_TERMINATED: u32 = 1u << 2;
const RAY_FLAG_ANY_HIT: u32 = 1u << 3; // For shadow rays - stop at first hit

// Sampling strategies
const SAMPLING_UNIFORM_HEMISPHERE: u32 = 0u;
const SAMPLING_COSINE_WEIGHTED: u32 = 1u;
const SAMPLING_GGX_IMPORTANCE: u32 = 2u;
const SAMPLING_STRATIFIED: u32 = 3u;
