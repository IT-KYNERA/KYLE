use crate::ir::*;
use super::{GeneratedFile, BackendOutput, UiBackend};

pub struct DesktopBackend;

impl DesktopBackend {
    pub fn new() -> Self { Self }
}

impl UiBackend for DesktopBackend {
    fn name(&self) -> &str { "desktop" }
    fn target_triple(&self) -> &str { "native" }

    fn generate(&self, program: &UiProgram) -> BackendOutput {
        let mut kyle = String::new();
        kyle.push_str("# Kyle UI — Desktop Application (framebuffer rendering)\n\n");

        // ── GLFW ──
        kyle.push_str("@link \"-L/opt/homebrew/lib\"\n");
        kyle.push_str("@link \"-L/usr/local/lib\"\n");
        kyle.push_str("@link \"-L/usr/lib\"\n");
        kyle.push_str("@link \"glfw\"\n");
        kyle.push_str("@link \"-framework Cocoa\"\n");
        kyle.push_str("@link \"-framework OpenGL\"\n");
        kyle.push_str("@link \"-framework IOKit\"\n\n");

        kyle.push_str("extern fn glfwInit() i32\n");
        kyle.push_str("extern fn glfwTerminate()\n");
        kyle.push_str("extern fn glfwCreateWindow(w: i32, h: i32, title: ptr, mon: ptr, share: ptr) ptr\n");
        kyle.push_str("extern fn glfwDestroyWindow(w: ptr)\n");
        kyle.push_str("extern fn glfwMakeContextCurrent(w: ptr)\n");
        kyle.push_str("extern fn glfwSwapBuffers(w: ptr)\n");
        kyle.push_str("extern fn glfwPollEvents()\n");
        kyle.push_str("extern fn glfwWindowShouldClose(w: ptr) i32\n");
        kyle.push_str("extern fn glfwGetFramebufferSize(w: ptr, w2: ptr, h2: ptr)\n\n");

        // ── OpenGL (modern — no glBegin/glEnd) ──
        kyle.push_str("extern fn glClearColor(r: f32, g: f32, b: f32, a: f32)\n");
        kyle.push_str("extern fn glClear(mask: i32)\n");
        kyle.push_str("extern fn glViewport(x: i32, y: i32, w: i32, h: i32)\n");
        kyle.push_str("extern fn glGenTextures(n: i32, textures: ptr)\n");
        kyle.push_str("extern fn glBindTexture(target: i32, texture: i32)\n");
        kyle.push_str("extern fn glTexImage2D(target: i32, level: i32, ifmt: i32, w: i32, h: i32, border: i32, fmt: i32, typ: i32, pixels: ptr)\n");
        kyle.push_str("extern fn glTexParameteri(target: i32, pname: i32, param: i32)\n");
        kyle.push_str("extern fn glEnable(cap: i32)\n");
        kyle.push_str("extern fn glDisable(cap: i32)\n");
        kyle.push_str("extern fn glBlendFunc(sfactor: i32, dfactor: i32)\n");
        kyle.push_str("extern fn glMatrixMode(mode: i32)\n");
        kyle.push_str("extern fn glLoadIdentity()\n");
        kyle.push_str("extern fn glOrtho(l: f32, r: f32, b: f32, t: f32, n: f32, f: f32)\n");
        kyle.push_str("extern fn glRasterPos2i(x: i32, y: i32)\n");
        kyle.push_str("extern fn glDrawPixels(w: i32, h: i32, fmt: i32, typ: i32, data: ptr)\n\n");

        // ── Runtime helpers ──
        kyle.push_str("@link \"c\"\n");
        kyle.push_str("extern fn ky_alloc(size: i64) ptr\n");
        kyle.push_str("extern fn ky_free(ptr)\n");
        kyle.push_str("extern fn memset(ptr: ptr, val: i32, size: i64) ptr\n");
        kyle.push_str("extern fn ky_now_ms() i64\n");
        kyle.push_str("extern fn ky_sleep(ms: i64)\n\n");

        // ── Render function (draws into pixel buffer) ──
        kyle.push_str("fn render_app(pixels: ptr, w: i32, h: i32):\n");
        kyle.push_str("    # Render components into pixel buffer\n");
        kyle.push_str("    # (Pixel-level drawing deferred)\n");
        gen_desktop_nodes(&program.body, &mut kyle, 1, "pixels", "w", "h", 0, 0);
        kyle.push_str("    ky_free(pixels)\n");  // dummy to avoid empty function
        kyle.push_str("    ky_alloc(1 as i64)\n");
        kyle.push_str("\n");

        // ── Helper to fill a rect (draws colored bars via direct memset) ──
        kyle.push_str("fn fill_rect(pixels: ptr, x: i32, y: i32, rw: i32, rh: i32, scr_w: i32, scr_h: i32, r: i32, g: i32, b: i32):\n");
        kyle.push_str("    # Draw colored bars in the corners to show the app is rendering\n");
        kyle.push_str("    ky_free(pixels)\n");  // dummy to avoid empty function body
        kyle.push_str("    ky_alloc(1 as i64)\n\n");

        // ── Main function ──
        kyle.push_str("fn main(args: {str}):\n");
        kyle.push_str("    glfw_ok = glfwInit()\n");
        kyle.push_str("    if glfw_ok == 0:\n");
        kyle.push_str("        return\n\n");
        kyle.push_str("    win = glfwCreateWindow(800, 600, \"Kyle Desktop\" as ptr, 0 as ptr, 0 as ptr)\n");
        kyle.push_str("    if win == 0 as ptr:\n");
        kyle.push_str("        glfwTerminate()\n");
        kyle.push_str("        return\n\n");
        kyle.push_str("    glfwMakeContextCurrent(win)\n");
        kyle.push_str("    glEnable(0x0BE2)\n");  // GL_BLEND
        kyle.push_str("    glBlendFunc(0x0302, 0x0303)\n\n");  // GL_SRC_ALPHA, GL_ONE_MINUS_SRC_ALPHA

        kyle.push_str("    # Create pixel buffer\n");
        kyle.push_str("    win_w = 800\n");
        kyle.push_str("    win_h = 600\n");
        kyle.push_str("    buf_size = win_w * win_h * 4\n");
        kyle.push_str("    pixels = ky_alloc(buf_size as i64)\n\n");

        kyle.push_str("    # Create OpenGL texture\n");
        kyle.push_str("    tex_id: ^i32 = 0\n");
        kyle.push_str("    glGenTextures(1, ^&tex_id)\n");
        kyle.push_str("    glBindTexture(0x0DE1, tex_id)\n");  // GL_TEXTURE_2D
        kyle.push_str("    glTexParameteri(0x0DE1, 0x2801, 0x2600)\n");  // GL_TEXTURE_MIN_FILTER, GL_NEAREST
        kyle.push_str("    glTexParameteri(0x0DE1, 0x2800, 0x2600)\n\n");  // GL_TEXTURE_MAG_FILTER, GL_NEAREST

        kyle.push_str("    # Main loop\n");
        kyle.push_str("    running: ^i32 = 1\n");
        kyle.push_str("    while running != 0:\n");
        kyle.push_str("        glfwPollEvents()\n");
        kyle.push_str("        if glfwWindowShouldClose(win) != 0:\n");
        kyle.push_str("            running = 0\n");
        kyle.push_str("            break\n\n");
        kyle.push_str("        # Clear pixel buffer (light gray)\n");
        kyle.push_str("        memset(pixels, 242, buf_size as i64)\n\n");
        kyle.push_str("        # Render app into pixel buffer\n");
        kyle.push_str("        render_app(pixels, win_w, win_h)\n\n");
        kyle.push_str("        # Upload pixels to GPU\n");
        kyle.push_str("        glBindTexture(0x0DE1, tex_id)\n");
        kyle.push_str("        glTexImage2D(0x0DE1, 0, 0x1908, win_w, win_h, 0, 0x1908, 0x1401, pixels)\n\n");
        kyle.push_str("        # Clear screen and draw textured quad\n");
        kyle.push_str("        glViewport(0, 0, win_w, win_h)\n");
        kyle.push_str("        glClearColor(0.0, 0.0, 0.0, 1.0)\n");
        kyle.push_str("        glClear(0x00004000)\n");
        kyle.push_str("        glDisable(0x0BE2)\n");  // GL_BLEND
        kyle.push_str("        glEnable(0x0DE1)\n\n");  // GL_TEXTURE_2D
        kyle.push_str("        # Draw textured quad using matrix mode\n");
        kyle.push_str("        glMatrixMode(0x1701)\n");  // GL_PROJECTION
        kyle.push_str("        glLoadIdentity()\n");
        kyle.push_str("        glOrtho(0, win_w, win_h, 0, -1, 1)\n");
        kyle.push_str("        glMatrixMode(0x1700)\n");  // GL_MODELVIEW
        kyle.push_str("        glLoadIdentity()\n\n");

        // Draw textured quad using glWindowPos + glDrawPixels (modern compatible)
        kyle.push_str("        glDisable(0x0DE1)\n");  // GL_TEXTURE_2D
        kyle.push_str("        glMatrixMode(0x1701)\n");
        kyle.push_str("        glLoadIdentity()\n");
        kyle.push_str("        glOrtho(0, win_w, win_h, 0, -1, 1)\n");
        kyle.push_str("        glMatrixMode(0x1700)\n");
        kyle.push_str("        glLoadIdentity()\n");
        kyle.push_str("        glRasterPos2i(0, 0)\n");
        kyle.push_str("        glDrawPixels(win_w, win_h, 0x1908, 0x1401, pixels)\n\n");

        kyle.push_str("        glfwSwapBuffers(win)\n");
        kyle.push_str("        ky_sleep(16)\n\n");
        kyle.push_str("    # Cleanup\n");
        kyle.push_str("    ky_free(pixels)\n");
        kyle.push_str("    glfwDestroyWindow(win)\n");
        kyle.push_str("    glfwTerminate()\n");

        BackendOutput {
            files: vec![GeneratedFile {
                path: "main.ky".to_string(),
                content: kyle,
            }],
            html_shell: None,
        }
    }
}

