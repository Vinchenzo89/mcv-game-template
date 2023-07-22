// Ensures this program runs as a windows subsystem and will not be allocated a console
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::collections::HashMap;
use game::{
    math::*,
    state::*
};

use windows_sys::{
    core::*,
    Win32::Foundation::*,
    Win32::Graphics::Gdi::*,
    Win32::Graphics::OpenGL::*,
    Win32::Media::{timeBeginPeriod, TIMERR_NOCANDO},
    Win32::System::LibraryLoader::*,
    Win32::UI::Input::KeyboardAndMouse::*,
    Win32::UI::WindowsAndMessaging::*,
};

// // The GL bindings
// pub mod gl {
//     include!(concat!(env!("OUT_DIR"), "/gl_bindings.rs"));
// }
// use gl::types::*;

// /// Macro to get c strings from literals without runtime overhead
// /// Literal must not contain any interior nul bytes!
// use std::ffi::CStr;
// macro_rules! c_str {
//     ($literal:expr) => {
//         CStr::from_bytes_with_nul_unchecked(concat!($literal, "\0").as_bytes())
//     }
// }

const GAME_TITLE: &str = "MCV Game Template";

const DEFAULT_SCREEN_WIDTH:  i32 = 1600;
const DEFAULT_SCREEN_HEIGHT: i32 = 900;
static mut GLOBAL_APP_RUNNING: bool = false;

const BITMAP_BYTES_PER_PIXEL: i32 = 4; // RGBA
#[derive(Default)]
pub struct LoadedBitmap {
    pub width: i32,
    pub height: i32,
    pub pitch: i32,
    pub data: Option<Box<[u8]>>,
}

pub const EMPTY_TEXTURE: u32 = 0;
#[derive(Copy, Clone)]
pub struct LoadedGpuTexture {
    pub id: u32
}
impl Default for LoadedGpuTexture {
    fn default() -> Self {
        Self { id: EMPTY_TEXTURE }
    }
}

#[derive(Default)]
pub struct GameAssets {
    pub debug_texture: LoadedGpuTexture,
    pub char_textures: HashMap<char, LoadedGpuTexture>,
}
pub fn assets_set_char_texture(assets: &mut GameAssets, c: char, texture: LoadedGpuTexture) {
    assets.char_textures.insert(c, texture);
}
pub fn assets_get_char_texture(assets: &GameAssets, c: char) -> Option<&LoadedGpuTexture> {
    assets.char_textures.get(&c)
}

const DEFAULT_FONT_GLYPH_BITMAP_DIM: i32 = 512;
pub struct FontOptions {
    pub height_pts: i32,
    pub glyph_bitmap_dim: i32,
}
impl Default for FontOptions {
    fn default() -> Self {
        Self { 
            height_pts: 128, // todo this is in Points and we want to specify in Pixels
            glyph_bitmap_dim: DEFAULT_FONT_GLYPH_BITMAP_DIM
        }
    }
}
#[derive(Default, Copy, Clone)]
pub struct Win32LoadedFont {
    pub hdc: CreatedHDC,
    pub hfont: HFONT,
    pub hbitmap: HBITMAP,
    pub metrics: Option<TEXTMETRICW>,
}
unsafe fn win32_create_font(font_name: &str, options: FontOptions) -> Win32LoadedFont {

    // let success = AddFontResourceExA(font_file.as_ptr(), FR_PRIVATE, 0 as *mut _);
    // debug_assert!(success > 0);

    let height = options.height_pts;
    let font = CreateFontA(
        height,
        0, // Width: Zero means it will be calculated
        0, // Escaptement: Not necesary
        0, // Orientation: Not rotated
        // Report FW_NORMAL should either be an i32 or fix CreateFontA to u32 param 
        FW_NORMAL as i32,  // Font Weight
        0, // Italic
        0, // Underline
        0, // Strikeout
        DEFAULT_CHARSET as u32,        // Charset: For specifying language?
        OUT_DEFAULT_PRECIS as u32,     // Precision
        CLIP_DEFAULT_PRECIS as u32,    // Clip Precision
        ANTIALIASED_QUALITY as u32,    // Quality
        (DEFAULT_PITCH|FF_DONTCARE) as u32,
        font_name.as_ptr() as _,
    );

    let dc: CreatedHDC = CreateCompatibleDC(GetDC(0));

    let mut info: BITMAPINFO = std::mem::zeroed();
    info.bmiHeader.biSize = std::mem::size_of::<BITMAPINFO>() as u32;
    info.bmiHeader.biWidth = DEFAULT_FONT_GLYPH_BITMAP_DIM;
    info.bmiHeader.biHeight = DEFAULT_FONT_GLYPH_BITMAP_DIM;
    info.bmiHeader.biPlanes = 1 as u16;
    info.bmiHeader.biBitCount = 32 as u16;
    info.bmiHeader.biCompression = BI_RGB;
    info.bmiHeader.biSizeImage = (BITMAP_BYTES_PER_PIXEL*DEFAULT_FONT_GLYPH_BITMAP_DIM*DEFAULT_FONT_GLYPH_BITMAP_DIM) as u32;
    info.bmiHeader.biXPelsPerMeter = 0;
    info.bmiHeader.biYPelsPerMeter = 0;
    info.bmiHeader.biClrUsed = 0;
    info.bmiHeader.biClrImportant = 0;

    // will point to the pixels created by CreateDIBSection()
    let mut pixels: *mut std::ffi::c_void = std::ptr::null_mut();
    let bitmap: HBITMAP = CreateDIBSection(dc, &info, DIB_RGB_COLORS, &mut pixels, 0, 0);
    debug_assert!(bitmap != 0);

    SelectObject(dc, bitmap);
    SelectObject(dc, font);

    let mut tm: TEXTMETRICW = std::mem::zeroed();
    GetTextMetricsW(dc, &mut tm);

    Win32LoadedFont {
        hdc: dc,
        hfont: font,
        hbitmap: bitmap,
        metrics: Some(tm),
    }
}

