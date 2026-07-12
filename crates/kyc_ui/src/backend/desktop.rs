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
        kyle.push_str("# Kyle UI — Desktop Application (self-contained)\n\n");

        // ── GLFW FFI declarations ──
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

        // ── OpenGL FFI ──
        kyle.push_str("extern fn glClearColor(r: f32, g: f32, b: f32, a: f32)\n");
        kyle.push_str("extern fn glClear(mask: i32)\n");
        kyle.push_str("extern fn glViewport(x: i32, y: i32, w: i32, h: i32)\n");
        kyle.push_str("extern fn glBegin(mode: i32)\n");
        kyle.push_str("extern fn glEnd()\n");
        kyle.push_str("extern fn glVertex2f(x: f32, y: f32)\n");
        kyle.push_str("extern fn glColor4f(r: f32, g: f32, b: f32, a: f32)\n\n");

        // ── Runtime helpers ──
        kyle.push_str("@link \"c\"\n");
        kyle.push_str("extern fn ky_alloc(size: i64) ptr\n");
        kyle.push_str("extern fn ky_free(ptr)\n");
        kyle.push_str("extern fn ky_memset(ptr, val: i32, size: i64)\n");
        kyle.push_str("extern fn ky_now_ms() i64\n");
        kyle.push_str("extern fn ky_sleep(ms: i64)\n\n");

        // ── Render function ──
        kyle.push_str("fn render_app(canvas_w: i32, canvas_h: i32):\n");

        // Read view_paths for page title
        let title = program.view_paths.first()
            .cloned().unwrap_or_else(|| "/".to_string());

        gen_desktop_nodes(&program.body, &mut kyle, 1, 0, 0);

        kyle.push_str("\n");

        // ── Main function ──
        kyle.push_str("fn main(args: {str}):\n");
        kyle.push_str("    # Init GLFW\n");
        kyle.push_str("    glfw_ok = glfwInit()\n");
        kyle.push_str("    if glfw_ok == 0:\n");
        kyle.push_str("        return\n\n");
        kyle.push_str("    # Create window\n");
        kyle.push_str("    win = glfwCreateWindow(800, 600, &(\"Kyle Desktop\" as ptr), 0 as ptr, 0 as ptr)\n");
        kyle.push_str("    if win == 0 as ptr:\n");
        kyle.push_str("        glfwTerminate()\n");
        kyle.push_str("        return\n\n");
        kyle.push_str("    glfwMakeContextCurrent(win)\n\n");
        kyle.push_str("    # Main loop\n");
        kyle.push_str("    running: ^i32 = 1\n");
        kyle.push_str("    while running != 0:\n");
        kyle.push_str("        glfwPollEvents()\n");
        kyle.push_str("        if glfwWindowShouldClose(win) != 0:\n");
        kyle.push_str("            running = 0\n");
        kyle.push_str("            break\n\n");
        kyle.push_str("        # Get framebuffer size\n");
        kyle.push_str("        fb_w: ^i32 = 0\n");
        kyle.push_str("        fb_h: ^i32 = 0\n");
        kyle.push_str("        glfwGetFramebufferSize(win, ^&fb_w, ^&fb_h)\n\n");
        kyle.push_str("        # Clear screen\n");
        kyle.push_str("        glViewport(0, 0, fb_w, fb_h)\n");
        kyle.push_str("        glClearColor(0.95, 0.95, 0.95, 1.0)\n");
        kyle.push_str("        glClear(0x00004000)\n\n");
        kyle.push_str("        # Render app\n");
        kyle.push_str("        render_app(fb_w, fb_h)\n\n");
        kyle.push_str("        glfwSwapBuffers(win)\n");
        kyle.push_str("        ky_sleep(16)\n\n");
        kyle.push_str("    # Cleanup\n");
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

fn gen_desktop_nodes(nodes: &[UiNode], kyle: &mut String, indent: usize, base_x: i32, base_y: i32) -> i32 {
    let mut y_offset = base_y;
    for node in nodes {
        y_offset = gen_desktop_node(node, kyle, indent, base_x, y_offset);
    }
    y_offset
}

