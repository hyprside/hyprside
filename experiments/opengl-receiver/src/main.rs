#![allow(warnings)]
use glfw::{Action, Context, Key, WindowEvent};
#[allow(warnings)]
mod gl {
    include!(concat!(env!("OUT_DIR"), "/gl_bindings.rs"));
}
#[allow(warnings)]
mod egl {
	pub type khronos_utime_nanoseconds_t = u64;
	pub type khronos_uint64_t = u64;
	pub type khronos_ssize_t = isize;

	pub type EGLint = i32;
	pub type EGLNativeDisplayType = *mut std::ffi::c_void;
	pub type EGLNativePixmapType = *mut std::ffi::c_void;
	pub type EGLNativeWindowType = *mut std::ffi::c_void;
	pub type NativeDisplayType = *mut std::ffi::c_void;
	pub type NativePixmapType = *mut std::ffi::c_void;
	pub type NativeWindowType = *mut std::ffi::c_void;

    include!(concat!(env!("OUT_DIR"), "/egl_bindings.rs"));
}
use std::ffi::{c_void, CString};
use std::os::fd::AsRawFd;
use std::os::unix::net::{UnixListener, UnixStream};
use std::ptr;
use std::sync::OnceLock;
use gl::types::{GLint, GLuint};

struct GLResources {
    program: GLuint,
    vao: GLuint,
    vbo: GLuint,
    ebo: GLuint,
    u_tex_loc: GLint,
}

static GL_LOADED: OnceLock<()> = OnceLock::new();
static GL_RESOURCES: OnceLock<GLResources> = OnceLock::new();

fn ensure_gl_loaded() {
    GL_LOADED.get_or_init(|| {
        unsafe {
            gl::load_with(|s| {
                let cstr = CString::new(s).unwrap();
                let addr = glfw::ffi::glfwGetProcAddress(cstr.as_ptr());
                addr.map_or_else(std::ptr::null, |f| f as _)
            });
        }
    });
}

unsafe fn compile_shader(src: &str, kind: GLuint) -> Result<GLuint, String> {
    let shader = gl::CreateShader(kind);
    let c_src = CString::new(src).unwrap();
    gl::ShaderSource(shader, 1, &c_src.as_ptr(), ptr::null());
    gl::CompileShader(shader);

    let mut status: GLint = 0;
    gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut status);
    if status == gl::FALSE as GLint {
        let mut len: GLint = 0;
        gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut len);
        let mut buf = vec![0u8; len as usize];
        gl::GetShaderInfoLog(
            shader,
            len,
            ptr::null_mut(),
            buf.as_mut_ptr() as *mut i8
        );
        gl::DeleteShader(shader);
        let log = String::from_utf8_lossy(&buf).to_string();
        return Err(format!("Shader compile error: {}", log));
    }
    Ok(shader)
}

unsafe fn link_program(vs: GLuint, fs: GLuint) -> Result<GLuint, String> {
    let program = gl::CreateProgram();
    gl::AttachShader(program, vs);
    gl::AttachShader(program, fs);
    gl::LinkProgram(program);

    let mut status: GLint = 0;
    gl::GetProgramiv(program, gl::LINK_STATUS, &mut status);
    if status == gl::FALSE as GLint {
        let mut len: GLint = 0;
        gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut len);
        let mut buf = vec![0u8; len as usize];
        gl::GetProgramInfoLog(
            program,
            len,
            ptr::null_mut(),
            buf.as_mut_ptr() as *mut i8
        );
        gl::DeleteProgram(program);
        let log = String::from_utf8_lossy(&buf).to_string();
        return Err(format!("Program link error: {}", log));
    }

    // shaders can be detached/deleted after successful link
    gl::DetachShader(program, vs);
    gl::DetachShader(program, fs);
    gl::DeleteShader(vs);
    gl::DeleteShader(fs);

    Ok(program)
}