unsafe fn win32_destroy_font(font: Win32LoadedFont) {
    DeleteObject(font.hfont);
    DeleteObject(font.hbitmap);
    DeleteObject(font.hdc);
}

unsafe fn win32_create_font_char_bitmap(font: Win32LoadedFont, character: u16) -> LoadedBitmap {

    let mut size: SIZE = std::mem::zeroed();
    GetTextExtentPoint32W(font.hdc, &character, 1, &mut size);

    let width = size.cx;
    let height = size.cy;

    // Set the background color that will be used
    // when the text is drawn with TextOutW()
    const BLACK: COLORREF = 0x00000000;
    SetBkColor(font.hdc, BLACK);

    // Now actually render the character text to the bitmap
    const WHITE: COLORREF = 0x00FFFFFF;
    SetTextColor(font.hdc, WHITE);
    TextOutW(font.hdc, 0, 0, &character, 1);

    let mut result = LoadedBitmap::default();
    result.width  = width;
    result.height = height;
    result.pitch  = width * BITMAP_BYTES_PER_PIXEL;

    // create the bitmap data
    let total_bytes = height.checked_mul(result.pitch).expect("Font Bitmap bytes exceeds i32");
    let total_bytes = total_bytes as usize;
    let mut data = Vec::<u8>::with_capacity(total_bytes);

    // get the bitmap data from the OS
    for y in 0..height {
        for x in 0..width {
            // OS writes the bitmap with the y flipped
            let flipped_y = height - 1 - y;

            let pixel = GetPixel(font.hdc, x, flipped_y);

            // the text color White should be set for all channels.
            // here we are grabing the alpha channel ARGB 
            let color = (pixel & 0xFF) as u8;
            let alpha = if color > 0 { 0xFF } else { 0x00 };

            // push this value for all 4 channels
            data.push(color); // R
            data.push(color); // G
            data.push(color); // B
            data.push(alpha); // A
        }
    }

    debug_assert!(data.len() == total_bytes, "bitmap len {} != total bytes {}", data.len(), total_bytes);

    // set the data
    result.data = Some(data.into_boxed_slice());

    result
}

// Enables the game code to be hot reloaded while developing
#[cfg(feature = "hotreload")]
mod hotreload {
    use super::*;

    #[derive(Copy, Clone)]
    pub struct GameApi {
        pub game_dll_handle: HINSTANCE,
        pub update_and_render: game::GameUpdateAndRenderFunc,
    }

    pub struct GameWatchAndCopy {
        pub file_watch_path: &'static str,
        pub file_copy_path: &'static str,
        pub last_modified: std::time::SystemTime,
    }

    pub unsafe fn win32_load_game_code(dll_path: &'static str) -> GameApi { 

        let game_dll_path = std::ffi::CString::new(dll_path).expect("CString::new() failed");
        let game_dll_handle = LoadLibraryA(game_dll_path.as_ptr() as *const _);
        debug_assert!(game_dll_handle != 0, "Load game.dll error {}", GetLastError());

        let game_update_name = std::ffi::CString::new("update_and_render").expect("CString::new() failed");
        let game_update_func: FARPROC = GetProcAddress(game_dll_handle, game_update_name.as_ptr() as *const _);
        debug_assert!(game_update_func.is_some(), "Get game::update() address error {}", GetLastError());
        
        let game_update_and_render = game_update_func.unwrap();
        let game_update_and_render: game::GameUpdateAndRenderFunc = std::mem::transmute(game_update_and_render);

        GameApi {
            game_dll_handle,
            update_and_render: game_update_and_render,
        }
    }

