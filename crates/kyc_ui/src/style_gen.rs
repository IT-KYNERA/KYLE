use crate::ast::*;

/// Compile style/template/theme declarations to JS objects.
pub fn generate_styles(styles: &[StyleDecl]) -> String {
    let mut js = String::new();

    // Generate style objects
    js.push_str("const styles = {\n");
    for decl in styles {
        match decl {
            StyleDecl::Style { name, props, .. }
            | StyleDecl::Layout { name, props, .. }
            | StyleDecl::Template { name, props, .. } => {
                js.push_str(&format!("  '{}': {{\n", name));
                for prop in props {
                    let css_prop = to_css_name(&prop.name);
                    let css_val = to_css_value(&prop.name, &prop.value);
                    js.push_str(&format!("    '{}': '{}',\n", css_prop, css_val));
                }
                js.push_str("  },\n");
            }
            StyleDecl::Theme { name, props } => {
                js.push_str(&format!("  'theme:{}': {{\n", name));
                for prop in props {
                    js.push_str(&format!("    '{}': '{}',\n", prop.name, prop.value));
                }
                js.push_str("  },\n");
            }
        }
    }
    js.push_str("};\n\n");

    // Generate applyStyle helper
    js.push_str("function applyStyle(el, styleName) {\n");
    js.push_str("  const s = styles[styleName];\n");
    js.push_str("  if (!s) return;\n");
    js.push_str("  for (const [prop, val] of Object.entries(s)) {\n");
    js.push_str("    el.style[prop] = val;\n");
    js.push_str("  }\n");
    js.push_str("}\n\n");

    js
}

fn to_css_name(prop: &str) -> &str {
    match prop {
        "background" => "background",
        "color" => "color",
        "font_size" => "fontSize",
        "font_weight" => "fontWeight",
        "font_family" => "fontFamily",
        "line_height" => "lineHeight",
        "letter_spacing" => "letterSpacing",
        "text_align" => "textAlign",
        "border_radius" => "borderRadius",
        "border" => "border",
        "border_top" => "borderTop",
        "border_right" => "borderRight",
        "border_bottom" => "borderBottom",
        "border_left" => "borderLeft",
        "padding" => "padding",
        "margin" => "margin",
        "width" => "width",
        "height" => "height",
        "min_width" => "minWidth",
        "max_width" => "maxWidth",
        "min_height" => "minHeight",
        "max_height" => "maxHeight",
        "opacity" => "opacity",
        "cursor" => "cursor",
        "overflow" => "overflow",
        "display" => "display",
        "gap" => "gap",
        "z_index" => "zIndex",
        "shadow" => "boxShadow",
        "transform" => "transform",
        "transition" => "transition",
        _ => prop, // passthrough
    }
}

fn to_css_value(prop: &str, val: &str) -> String {
    // Handle function calls like Color("#...") or Spacing.all(12)
    let val = val.trim();
    if val.starts_with("Color(") {
        let inner = val.trim_start_matches("Color(").trim_end_matches(')');
        return inner.trim_matches('"').to_string();
    }
    if val.starts_with("Spacing") {
        // Extract a pixel value
        if let Some(num) = val.split(|c: char| !c.is_ascii_digit()).find(|s| !s.is_empty()) {
            return format!("{}px", num);
        }
        return "0".to_string();
    }
    if prop == "font_size" || prop == "border_radius" || prop == "gap"
        || prop == "line_height"
    {
        if val.chars().all(|c| c.is_ascii_digit() || c == '.') {
            return format!("{}px", val);
        }
    }
    // Handle hex colors directly
    if val.starts_with('#') || val == "white" || val == "black" || val == "transparent" {
        return val.to_string();
    }
    val.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_style_to_css() {
        let styles = vec![
            StyleDecl::Style {
                component: "button".to_string(),
                name: "Primary".to_string(),
                props: vec![
                    StyleProp { name: "background".to_string(), value: "Color(\"#0066FF\")".to_string() },
                    StyleProp { name: "color".to_string(), value: "Color(\"#FFFFFF\")".to_string() },
                    StyleProp { name: "border_radius".to_string(), value: "8".to_string() },
                    StyleProp { name: "font_size".to_string(), value: "14".to_string() },
                ],
            },
        ];
        let js = generate_styles(&styles);
        assert!(js.contains("Primary"));
        assert!(js.contains("background"));
        assert!(js.contains("0066FF"));
        assert!(js.contains("8px"));
    }
}