/// Generate drawing commands for a single node, returning the updated y_offset.
fn gen_desktop_node(node: &UiNode, kyle: &mut String, indent: usize, base_x: i32, base_y: i32) -> i32 {
    match node {
        UiNode::Element { tag, attrs, children } => {
            match tag {
                ComponentTag::Text | ComponentTag::Label => {
                    let text = get_str_attr(attrs, "value", "text").unwrap_or("\"Hello\"");
                    let x = get_int_attr(attrs, "x").unwrap_or(base_x + 20);
                    let y = get_int_attr(attrs, "y").unwrap_or(base_y + 30);
                    let color = get_color_attrs(attrs);
                    let text_str = text.trim_matches('"');

                    emit_line(kyle, indent, &format!("# Text: {}", text_str));
                    emit_line(kyle, indent, &format!(
                        "glColor4f({:.1}, {:.1}, {:.1}, 1.0)", color.0, color.1, color.2));
                    emit_line(kyle, indent, "glBegin(0x0006)");
                    emit_line(kyle, indent, &format!("glVertex2f({} as f32, {} as f32)", x, y));
                    emit_line(kyle, indent, &format!("glVertex2f({} as f32 + 200.0, {} as f32)", x, y));
                    emit_line(kyle, indent, &format!("glVertex2f({} as f32 + 200.0, {} as f32 + 20.0)", x, y));
                    emit_line(kyle, indent, &format!("glVertex2f({} as f32, {} as f32 + 20.0)", x, y));
                    emit_line(kyle, indent, "glEnd()");

                    base_y + 60 // each text takes ~60px
                }
                ComponentTag::Button => {
                    let x = get_int_attr(attrs, "x").unwrap_or(base_x + 20);
                    let y = get_int_attr(attrs, "y").unwrap_or(base_y + 50);
                    let btn_w = 100; let btn_h = 36;

                    emit_line(kyle, indent, "# Button");
                    emit_line(kyle, indent, "glColor4f(0.0, 0.4, 1.0, 1.0)");
                    emit_line(kyle, indent, "glBegin(0x0006)");
                    emit_line(kyle, indent, &format!("glVertex2f({} as f32, {} as f32)", x, y));
                    emit_line(kyle, indent, &format!("glVertex2f({} as f32, {} as f32)", x + btn_w, y));
                    emit_line(kyle, indent, &format!("glVertex2f({} as f32, {} as f32)", x + btn_w, y + btn_h));
                    emit_line(kyle, indent, &format!("glVertex2f({} as f32, {} as f32)", x, y + btn_h));
                    emit_line(kyle, indent, "glEnd()");

                    base_y + btn_h + 10
                }
                ComponentTag::View | ComponentTag::Column | ComponentTag::Row
                | ComponentTag::Card | ComponentTag::Spacer | ComponentTag::Surface => {
                    let padding = get_int_attr(attrs, "padding").unwrap_or(0);
                    gen_desktop_nodes(children, kyle, indent, base_x + padding, base_y + padding)
                }
                _ => {
                    // Custom/unknown component — render children at same level
                    gen_desktop_nodes(children, kyle, indent, base_x, base_y)
                }
            }
        }
        UiNode::SelfClosing { tag, attrs } => {
            gen_desktop_node(
                &UiNode::Element { tag: tag.clone(), attrs: attrs.clone(), children: vec![] },
                kyle, indent, base_x, base_y
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

fn get_color_attrs(attrs: &[UiAttr]) -> (f32, f32, f32) {
    for a in attrs {
        if a.name == "color" {
            if let AttrValue::String(ref s) = a.value {
                if s.starts_with('#') {
                    let hex = s.trim_start_matches('#');
                    if let Ok(val) = i32::from_str_radix(hex, 16) {
                        let r = ((val >> 16) & 0xFF) as f32 / 255.0;
                        let g = ((val >> 8) & 0xFF) as f32 / 255.0;
                        let b = (val & 0xFF) as f32 / 255.0;
                        return (r, g, b);
                    }
                }
            }
        }
    }
    (0.2, 0.2, 0.2) // default dark gray
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
        assert!(output.files[0].content.contains("extern fn"));
        assert!(output.files[0].content.contains("fn main"));
        assert!(output.files[0].content.contains("glfwCreateWindow"));
    }
}