    pub fn win32_get_file_modified_time(file_path: &str) -> Result<std::time::SystemTime, std::io::Error> {
        let metadata = std::fs::metadata(file_path)?;
        let last_modified = metadata.modified()?;
        Ok(last_modified)
    }

    pub unsafe fn win32_reload_game_code(watch_file: &mut GameWatchAndCopy, gc: Option<GameApi>) -> Option<GameApi> {
        // The file watching comes first
        let modified = win32_file_modified_get(watch_file.file_watch_path).unwrap();
        let duration = modified.duration_since(watch_file.last_modified);
        let reload_code = match duration {
            Ok(d) => !d.is_zero() || gc.is_none(),
            // error means last_modified > modified which doesn't matter because
            // what we care about is whether the times are different
            Err(_) => gc.is_none(), 
        };

        if reload_code {
            println!("Reloading {} ...", watch_file.file_watch_path);
            watch_file.last_modified = modified;

            // we have to free the last version of the library we loaded
            match gc {
                None => {},
                Some(code) => {
                    // need to free the loaded library so it releases the file
                    let freed = FreeLibrary(code.game_dll_handle);
                    debug_assert!(freed == TRUE, "FreeLibrary() error");
                }
            }

            // now copy the latest .dll to our special named .dll we will load
            // NOTE rustc holds onto the newly built .dll while writing to it preventing the copy from access
            // Not sure if this is great but making multiple attempts until it is released seems to work
            let mut copy_attempts: i32 = 100;
            while copy_attempts > 0 {
                match std::fs::copy(watch_file.file_watch_path, watch_file.file_copy_path) {
                    Ok(bytes) => break,
                    Err(e) => {
                        --copy_attempts;
                    },
                }
            }

            // now load the copied version of the .dll
            let new_api = win32_load_game_code(watch_file.file_copy_path);

            return Some(new_api);
        };

        gc
    }
}

unsafe extern "system" fn window_proc_callback(
    window_handle: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM) -> LRESULT {
    let mut result = 0;

    match message {
        WM_CLOSE | WM_QUIT => {
            GLOBAL_APP_RUNNING = false;
        },
        _ => {
            result = DefWindowProcA(window_handle, message, wparam, lparam);
        }
    }
    
    result
}