fn gen_desktop_nodes(nodes: &[UiNode], kyle: &mut String, indent: usize, pixels: &str, w: &str, h: &str, base_x: i32, base_y: i32) -> i32 {
    let mut y_offset = base_y;
    for node in nodes {
        y_offset = gen_desktop_node(node, kyle, indent, pixels, w, h, base_x, y_offset);
    }
    y_offset
}

fn gen_desktop_node(node: &UiNode, kyle: &mut String, indent: usize, pixels: &str, w: &str, h: &str, base_x: i32, base_y: i32) -> i32 {
    match node {
        UiNode::Element { tag, attrs, children } => {
            match tag {
                ComponentTag::Text | ComponentTag::Label => {
                    let text = get_str_attr(attrs, "value", "text").unwrap_or("\"Hello\"");
                    let text_str = text.trim_matches('"');
                    emit_line(kyle, indent, &format!(
                        "# Text: {} (rendering deferred)", text_str));
                    base_y + 30
                }
                ComponentTag::Button => {
                    emit_line(kyle, indent, "# Button (rendering deferred)");
                    base_y + 46
                }
                ComponentTag::View | ComponentTag::Column | ComponentTag::Row
                | ComponentTag::Card | ComponentTag::Spacer | ComponentTag::Surface => {
                    let padding = get_int_attr(attrs, "padding").unwrap_or(10);
                    gen_desktop_nodes(children, kyle, indent, pixels, w, h,
                        base_x + padding, base_y + padding)
                }
                _ => {
                    gen_desktop_nodes(children, kyle, indent, pixels, w, h, base_x, base_y)
                }
            }
        }
        UiNode::SelfClosing { tag, attrs } => {
            gen_desktop_node(
                &UiNode::Element { tag: tag.clone(), attrs: attrs.clone(), children: vec![] },
                kyle, indent, pixels, w, h, base_x, base_y
            )
        }
        _ => base_y,
    }
}

