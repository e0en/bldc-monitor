#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use std::sync::mpsc::{Receiver, Sender, channel};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use eframe::egui;

fn main() -> eframe::Result {
    env_logger::init();

    let (command_sender, command_receiver) = channel::<MotorCommand>();
    let (status_sender, status_receiver) = channel::<MotorStatus>();

    thread::spawn(|| {
        communicate(status_sender, command_receiver);
    });

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1024.0, 768.0]),
        ..Default::default()
    };
    eframe::run_native(
        "BLDC monitor",
        options,
        Box::new(|_| {
            Ok(Box::<MyApp>::new(MyApp::new(
                command_sender,
                status_receiver,
            )))
        }),
    )
}

struct MotorStatus {
    timestamp: f32,
    angle: f32,
    velocity: f32,
    torque: f32,
}

fn communicate(status_send: Sender<MotorStatus>, command_recv: Receiver<MotorCommand>) {
    let t0 = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    loop {
        for c in command_recv.try_iter() {
            send_command(c);
        }
        if let Ok(t) = SystemTime::now().duration_since(UNIX_EPOCH) {
            let dt = t - t0;
            let timestamp = (dt.as_micros() as f32) / 1_000_000.0;

            let angle = f32::sin(timestamp * 500.0);
            let velocity = f32::cos(timestamp * 251.0);
            let torque = f32::cos(2.0 * timestamp * 101.0);

            let _ = status_send.send(MotorStatus {
                timestamp,
                angle,
                velocity,
                torque,
            });
            std::thread::sleep(Duration::from_millis(1));
        }
    }
}

struct MyApp {
    command_send: Sender<MotorCommand>,
    status_recv: Receiver<MotorStatus>,

    angle_string: String,
    velocity_string: String,
    torque_string: String,

    angle: f32,
    velocity: f32,
    torque: f32,

    plot_type: PlotType,
    plot_points: Vec<egui_plot::PlotPoint>,
    is_plotting: bool,
}

#[derive(Clone, Copy)]
enum MotorCommand {
    Angle(f32),
    Velocity(f32),
    Torque(f32),
    Enable,
    Disable,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PlotType {
    Angle,
    Velocity,
    Torque,
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

impl MyApp {
    fn new(command_send: Sender<MotorCommand>, status_recv: Receiver<MotorStatus>) -> Self {
        let position = 0.0;
        let velocity = 0.0;
        let torque = 0.0;

        Self {
            command_send,
            status_recv,

            angle_string: position.to_string(),
            velocity_string: velocity.to_string(),
            torque_string: torque.to_string(),

            angle: position,
            velocity,
            torque,

            plot_type: PlotType::Angle,
            plot_points: vec![],
            is_plotting: false,
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        for s in self.status_recv.try_iter() {
            if self.is_plotting {
                let value = match self.plot_type {
                    PlotType::Angle => s.angle,
                    PlotType::Velocity => s.velocity,
                    PlotType::Torque => s.torque,
                };
                let new_point = egui_plot::PlotPoint::new(s.timestamp as f64, value as f64);
                self.plot_points.push(new_point);
            }
        }
        ctx.set_pixels_per_point(1.5);
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("BLDC Monitor");
            ui.horizontal(|ui| {
                let name_label = ui.label("Angle");
                ui.text_edit_singleline(&mut self.angle_string)
                    .labelled_by(name_label.id);

                let mut is_button_active = true;
                match self.angle_string.parse::<f32>() {
                    Ok(n) => self.angle = n,
                    _ => {
                        is_button_active = false;
                    }
                }

                if ui
                    .add_enabled(is_button_active, egui::Button::new("Set"))
                    .clicked()
                {
                    let _ = self.command_send.send(MotorCommand::Angle(self.angle));
                };
            });

            ui.horizontal(|ui| {
                let name_label = ui.label("Velocity");
                ui.text_edit_singleline(&mut self.velocity_string)
                    .labelled_by(name_label.id);

                let mut is_button_active = true;
                match self.velocity_string.parse::<f32>() {
                    Ok(n) => self.velocity = n,
                    _ => {
                        is_button_active = false;
                    }
                }

                if ui
                    .add_enabled(is_button_active, egui::Button::new("Set"))
                    .clicked()
                {
                    let _ = self
                        .command_send
                        .send(MotorCommand::Velocity(self.velocity));
                };
            });

            ui.horizontal(|ui| {
                let name_label = ui.label("Torque");
                ui.text_edit_singleline(&mut self.torque_string)
                    .labelled_by(name_label.id);

                let mut is_button_active = true;
                match self.torque_string.parse::<f32>() {
                    Ok(n) => self.torque = n,
                    _ => {
                        is_button_active = false;
                    }
                }

                if ui
                    .add_enabled(is_button_active, egui::Button::new("Set"))
                    .clicked()
                {
                    let _ = self.command_send.send(MotorCommand::Torque(self.torque));
                };
            });

            if ui.button("disable").clicked() {
                let _ = self.command_send.send(MotorCommand::Disable);
            }
            if ui.button("enable").clicked() {
                let _ = self.command_send.send(MotorCommand::Enable);
            }

            let before = self.plot_type;
            ui.label("Plot Type");
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.plot_type, PlotType::Angle, "Angle");
                ui.selectable_value(&mut self.plot_type, PlotType::Velocity, "Velocity");
                ui.selectable_value(&mut self.plot_type, PlotType::Torque, "Torque");
            });
            if before != self.plot_type {
                self.plot_points.clear();
            }
            if ui.button("Plot").clicked() {
                if !self.is_plotting {
                    self.plot_points.clear();
                }
                self.is_plotting = !self.is_plotting;
            }

            egui_plot::Plot::new("plot").show(ui, |plot_ui| {
                let points = egui_plot::PlotPoints::Borrowed(&self.plot_points);
                let line = egui_plot::Line::new("Sine", points)
                    .name("Sine")
                    .style(egui_plot::LineStyle::Solid)
                    .color(egui::Color32::RED)
                    .width(2.0);
                plot_ui.line(line);
            });
            ctx.request_repaint();
        });
    }
}