fn ensure_resources() -> &'static GLResources {
    ensure_gl_loaded();

    GL_RESOURCES.get_or_init(|| {
        unsafe {
            // Vertex data: full-screen quad (NDC) with UVs
            //  x,   y,   u,  v
            let vertices: [f32; 16] = [
                -1.0, -1.0, 0.0, 0.0,
                 1.0, -1.0, 1.0, 0.0,
                 1.0,  1.0, 1.0, 1.0,
                -1.0,  1.0, 0.0, 1.0,
            ];
            let indices: [u32; 6] = [0, 1, 2, 2, 3, 0];

            // Shaders
            let vs_src = r#"#version 330 core
layout(location = 0) in vec2 aPos;
layout(location = 1) in vec2 aUV;
out vec2 vUV;
void main() {
    vUV = aUV;
    gl_Position = vec4(aPos, 0.0, 1.0);
}"#;

            let fs_src = r#"#version 330 core
in vec2 vUV;
out vec4 FragColor;
uniform sampler2D uTex;
void main() {
    FragColor = texture(uTex, vUV);
}"#;

            let vs = compile_shader(vs_src, gl::VERTEX_SHADER)
                .expect("Failed to compile vertex shader");
            let fs = compile_shader(fs_src, gl::FRAGMENT_SHADER)
                .expect("Failed to compile fragment shader");
            let program = link_program(vs, fs).expect("Failed to link shader program");

            // Buffers/VAO
            let mut vao: GLuint = 0;
            let mut vbo: GLuint = 0;
            let mut ebo: GLuint = 0;
            gl::GenVertexArrays(1, &mut vao);
            gl::GenBuffers(1, &mut vbo);
            gl::GenBuffers(1, &mut ebo);

            gl::BindVertexArray(vao);

            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (vertices.len() * std::mem::size_of::<f32>()) as isize,
                vertices.as_ptr() as *const c_void,
                gl::STATIC_DRAW,
            );

            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                (indices.len() * std::mem::size_of::<u32>()) as isize,
                indices.as_ptr() as *const c_void,
                gl::STATIC_DRAW,
            );

            // Vertex attributes: location 0 = vec2 position, location 1 = vec2 uv
            let stride = (4 * std::mem::size_of::<f32>()) as i32;
            gl::EnableVertexAttribArray(0);
            gl::VertexAttribPointer(
                0,
                2,
                gl::FLOAT,
                gl::FALSE,
                stride,
                ptr::null(),
            );
            gl::EnableVertexAttribArray(1);
            gl::VertexAttribPointer(
                1,
                2,
                gl::FLOAT,
                gl::FALSE,
                stride,
                (2 * std::mem::size_of::<f32>()) as *const c_void,
            );

            gl::BindVertexArray(0);
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);

            // Uniforms
            gl::UseProgram(program);
            let u_name = CString::new("uTex").unwrap();
            let u_tex_loc = gl::GetUniformLocation(program, u_name.as_ptr());
            if u_tex_loc >= 0 {
                gl::Uniform1i(u_tex_loc, 0); // texture unit 0
            }
            gl::UseProgram(0);

            GLResources {
                program,
                vao,
                vbo,
                ebo,
                u_tex_loc,
            }
        }
    })
}
pub fn draw_texture(texture_id: u32) {
    // Ensure OpenGL is loaded and resources are ready
    let res = ensure_resources();

    unsafe {
        // Minimal state setup for drawing a textured full-screen quad
        gl::Disable(gl::DEPTH_TEST);
        gl::ActiveTexture(gl::TEXTURE0);
        gl::BindTexture(gl::TEXTURE_2D, texture_id);

        gl::UseProgram(res.program);
        if res.u_tex_loc >= 0 {
            gl::Uniform1i(res.u_tex_loc, 0);
        }
        gl::BindVertexArray(res.vao);

        // EBO is bound via VAO state in core profile; ensure it's bound
        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, res.ebo);
        gl::DrawElements(gl::TRIANGLES, 6, gl::UNSIGNED_INT, ptr::null());

        // Optional: unbind to reduce side-effects
        gl::BindVertexArray(0);
        gl::UseProgram(0);
        gl::BindTexture(gl::TEXTURE_2D, 0);
    }
}
use std::os::fd::RawFd;
use gl::types::{GLenum};
use nix::{cmsg_space, libc};
#[derive(Debug)]
pub struct ExternalTexture {
    pub texture: GLuint,
    pub image: egl::types::EGLImageKHR,
    pub fd: RawFd,
    pub width: i32,
    pub height: i32,
}

impl ExternalTexture {
    /// Importa um DMA-BUF (fd) como textura OpenGL
    pub unsafe fn import(fd: RawFd, width: i32, height: i32, fourcc: i32) -> Option<Self> {
        let display = egl::GetCurrentDisplay();
        let context = egl::GetCurrentContext();

        if display == egl::NO_DISPLAY || context == egl::NO_CONTEXT {
            eprintln!("EGL not initialized or no current context");
            return None;
        }

        let attribs = [
            egl::WIDTH as i32, width,
            egl::HEIGHT as i32, height,
            egl::LINUX_DRM_FOURCC_EXT as i32, fourcc,
            egl::DMA_BUF_PLANE0_FD_EXT as i32, fd,
            egl::DMA_BUF_PLANE0_PITCH_EXT as i32, width * 4,
            egl::DMA_BUF_PLANE0_OFFSET_EXT as i32, 0,
            egl::NONE as i32,
        ];

        let image = egl::CreateImageKHR(
            display,
            context,
            egl::LINUX_DMA_BUF_EXT,
            std::ptr::null_mut(),
            attribs.as_ptr(),
        );
        if image == egl::NO_IMAGE_KHR {
            eprintln!("Failed to create EGLImage from DMA-BUF");
            return None;
        }

        let mut tex: GLuint = 0;
        gl::GenTextures(1, &mut tex);
        gl::BindTexture(gl::TEXTURE_2D, tex);
        gl::EGLImageTargetTexture2DOES(gl::TEXTURE_2D as GLenum, image);

        // Configuração básica
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
        gl::BindTexture(gl::TEXTURE_2D, 0);

        Some(Self {
            texture: tex,
            image,
            fd,
            width,
            height,
        })
    }
}