fn main() {

    #[cfg(feature = "hotreload")]
    use hotreload::*;
    #[cfg(feature = "hotreload")]
    let (mut watch_file, mut game_api) = {
        let api: Option<GameApi> = None;
        let watch = GameWatchAndCopy {
            file_watch_path: "target\\debug\\game.dll",
            file_copy_path: "target\\debug\\win32_game.dll",
            last_modified: std::time::SystemTime::now(),
        };
        (watch, api)
    };

    // Set the system counter granularity to 1ms so counters like Sleep() can be as granular as 1ms.
    let sleep_is_granular = unsafe { timeBeginPeriod(1) != TIMERR_NOCANDO };

    let hinstance :HINSTANCE = unsafe { GetModuleHandleA(0 as PCSTR) };
    
    let window_class = WNDCLASSA {
        style: CS_OWNDC|CS_HREDRAW|CS_VREDRAW,
        lpfnWndProc: Some(window_proc_callback),
        cbClsExtra: 0,
        cbWndExtra: 0,
        hInstance: hinstance,
        hIcon: 0,
        hCursor: 0,
        hbrBackground: 0,
        lpszMenuName: 0 as *const u8,
        lpszClassName: b"Game_WindowClass\0".as_ptr(),
    };
    
    let window_result: Result<HWND, &str> = unsafe {
        match RegisterClassA(&window_class) {
            0 => Err("Error registering window class"),
            _ => {
                let handle = CreateWindowExA(
                    0,
                    window_class.lpszClassName,
                    GAME_TITLE.as_ptr(), 
                    WS_OVERLAPPEDWINDOW|WS_VISIBLE, 
                    CW_USEDEFAULT, CW_USEDEFAULT,
                    DEFAULT_SCREEN_WIDTH, 
                    DEFAULT_SCREEN_HEIGHT,
                    0,
                    0,
                    hinstance, 
                    0 as *const ::core::ffi::c_void);

                match handle {
                    0 => Err("Error creating window"),
                    h => Ok(h)
                }
            }
        }
    };

    // NOTE Panics and aborts the process
    let window_handle :HWND = match window_result {
        Err(e) => std::panic!("An error occurred: {}",e),
        Ok(h) => h,
    };

    // Initialize OpenGL functions
    unsafe {
        let window_hdc :HDC = GetDC(window_handle); // Window Device Context

        // configure pixel format
        let pixel_format_size = std::mem::size_of::<PIXELFORMATDESCRIPTOR>() as u16;
        let mut pixel_format :PIXELFORMATDESCRIPTOR = std::mem::zeroed();
        pixel_format.nSize = pixel_format_size;     //  size of this pfd  
        pixel_format.nVersion = 1;                  // version number  
        pixel_format.dwFlags =  PFD_DRAW_TO_WINDOW  // support window  
                               |PFD_SUPPORT_OPENGL  // support OpenGL  
                               |PFD_DOUBLEBUFFER;   // double buffered  
        pixel_format.iPixelType = PFD_TYPE_RGBA;    // RGBA type  
        pixel_format.cColorBits = 24;               // 24-bit color depth
        pixel_format.cAlphaBits = 8;                // 32-bit z-buffer
        pixel_format.iLayerType = PFD_MAIN_PLANE;   // main layer  

        let suggested_pixel_format_index = ChoosePixelFormat(window_hdc, &pixel_format);
        let mut suggested_pixel_format :PIXELFORMATDESCRIPTOR = std::mem::zeroed();
        DescribePixelFormat(window_hdc, 
                            suggested_pixel_format_index,
                            pixel_format_size as u32, 
                            &mut suggested_pixel_format);
        if SetPixelFormat(window_hdc, suggested_pixel_format_index, &suggested_pixel_format) == 0 {
            std::panic!("SetPixelFormat failed");
        }

        let window_hglrc  :HGLRC = wglCreateContext(window_hdc); // Window Render Context 
        if wglMakeCurrent(window_hdc, window_hglrc) > 0 {
            // todo come back and use modern Open GL features
            // gl::load_with(|fn_name| {
            //     // println!("Loading fn: {}", fn_name);
            //     let n = format!("{}\0", fn_name).as_ptr();
            //     match wglGetProcAddress(n) {
            //         None => {
            //             println!("Error occurred loading GL function: {}", fn_name);
            //             0 as *const _
            //         },
            //         Some(fn_ptr) => fn_ptr as *const _ ,
            //     }
            // });
        }

        ReleaseDC(window_handle, window_hdc);
    }

    
    let mut ctx = GameState::default();
    let mut input = GameInput::default();
    let mut assets = GameAssets::default();

    let debug_bitmap = {
        let mut result = LoadedBitmap::default();
        result.width = 256;
        result.height = 256;
        result.pitch = result.width * BITMAP_BYTES_PER_PIXEL;
        
        let mut data = Vec::<u8>::with_capacity((result.height * result.pitch) as usize);
        for y in 0..result.height {
            for x in 0..result.width {
                data.push(y as u8); // R?
                data.push(0); // G?
                data.push(x as u8); // B?
                data.push(255); // Alpha?
            }
        }
        
        result.data = Some(data.into_boxed_slice());
        
        result
    };
    
    assets.debug_texture = unsafe { 
        win32_opengl_texture_create(
            debug_bitmap.width,
            debug_bitmap.height,
            debug_bitmap.data.as_ref().unwrap(),
        ) 
    };
    
    unsafe {
        let arial_font = win32_create_font("Arial\0", FontOptions::default());
        
        for c in GAME_TITLE.chars() {
            if c == '\0' { break; }
            let char_bitmap = win32_create_font_char_bitmap(arial_font, c as u16);
            let texture = win32_opengl_texture_create(
                char_bitmap.width, 
                char_bitmap.height, 
                char_bitmap.data.as_ref().unwrap());
            assets_set_char_texture(&mut assets, c as char, texture);
        }

        win32_destroy_font(arial_font);
    };
    
    // Target frame rate stuff
    let target_frames_per_sec: f32 = 30.0;
    let target_frame_rate_ms: f32 = 1000.0/target_frames_per_sec;
    
    // Initialize window stuff
    unsafe { GLOBAL_APP_RUNNING = true; }
    while unsafe { GLOBAL_APP_RUNNING } {
        let work_timer = std::time::Instant::now();

        // should we always force the game step is the target vs actual ellapsed
        input.frame_dt_sec = target_frames_per_sec.recip();
        
        // Peek window messages
        unsafe {
            let mut msg: MSG = std::mem::zeroed();
            while PeekMessageA(&mut msg, 0, 0, 0, PM_REMOVE) != 0 {
                match msg.message {
                    WM_KEYUP|
                    WM_KEYDOWN|
                    WM_SYSKEYUP|
                    WM_SYSKEYDOWN => {
                        #[allow(non_snake_case)]
                        let KeyIsDownBitFlag = 1 << 31;
                        #[allow(non_snake_case)]
                        let KeyWasDownBitFlag = 1 << 30;
                        
                        let is_down   = (KeyIsDownBitFlag  & msg.lParam) == 0;
                        let _was_down  = (KeyWasDownBitFlag & msg.lParam) != 0;
                        
                        if VK_SPACE as usize == msg.wParam {
                            input.launch_down = is_down;
                        }
                        else if 'A' as usize == msg.wParam {
                            input.turn_left = is_down;
                        }
                        else if 'D' as usize == msg.wParam {
                            input.turn_right = is_down;
                        }
                        else if 'W' as usize == msg.wParam {
                            input.accelerate = is_down;
                        }
                        else if 'S' as usize == msg.wParam {
                            input.decelerate = is_down;
                        }
                    },
                    _ => {},
                }
                TranslateMessage(&msg);
                DispatchMessageA(&msg);
            }
        }

        let (screen_width, screen_height) = unsafe {
            let mut window_rect: RECT = std::mem::zeroed();
            GetWindowRect(window_handle, &mut window_rect);
            (window_rect.right - window_rect.left,
             window_rect.bottom - window_rect.top)
        };
        input.screen_width = screen_width;
        input.screen_height = screen_height;

        #[cfg(feature = "hotreload")]
        {
            game_api = unsafe { win32_reload_game_code(&mut watch_file, game_api) };
            if let Some(api) = game_api {
                (api.update_and_render)(&input, &mut ctx);
            }
        }
        #[cfg(not(feature = "hotreload"))]
        game::update_and_render(&input, &mut ctx);

        // Render main game
        win32_opengl_render(0, 0, screen_width, screen_height, &assets, &ctx);

        // Render mini map
        // win32_opengl_render(20, 20, 200, 200, &assets, &ctx);

        // Enforce the frame rate
        let mut time_ellapsed_ms = work_timer.elapsed().as_secs_f32() * 1000.0;
        while time_ellapsed_ms < target_frame_rate_ms {
            if sleep_is_granular {
                let remaining_ms = (target_frame_rate_ms - time_ellapsed_ms) as u64;
                std::thread::sleep(std::time::Duration::from_millis(remaining_ms));
            }
            time_ellapsed_ms = work_timer.elapsed().as_secs_f32() * 1000.0;
        }

        unsafe {
            let window_hdc = GetDC(window_handle);
            SwapBuffers(window_hdc);
            ReleaseDC(window_handle, window_hdc);
        }
    }
}

