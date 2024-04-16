use hudhook::hooks::dx11::ImguiDx11Hooks;
use hudhook::ImguiRenderLoop;
use imgui::*;
use spoiler::Spoilers;
use std::time::Instant;

mod spoiler;

struct NightjarApp {
    _start_time: Instant,
    spoilers: Spoilers,
    ui_file_path: String,
    ui_error_text: String,
}

impl NightjarApp {
    fn new() -> Self {
        let mut spoilers: Spoilers = Spoilers::new();
        let error_text = match spoilers.read_recent() {
            Ok(_) => {String::new()},
            Err(e) => {e.to_string()},
        };

        Self {
            _start_time: Instant::now(),
            spoilers: spoilers,
            ui_file_path: String::new(),
            ui_error_text: error_text,
        }
    }
}

impl ImguiRenderLoop for NightjarApp {
    fn render(&mut self, ui: &mut Ui) {
        let position: [f32; 2] = [0., 0.];
        ui.window("Nightjar Monocular")
            .size([600., 200.], Condition::Appearing)
            .position(position, Condition::Appearing)
            .build(|| {
                if ui.button("custom") {
                    match self.spoilers.read_file(&self.ui_file_path) {
                        Ok(_) => self.ui_error_text.clear(),
                        Err(e) => self.ui_error_text = e.to_string(),
                    }
                }
                ui.same_line();
                ui.input_text("log directory", &mut self.ui_file_path)
                    .build();

                if !self.ui_error_text.is_empty() {
                    let color = [1., 0., 0., 1.];
                    ui.text_colored(color, &self.ui_error_text);
                }

                self.spoilers.key_items.render(ui);
                self.spoilers.quest_items.render(ui);
                self.spoilers.upgrade_items.render(ui);
                self.spoilers.healing_items.render(ui);
            });
    }
}

hudhook::hudhook!(ImguiDx11Hooks, NightjarApp::new());
