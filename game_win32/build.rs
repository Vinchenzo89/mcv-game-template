extern crate gl_generator;

use gl_generator::{Registry, Api, Profile, Fallbacks};
use std::env;
use std::fs::File;
use std::path::Path;

fn main() {
    // GL Generator generates GL bindings to OUT_DIR
    let dest = env::var("OUT_DIR").unwrap();
    let mut file = File::create(&Path::new(&dest).join("gl_bindings.rs")).unwrap();

    Registry::new(
        Api::Gl, (4, 5), 
        Profile::Compatibility, 
        Fallbacks::All, 
        [
            /*  Extensions go here
            "GL_AMD_depth_clamp_separate",
            "GL_APPLE_vertex_array_object",
            "GL_ARB_bindless_texture",
            "GL_ARB_buffer_storage",
            "GL_ARB_compute_shader",
            "GL_ARB_copy_buffer",
            "GL_ARB_debug_output",
            "GL_ARB_depth_texture",
            "GL_ARB_direct_state_access",
            "GL_ARB_draw_buffers",
            "GL_ARB_ES2_compatibility",
            "GL_ARB_ES3_compatibility",
            "GL_ARB_ES3_1_compatibility",
            "GL_ARB_ES3_2_compatibility",
            "GL_ARB_framebuffer_sRGB",
            "GL_ARB_geometry_shader4",
            "GL_ARB_gl_spirv",
            "GL_ARB_gpu_shader_fp64",
            "GL_ARB_gpu_shader_int64",
            "GL_ARB_invalidate_subdata",
            "GL_ARB_multi_draw_indirect",
            "GL_ARB_occlusion_query",
            "GL_ARB_pixel_buffer_object",
            "GL_ARB_robustness",
            "GL_ARB_seamless_cube_map",
            "GL_ARB_shader_image_load_store",
            "GL_ARB_shader_objects",
            "GL_ARB_texture_buffer_object",
            "GL_ARB_texture_float",
            "GL_ARB_texture_multisample",
            "GL_ARB_texture_rg",
            "GL_ARB_texture_rgb10_a2ui",
            "GL_ARB_transform_feedback3",
            "GL_ARB_vertex_buffer_object",
            "GL_ARB_vertex_shader",
            "GL_ATI_draw_buffers",
            "GL_ATI_meminfo",
            "GL_EXT_debug_marker",
            "GL_EXT_direct_state_access",
            "GL_EXT_framebuffer_blit",
            "GL_EXT_framebuffer_multisample",
            "GL_EXT_framebuffer_object",
            "GL_EXT_framebuffer_sRGB",
            "GL_EXT_gpu_shader4",
            "GL_EXT_packed_depth_stencil",
            "GL_EXT_provoking_vertex",
            "GL_EXT_texture_array",
            "GL_EXT_texture_buffer_object",
            "GL_EXT_texture_compression_s3tc",
            "GL_EXT_texture_filter_anisotropic",
            "GL_EXT_texture_integer",
            "GL_EXT_texture_sRGB",
            "GL_EXT_transform_feedback",
            "GL_GREMEDY_string_marker",
            "GL_KHR_robustness",
            "GL_NVX_gpu_memory_info",
            "GL_NV_conditional_render",
            "GL_NV_vertex_attrib_integer_64bit",
            */
        ])
    .write_bindings(gl_generator::GlobalGenerator, &mut file)
    .unwrap();
}