impl Drop for ExternalTexture {
    fn drop(&mut self) {
        unsafe {
            // apagar textura
            if self.texture != 0 {
                gl::DeleteTextures(1, &self.texture);
            }

            // destruir EGLImage
            if self.image != egl::NO_IMAGE_KHR {
                let display = egl::GetCurrentDisplay();
                egl::DestroyImageKHR(display, self.image);
            }

            // fechar o fd
            libc::close(self.fd);
        }
    }
}

fn main() {
    // Initialize GLFW
    let mut glfw = glfw::init(glfw::fail_on_errors).expect("Failed to initialize GLFW");

    // Set up an OpenGL context (not strictly necessary for a basic window, but common)
    glfw.window_hint(glfw::WindowHint::ContextVersion(3, 3));
    glfw.window_hint(glfw::WindowHint::OpenGlProfile(glfw::OpenGlProfileHint::Core));
    glfw.window_hint(glfw::WindowHint::ContextCreationApi(glfw::ContextCreationApi::Egl));
    // Create the window
    let (mut window, events) = glfw
        .create_window(800, 600, "📡 Receiver", glfw::WindowMode::Windowed)
        .expect("Failed to create GLFW window");

    // Make the context current and enable event polling
    window.make_current();
    window.set_key_polling(true);
    window.set_close_polling(true);
    window.set_size_polling(true);
    window.set_framebuffer_size_polling(true);
    window.make_current();

    // 3. Carregar EGL
    unsafe {
    	egl::load_with(|s| glfw.get_proc_address_raw(s).map_or_else(std::ptr::null, |f| f as *const c_void));
    }

    // 4. Carregar GL
    unsafe {
    	gl::load_with(|s| glfw.get_proc_address_raw(s).map_or_else(std::ptr::null, |f| f as *const c_void));
    }
    let listener = match UnixListener::bind("/tmp/opengl-receiver.sock") {
        Ok(sock) => sock,
        Err(e) => {
            println!("Couldn't connect: {e:?}");
            return
        }
    };
    listener.set_nonblocking(true).unwrap();
    let mut client: Option<UnixStream> = None;
    let mut texture: Option<ExternalTexture> = None;
    // Main loop
    while !window.should_close() {
        // Swap buffers and poll for events
        window.swap_buffers();
        glfw.poll_events();

        for (_, event) in glfw::flush_messages(&events) {
            match event {
                WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                    window.set_should_close(true);
                }
                WindowEvent::Close => window.set_should_close(true),
                _ => {}
            }
        }

        // update
        if let Ok((s, _)) = listener.accept() {
        	client = Some(s);
        }
        if let Some(img_fd) = client.as_ref().and_then(receive_fd) {
        	texture = unsafe {ExternalTexture::import(img_fd, 500, 500, 875_713_089)};
         println!("Received texture: {texture:#?}");
        }
        if let Some(tex) = &texture {
        	draw_texture(tex.texture);
        }
    }
}
fn receive_fd(stream: &UnixStream) -> Option<i32> {
    use nix::sys::socket::{recvmsg, ControlMessageOwned, MsgFlags};
    use std::io::IoSliceMut;
    use std::os::fd::{AsRawFd, RawFd};

    let fd = stream.as_raw_fd();

    let mut buf = [0u8; 1];
    let mut iov = [IoSliceMut::new(&mut buf)];
    let mut cmsg_space = cmsg_space!([RawFd; 1]);

    match recvmsg::<()>(fd, &mut iov, Some(&mut cmsg_space), MsgFlags::MSG_DONTWAIT) {
        Ok(msg) => {
            for cmsg in msg.cmsgs().unwrap() {
                if let ControlMessageOwned::ScmRights(fds) = cmsg {
                    if let Some(&received_fd) = fds.first() {
                        return Some(received_fd as i32);
                    }
                }
            }
            None
        }
        Err(_) => None,
    }
}
