use syntect::highlighting::Color;

pub fn color_to_string(c: Color) -> String {
    format!("rgba({}, {}, {}, {})", c.r, c.g, c.b, c.a as f32 / 255.0)
}

#[test]
fn color_to_string_test() {
    assert_eq!(
        "rgba(255, 255, 255, 1)".to_string(),
        color_to_string(Color {
            r: 255,
            g: 255,
            b: 255,
            a: 255,
        })
    );
    assert_eq!(
        "rgba(10, 10, 10, 0.33333334)".to_string(),
        color_to_string(Color {
            r: 10,
            g: 10,
            b: 10,
            a: 85,
        })
    );
}
