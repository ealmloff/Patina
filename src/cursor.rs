pub struct Cursor {
    pos: i32,
}

impl Cursor {
    pub fn handle_input(&mut self, data: &dioxus_html::on::KeyboardData) {}
}

impl Default for Cursor {
    fn default() -> Self {
        Self { pos: 0 }
    }
}
