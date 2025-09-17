#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::egui;

fn main() -> eframe::Result {
    env_logger::init();
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1024.0, 768.0]),
        ..Default::default()
    };
    eframe::run_native(
        "BLDC monitor",
        options,
        Box::new(|_| Ok(Box::<MyApp>::default())),
    )
}

struct MyApp {
    angle_string: String,
    velocity_string: String,
    torque_string: String,

    angle: f32,
    velocity: f32,
    torque: f32,
}

#[derive(Clone, Copy)]
enum MotorCommand {
    Angle(f32),
    Velocity(f32),
    Torque(f32),
    Enable,
    Disable,
}

fn send_command(c: MotorCommand) {
    match c {
        MotorCommand::Angle(n) => println!("Angle {}", n),
        MotorCommand::Velocity(n) => println!("Velocity {}", n),
        MotorCommand::Torque(n) => println!("Torque {}", n),
        MotorCommand::Enable => println!("Enable"),
        MotorCommand::Disable => println!("Disable"),
    }
}

impl Default for MyApp {
    fn default() -> Self {
        let position = 0.0;
        let velocity = 0.0;
        let torque = 0.0;

        Self {
            angle_string: position.to_string(),
            velocity_string: velocity.to_string(),
            torque_string: torque.to_string(),

            angle: position,
            velocity,
            torque,
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.set_pixels_per_point(1.5);
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("BLDC Monitor");
            ui.horizontal(|ui| {
                let name_label = ui.label("Angle");
                let value_edit = ui
                    .text_edit_singleline(&mut self.angle_string)
                    .labelled_by(name_label.id);

                let mut is_button_active = true;
                if value_edit.changed() {
                    match self.angle_string.parse::<f32>() {
                        Ok(n) => self.angle = n,
                        _ => {
                            is_button_active = false;
                        }
                    }
                }

                if ui
                    .add_enabled(is_button_active, egui::Button::new("Set"))
                    .clicked()
                {
                    send_command(MotorCommand::Angle(self.angle));
                };
            });

            ui.horizontal(|ui| {
                let name_label = ui.label("Velocity");
                let value_edit = ui
                    .text_edit_singleline(&mut self.velocity_string)
                    .labelled_by(name_label.id);

                let mut is_button_active = true;
                if value_edit.changed() {
                    match self.velocity_string.parse::<f32>() {
                        Ok(n) => self.velocity = n,
                        _ => {
                            is_button_active = false;
                        }
                    }
                }

                if ui
                    .add_enabled(is_button_active, egui::Button::new("Set"))
                    .clicked()
                {
                    send_command(MotorCommand::Velocity(self.velocity));
                };
            });

            ui.horizontal(|ui| {
                let name_label = ui.label("Torque");
                let value_edit = ui
                    .text_edit_singleline(&mut self.torque_string)
                    .labelled_by(name_label.id);

                let mut is_button_active = true;
                if value_edit.changed() {
                    match self.torque_string.parse::<f32>() {
                        Ok(n) => self.torque = n,
                        _ => {
                            is_button_active = false;
                        }
                    }
                }

                if ui
                    .add_enabled(is_button_active, egui::Button::new("Set"))
                    .clicked()
                {
                    send_command(MotorCommand::Torque(self.torque));
                };
            });

            if ui.button("disable").clicked() {
                send_command(MotorCommand::Disable);
            }
            if ui.button("enable").clicked() {
                send_command(MotorCommand::Enable);
            }
        });
    }
}