fn win32_opengl_render(x: i32, y: i32, width: i32, height: i32, assets: &GameAssets, ctx: &GameState) { 
    unsafe {
        glViewport(x, y, width, height);
        glClearColor(0.0, 0.05, 0.11, 1.0);
        glClear(GL_COLOR_BUFFER_BIT);

        // Projection
        glMatrixMode(GL_PROJECTION);
        glLoadIdentity();
        let screen_half_width = width / 2;
        let screen_half_height = height / 2;
        glOrtho(-screen_half_width as f64,  screen_half_width as f64, 
                -screen_half_height as f64, screen_half_height as f64, 
                0.0, 100.0);

        // will this work
        let player_pos = ctx.player.pos;
        glMultMatrixf([
            1.0, 0.0, 0.0, 0.0,
            0.0, 1.0, 0.0, 0.0,
            0.0, 0.0, 1.0, 0.0,
            -player_pos.x, -player_pos.y, 0.0, 1.0,
        ].as_ptr());

        // Start Model View spaces
        glMatrixMode(GL_MODELVIEW);
        glLoadIdentity(); // Reset the ModelView matrix to identity matrix
        
        // Render space background
        glPushMatrix();
        glColor3f(1.0, 1.0, 1.0);
        glBegin(GL_POINTS);
        let stars = ctx.space_stars.as_ref().unwrap();
        for s in stars  {
            glVertex2f(s.pos.x, s.pos.y);
        }
        glEnd();
        glPopMatrix();

        // Render the world map
        // TODO not ready for world tile map yet until we want to scale the size of game
        // let tile_pixel_dim :i32 = 64;
        // let tile_map_cols = (width / tile_pixel_dim) + 1;
        // let tile_map_rows = (height / tile_pixel_dim) + 1;
        // glColor3f(0.5, 0.5, 0.5);
        // glPolygonMode(GL_FRONT_AND_BACK, GL_LINE);
        // for row in 0..tile_map_rows {
        //     for col in 0..tile_map_cols {
        //         let tx = col * tile_pixel_dim;
        //         let ty = row * tile_pixel_dim;
        //         let tx = tx + tile_pixel_dim / 2;
        //         let ty = ty + tile_pixel_dim / 2;
        //         let dim = tile_pixel_dim;
        //         let mv = [
        //             dim as f32, 0.0, 0.0, 0.0,
        //             0.0, dim as f32, 0.0, 0.0,
        //             0.0, 0.0, 1.0, 0.0,
        //             tx as f32,  ty as f32, 0.0, 1.0,
        //         ];
        //         glPushMatrix();
        //         glMultMatrixf(mv.as_ptr());
        //         glBegin(GL_QUADS);
        //         glVertex2f(-1.0,  1.0);
        //         glVertex2f(-1.0, -1.0);
        //         glVertex2f( 1.0, -1.0);
        //         glVertex2f( 1.0,  1.0);
        //         glEnd();
        //         glPopMatrix();
        //     }
        // }
        // glPolygonMode(GL_FRONT_AND_BACK, GL_FILL);

        // Render planets
        let circle_points = 360;
        let circle_point_angle_step_radians: f32 = (360.0/circle_points as f32).to_radians();
        let planets = ctx.planets.as_ref().unwrap();
        for planet in planets {
            glColor3f(planet.color.x, planet.color.y, planet.color.z);
            let mv = [
                planet.radius, 0.0, 0.0, 0.0,
                0.0, planet.radius, 0.0, 0.0,
                0.0, 0.0, 1.0, 0.0,
                planet.pos.x, planet.pos.y, 0.0, 1.0,
            ];
            glPushMatrix();
            glMultMatrixf(mv.as_ptr());
            glBegin(GL_TRIANGLE_FAN);
            for p in 0..circle_points {
                let px = (p as f32 * circle_point_angle_step_radians).cos();
                let py = (p as f32 * circle_point_angle_step_radians).sin();
                glVertex2f(px, py);
            }
            glEnd();
            glPopMatrix();

            glColor3f(planet.color.x, planet.color.y, planet.color.z);
            let mv = [
                planet.g_radius, 0.0, 0.0, 0.0,
                0.0, planet.g_radius, 0.0, 0.0,
                0.0, 0.0, 1.0, 0.0,
                planet.pos.x, planet.pos.y, 0.0, 1.0,
            ];
            glPushMatrix();
            glMultMatrixf(mv.as_ptr());
            glBegin(GL_LINE_STRIP);
            for p in 0..circle_points {
                let px = (p as f32 * circle_point_angle_step_radians).cos();
                let py = (p as f32 * circle_point_angle_step_radians).sin();
                glVertex2f(px, py);
            }
            glEnd();
            glPopMatrix();
            
            // Render planet stuff
            let fuel_pos = vector_2f_add(planet.pos, planet.item.pos);
            let fuel_color = vector_4f(0.1, 0.2, 1.0, 1.0);
            let fuel_dim = vector_2f(10.0, 10.0);
            glColor3f(fuel_color.x, fuel_color.y, fuel_color.z);
            let mv = [
                fuel_dim.x, 0.0, 0.0, 0.0,
                0.0, fuel_dim.y, 0.0, 0.0,
                0.0, 0.0, 1.0, 0.0,
                fuel_pos.x, fuel_pos.y, 0.0, 1.0,
            ];
            glPushMatrix();
            glMultMatrixf(mv.as_ptr());
            glBegin(GL_QUADS);
            glVertex2f(-1.0,  1.0);
            glVertex2f(-1.0, -1.0);
            glVertex2f( 1.0, -1.0);
            glVertex2f( 1.0,  1.0);
            glEnd();
            glPopMatrix();

            // Render Landing Zone
            let lz_pos = vector_2f_add(planet.pos, planet.lz_rel_pos);
            let lz_dim = vector_2f(10.0, 10.0);
            let lz_color = planet.lz_color;
            glColor3f(lz_color.x, lz_color.y, lz_color.z);
            let mv = [
                lz_dim.x, 0.0, 0.0, 0.0,
                0.0, lz_dim.y, 0.0, 0.0,
                0.0, 0.0, 1.0, 0.0,
                lz_pos.x, lz_pos.y, 0.0, 1.0,
            ];
            glPushMatrix();
            glMultMatrixf(mv.as_ptr());
            glBegin(GL_QUADS);
            glVertex2f(-1.0,  1.0);
            glVertex2f(-1.0, -1.0);
            glVertex2f( 1.0, -1.0);
            glVertex2f( 1.0,  1.0);
            glEnd();
            glPopMatrix();
        }

        // Render Player
        glColor3f(1.0, 1.0, 1.0);
        let s = vector_2f(10.0, 10.0);  // scale
        let t = &ctx.player.pos;        // translate
        let r = *&ctx.player.rot;       // rotation
        let mv = [
            s.x, 0.0, 0.0, 0.0,
            0.0, s.y, 0.0, 0.0,
            0.0, 0.0, 1.0, 0.0,
            t.x, t.y, 0.0, 1.0,
        ];
        glPushMatrix();
        glMultMatrixf(mv.as_ptr());
        glMultMatrixf([
            r.cos(),  r.sin(), 0.0, 0.0,
           -r.sin(),  r.cos(), 0.0, 0.0,
            0.0,      0.0,     1.0, 0.0,
            0.0,      0.0,     0.0, 1.0,
        ].as_ptr());
        glBegin(GL_TRIANGLES);
        glVertex2f( 1.0,  0.0);
        glVertex2f(-1.0,  1.0);
        glVertex2f(-0.5,  0.0);   

        glVertex2f( 1.0,  0.0);
        glVertex2f(-0.5,  0.0);
        glVertex2f(-1.0, -1.0);
        glEnd();
        glPopMatrix();

        // Render Navigation Path
        if let Some(path) = &ctx.nav_path {
            glEnable(GL_LINE_SMOOTH);
            glLineWidth(2.0);
            glBegin(GL_LINES);
            for p in path.points.iter() {
                glVertex2f(p.p.x, p.p.y);
                glColor3f(p.c.x, p.c.y, p.c.z);
            }
            glEnd();
            glLineWidth(1.0);
            glDisable(GL_LINE_SMOOTH);
        }
        
        // Render Forces
        if let Some(forces) = &ctx.debug_player_forces {
            let player_pos = ctx.player.pos;
            glColor3f(1.0, 1.0, 0.0);
            // glLoadIdentity();
            glEnable(GL_LINE_SMOOTH);
            glLineWidth(2.0);
            glBegin(GL_LINES);
            for f in forces {
                let force = vector_2f_add(player_pos, vector_2f_scale(*f, 100.0));
                glVertex2f(player_pos.x, player_pos.y);
                glVertex2f(force.x, force.y);
            }
            glEnd();
            glLineWidth(1.0);
            glDisable(GL_LINE_SMOOTH);
        }

        // Render Text as quads
        glLoadIdentity();
        glEnable(GL_BLEND);
        glBlendFunc(GL_SRC_ALPHA, GL_ONE_MINUS_SRC_ALPHA); 
        // todo use actual font metrics to properly size and position sentences
        let char_scale = vector_2f(40.0, 40.0);
        let text_line_width = GAME_TITLE.len() as f32 * char_scale.x;
        let mut text_char_pos = vector_2f(-text_line_width, 0.0);
        for c in GAME_TITLE.chars() {
            let texture = assets.char_textures.get(&c).expect("Missing char texture");
            let s = char_scale;
            let t = text_char_pos;
            text_char_pos.x += char_scale.x * 2.0;
            let mv = [
                s.x, 0.0, 0.0, 0.0,
                0.0, s.y, 0.0, 0.0,
                0.0, 0.0, 1.0, 0.0,
                t.x, t.y, 0.0, 1.0,
            ];
            glColor4f(1.0, 1.0, 0.0, 1.0 - ctx.title_fade);
            glLoadMatrixf(mv.as_ptr());
            glEnable(GL_TEXTURE_2D);
            glBindTexture(GL_TEXTURE_2D, texture.id);
            glBegin(GL_QUADS);
            glTexCoord2f(0.0, 1.0);
            glVertex2f(-1.0,  1.0);
            glTexCoord2f(0.0, 0.0);
            glVertex2f(-1.0, -1.0);
            glTexCoord2f(1.0, 0.0);
            glVertex2f( 1.0, -1.0);
            glTexCoord2f(1.0, 1.0);
            glVertex2f( 1.0,  1.0);
            glEnd();
            glBindTexture(GL_TEXTURE_2D, 0);
            glDisable(GL_TEXTURE_2D);
        }
        glDisable(GL_BLEND);

        // Render Text character as a quad
        // glLoadIdentity();
        // let texture = assets.char_b_texture.id;
        // let s = vector_2f(50.0, 50.0);
        // let t = vector_2f(200.0, 200.0);
        // let mv = [
        //     s.x, 0.0, 0.0, 0.0,
        //     0.0, s.y, 0.0, 0.0,
        //     0.0, 0.0, 1.0, 0.0,
        //     t.x, t.y, 0.0, 1.0,
        // ];
        // glLoadMatrixf(mv.as_ptr());
        // glEnable(GL_TEXTURE_2D);
        // glBindTexture(GL_TEXTURE_2D, texture);
        // glBegin(GL_QUADS);
        // glTexCoord2f(0.0, 1.0);
        // glVertex2f(-1.0,  1.0);
        // glTexCoord2f(0.0, 0.0);
        // glVertex2f(-1.0, -1.0);
        // glTexCoord2f(1.0, 0.0);
        // glVertex2f( 1.0, -1.0);
        // glTexCoord2f(1.0, 1.0);
        // glVertex2f( 1.0,  1.0);
        // glEnd();
        // glBindTexture(GL_TEXTURE_2D, 0);
        // glDisable(GL_TEXTURE_2D);

        // Render test bitmap to quad
        // glLoadIdentity();
        // let s = vector_2f(100.0, 100.0);
        // let t = vector_2f(-400.0, -200.0);
        // let mv = [
        //     s.x, 0.0, 0.0, 0.0,
        //     0.0, s.y, 0.0, 0.0,
        //     0.0, 0.0, 1.0, 0.0,
        //     t.x, t.y, 0.0, 1.0,
        // ];
        // glLoadMatrixf(mv.as_ptr());
        // glEnable(GL_TEXTURE_2D);
        // glBindTexture(GL_TEXTURE_2D, assets.debug_texture.id);
        // glBegin(GL_QUADS);
        // glTexCoord2f(0.0, 1.0);
        // glVertex2f(-1.0,  1.0);
        // glTexCoord2f(0.0, 0.0);
        // glVertex2f(-1.0, -1.0);
        // glTexCoord2f(1.0, 0.0);
        // glVertex2f( 1.0, -1.0);
        // glTexCoord2f(1.0, 1.0);
        // glVertex2f( 1.0,  1.0);
        // glEnd();
        // glBindTexture(GL_TEXTURE_2D, 0);
        // glDisable(GL_TEXTURE_2D);

        // Center the world on the player

        //
        // Render HUD
        //

        // HUD Projection
        glMatrixMode(GL_PROJECTION);
        glLoadIdentity();
        let screen_half_width = width / 2;
        let screen_half_height = height / 2;
        glOrtho(-screen_half_width as f64,  screen_half_width as f64, 
                -screen_half_height as f64, screen_half_height as f64, 
                0.0, 100.0);
        // Start Model View spaces
        glMatrixMode(GL_MODELVIEW);
        glLoadIdentity(); // Reset the ModelView matrix to identity matrix

        // Render fuel bar
        let rect_padding = 10.0;
        let rect_width = screen_half_width as f32 - (2.0 * rect_padding);
        let rect_height = 10.0;
        let rect_center = (
            0.0, -(screen_half_height as f32 - (2.0 * rect_padding) - rect_height/2.0)
        );
        let rect_width = rect_width * ctx.ship.fuel_level;
        glColor3f(0.0, 0.0, 1.0);
        glLoadIdentity();
        glTranslatef(rect_center.0, rect_center.1, 0.0);
        glScalef(rect_width, rect_height, 1.0);
        glBegin(GL_QUADS);
        glVertex2f(-1.0,  1.0);
        glVertex2f(-1.0, -1.0);
        glVertex2f( 1.0, -1.0);
        glVertex2f( 1.0,  1.0);
        glEnd();
    }
}