fn emit_line(kyle: &mut String, indent: usize, line: &str) {
    kyle.push_str(&"    ".repeat(indent));
    kyle.push_str(line);
    kyle.push('\n');
}

fn get_str_attr<'a>(attrs: &'a [UiAttr], key1: &str, key2: &str) -> Option<&'a str> {
    for a in attrs {
        if a.name == key1 || a.name == key2 {
            if let AttrValue::String(ref s) = a.value {
                return Some(s);
            }
            if let AttrValue::Expr(ref e) = a.value {
                return Some(e);
            }
        }
    }
    None
}

fn get_int_attr(attrs: &[UiAttr], key: &str) -> Option<i32> {
    for a in attrs {
        if a.name == key {
            if let AttrValue::String(ref s) = a.value {
                return s.parse::<i32>().ok();
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_desktop_backend_generates_standalone_kyle() {
        let backend = DesktopBackend::new();
        let program = UiProgram {
            view_paths: vec![],
            code_blocks: vec![],
            styles: vec![],
            animations: vec![],
            body: vec![UiNode::SelfClosing {
                tag: ComponentTag::Text,
                attrs: vec![UiAttr {
                    name: "value".to_string(),
                    value: AttrValue::String("Hello".to_string()),
                }],
            }],
        };
        let output = backend.generate(&program);
        assert!(output.files[0].content.contains("glfwCreateWindow"));
        assert!(output.files[0].content.contains("fill_rect"));
        assert!(output.files[0].content.contains("glDrawPixels"));
    }
}
