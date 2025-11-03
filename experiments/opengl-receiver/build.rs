use std::{env, fs::File, path::PathBuf};
use gl_generator::{Api, Fallbacks, Profile, Registry};

fn main() {
    // Diretório de saída (onde o cargo coloca ficheiros gerados)
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    // ========================
    // Geração de bindings EGL
    // ========================
    let egl_path = out_dir.join("egl_bindings.rs");
    let mut egl_file = File::create(&egl_path).unwrap();
    Registry::new(Api::Egl, (1, 5), Profile::Core, Fallbacks::All, [
        "EGL_KHR_image_base",
        "EGL_EXT_image_dma_buf_import",
        "EGL_EXT_image_dma_buf_import_modifiers",
        "EGL_MESA_image_dma_buf_export",
        "EGL_ANDROID_native_fence_sync",
        "EGL_KHR_fence_sync",
        "EGL_KHR_surfaceless_context",
    ],)
        .write_bindings(gl_generator::GlobalGenerator, &mut egl_file)
        .unwrap();

    // =========================
    // Geração de bindings OpenGL
    // =========================
    let gl_path = out_dir.join("gl_bindings.rs");
    let mut gl_file = File::create(&gl_path).unwrap();
    Registry::new(Api::Gles2, (3, 3), Profile::Core, Fallbacks::All, [
            "GL_OES_EGL_image",
            "GL_OES_EGL_image_external",
            "GL_EXT_memory_object_fd",
            "GL_EXT_semaphore_fd"
        ])
        .write_bindings(gl_generator::GlobalGenerator, &mut gl_file)
        .unwrap();

    // Diz ao cargo para recompilar se este ficheiro mudar
    println!("cargo:rerun-if-changed=build.rs");
}