pub unsafe fn win32_opengl_texture_create<T>(width: i32, height: i32, data: &[T]) -> LoadedGpuTexture {
    let mut texture_handle: u32 = 0;
    glGenTextures(1, &mut texture_handle);
    glBindTexture(GL_TEXTURE_2D, texture_handle);

    // texture paramters
    // From this post: https://stackoverflow.com/questions/56823126/how-is-gl-clamp-in-opengl-different-from-gl-clamp-to-edge
    /*
    GL normally clamps such that the texture coordinates are limited to exactly the range [0;1]. 
    When a texture coordinate is clamped using this algorithm, the texture sampling filter straddles the edge of the texture image, 
    taking half its sample values from within the texture image, and the other half from the texture border. 
    It is sometimes desirable to clamp a texture without requiring a border, and without using the constant border color.

    A new texture clamping algorithm, CLAMP_TO_EDGE, clamps texture coordinates at all mipmap levels such that the texture filter never samples a border 
    texel. The color returned when clamping is derived only from texels at the edge of the texture image.

    The CLAMP option was deprecated in spec 3.0 2008 and removed in spec 3.2 Core 2009. You can still use it if (and only if) you don't set a Core Profile context.
     */
    // todo Come back and use modern Spec features
    // glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_S,     gl::CLAMP_TO_EDGE as i32);
    // glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_T,     gl::CLAMP_TO_EDGE as i32);
    // NOTE Using the GL 1.1 Windows spec for testing
    glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_S,     GL_CLAMP as i32);
    glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_T,     GL_CLAMP as i32);
    glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, GL_LINEAR as i32);
    glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, GL_LINEAR as i32);

    // FIXME : Make these be passed in as options to describe the texture
    let internal_format = GL_RGBA as i32;
    let pixel_format = GL_RGBA;
    let pixel_component_type = GL_UNSIGNED_BYTE;

    // Report glTexImage2D internal format param is i32 but GL_RGBA, ect. constants are u32
    glTexImage2D(GL_TEXTURE_2D, 0, internal_format, width, height, 0, pixel_format, pixel_component_type, data.as_ptr() as *const _);

    // todo Come back and use modern Spec features. Plus make this a setting?
    // gl::GenerateMipmap(GL_TEXTURE_2D);

    LoadedGpuTexture {
        id: texture_handle
    }
}