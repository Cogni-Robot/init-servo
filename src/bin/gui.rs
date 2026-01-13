use eframe::egui;
use egui_plot::{Line, Plot, PlotPoints};
use st3215::ST3215;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{channel, Sender, Receiver};
use std::thread;
use std::time::{Duration, Instant};

#[derive(Clone)]
enum ServoCommand {
    Move { id: u8, position: u16, speed: u16, acceleration: u8 },
    EnableTorque { id: u8 },
    DisableTorque { id: u8 },
    ScanServos,
}

struct ServoData {
    position: Option<u16>,
    speed: Option<u16>,
    load: Option<f32>,
    voltage: Option<f32>,
    current: Option<f32>,
    temperature: Option<u8>,
    is_moving: Option<bool>,
    last_update: Instant,
}

impl Default for ServoData {
    fn default() -> Self {
        Self {
            position: None,
            speed: None,
            load: None,
            voltage: None,
            current: None,
            temperature: None,
            is_moving: None,
            last_update: Instant::now(),
        }
    }
}

const PORT: &str = "/dev/ttyACM0";

struct AppState {
    connected: bool,
    servo_ids: Vec<u8>,
    selected_servo: Option<u8>,
    servo_data: ServoData,
    new_id_input: String,
    target_position: u16,
    target_speed: u16,
    acceleration: u8,
    torque_enabled: bool,
    position_history: Vec<(f64, f64)>,
    temperature_history: Vec<(f64, f64)>,
    start_time: Instant,
    command_sender: Sender<ServoCommand>,
}

impl Default for AppState {
    fn default() -> Self {
        let (tx, _) = channel();
        Self {
            connected: false,
            servo_ids: Vec::new(),
            selected_servo: None,
            servo_data: ServoData::default(),
            new_id_input: String::new(),
            target_position: 2048,
            target_speed: 1000,
            acceleration: 50,
            torque_enabled: false,
            position_history: Vec::new(),
            temperature_history: Vec::new(),
            start_time: Instant::now(),
            command_sender: tx,
        }
    }
}

struct ServoGuiApp {
    state: Arc<Mutex<AppState>>,
}

impl ServoGuiApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let (tx, rx) = channel::<ServoCommand>();
        let mut default_state = AppState::default();
        default_state.command_sender = tx;
        let state = Arc::new(Mutex::new(default_state));
        
        // Configure le style moderne
        let mut style = (*cc.egui_ctx.style()).clone();
        style.visuals.window_corner_radius = egui::CornerRadius::same(10);
        style.visuals.window_shadow.blur = 20;
        style.spacing.item_spacing = egui::vec2(8.0, 8.0);
        cc.egui_ctx.set_style(style);
        
        // Thread de monitoring
        let state_clone = Arc::clone(&state);
        let ctx_clone = cc.egui_ctx.clone();
        thread::spawn(move || {
            monitoring_thread(state_clone, ctx_clone, rx);
        });

        Self { state }
    }
}

impl eframe::App for ServoGuiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Panel supérieur avec titre
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.add_space(10.0);
            ui.horizontal(|ui| {
                ui.heading("Cogni-Robot Servo Control");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label("by notpunchnox");
                    let state = self.state.lock().unwrap();
                    let status_color = if state.connected {
                        egui::Color32::from_rgb(46, 204, 113)
                    } else {
                        egui::Color32::from_rgb(231, 76, 60)
                    };
                    ui.colored_label(status_color, if state.connected { "Connected" } else { "Disconnected" });
                });
            });
            ui.add_space(10.0);
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            let mut state = self.state.lock().unwrap();
            
            ui.add_space(10.0);
            
            // Section de détection des servos
            ui.group(|ui| {
                ui.set_min_height(100.0);
                ui.heading("Servo Detection");
                ui.add_space(5.0);
                
                ui.horizontal(|ui| {
                    if ui.button("Scan Servos").clicked() {
                        let _ = state.command_sender.send(ServoCommand::ScanServos);
                    }
                    
                    if state.servo_ids.is_empty() {
                        ui.colored_label(egui::Color32::from_rgb(230, 126, 34), "⚠ No servo detected");
                    } else {
                        ui.label(format!("Detected servos: {} ", state.servo_ids.len()));
                        ui.label(format!("{:?}", state.servo_ids));
                    }
                });
                
                if !state.servo_ids.is_empty() {
                    ui.add_space(5.0);
                    ui.horizontal(|ui| {
                        ui.label("Select servo:");
                        for &id in &state.servo_ids.clone() {
                            let is_selected = state.selected_servo == Some(id);
                            if ui.selectable_label(is_selected, format!("ID {}", id)).clicked() {
                                state.selected_servo = Some(id);
                            }
                        }
                    });
                }
            });

            ui.add_space(10.0);

            // Section de changement d'ID
            if state.servo_ids.len() == 1 {
                ui.group(|ui| {
                    ui.heading("Change Servo ID");
                    ui.add_space(5.0);
                    
                    ui.horizontal(|ui| {
                        ui.label(format!("Current ID: {}", state.servo_ids[0]));
                        ui.label("→ New ID:");
                        ui.add(egui::TextEdit::singleline(&mut state.new_id_input)
                            .desired_width(60.0)
                            .hint_text("0-253"));
                        
                        if ui.button("Apply").clicked() {
                            if let Ok(new_id) = state.new_id_input.parse::<u8>() {
                                if new_id <= 253 {
                                    if let Ok(servo) = ST3215::new(PORT) {
                                        let _ = servo.change_id(state.servo_ids[0], new_id);
                                    }
                                }
                            }
                        }
                    });
                });
                ui.add_space(10.0);
            }

            // Section de contrôle du servo sélectionné
            if let Some(servo_id) = state.selected_servo {
                ui.group(|ui| {
                    ui.heading(format!("Control Servo ID {}", servo_id));
                    ui.add_space(5.0);
                    
                    // Affichage des données en temps réel
                    ui.columns(3, |columns| {
                        columns[0].vertical(|ui| {
                            ui.label("Position:");
                            if let Some(pos) = state.servo_data.position {
                                ui.heading(format!("{}", pos));
                            } else {
                                ui.label("N/A");
                            }
                        });
                        
                        columns[1].vertical(|ui| {
                            ui.label("Temperature:");
                            if let Some(temp) = state.servo_data.temperature {
                                let color = if temp > 60 {
                                    egui::Color32::RED
                                } else if temp > 45 {
                                    egui::Color32::from_rgb(230, 126, 34)
                                } else {
                                    egui::Color32::from_rgb(46, 204, 113)
                                };
                                ui.colored_label(color, format!("{}°C", temp));
                            } else {
                                ui.label("N/A");
                            }
                        });
                        
                        columns[2].vertical(|ui| {
                            ui.label("Voltage:");
                            if let Some(v) = state.servo_data.voltage {
                                ui.heading(format!("{:.2}V", v));
                            } else {
                                ui.label("N/A");
                            }
                        });
                    });
                    
                    ui.add_space(10.0);
                    
                    // Contrôles de mouvement
                    ui.separator();
                    ui.add_space(5.0);
                    ui.label("Target Position (0-4095):");
                    ui.add(egui::Slider::new(&mut state.target_position, 0..=4095));
                    
                    ui.label("Speed (0-3400):");
                    ui.add(egui::Slider::new(&mut state.target_speed, 0..=3400));
                    
                    ui.label("Acceleration (0-254):");
                    ui.add(egui::Slider::new(&mut state.acceleration, 0..=254));
                    
                    ui.add_space(5.0);
                    
                    ui.horizontal(|ui| {
                        if ui.button("Move").clicked() {
                            let _ = state.command_sender.send(ServoCommand::Move {
                                id: servo_id,
                                position: state.target_position,
                                speed: state.target_speed,
                                acceleration: state.acceleration,
                            });
                            if !state.torque_enabled {
                                state.torque_enabled = true;
                            }
                        }
                        
                        let torque_text = if state.torque_enabled { "Disable Torque" } else { "Enable Torque" };
                        if ui.button(torque_text).clicked() {
                            if state.torque_enabled {
                                let _ = state.command_sender.send(ServoCommand::DisableTorque { id: servo_id });
                            } else {
                                let _ = state.command_sender.send(ServoCommand::EnableTorque { id: servo_id });
                            }
                            state.torque_enabled = !state.torque_enabled;
                        }
                    });
                });
                
                ui.add_space(10.0);
                
                // Graphiques
                ui.group(|ui| {
                    ui.heading("Real-time Monitoring");
                    ui.add_space(5.0);
                    
                    // Graphique de position
                    Plot::new("position_plot")
                        .height(150.0)
                        .view_aspect(2.0)
                        .show(ui, |plot_ui| {
                            let points: PlotPoints = state.position_history.iter()
                                .map(|(x, y)| [*x, *y])
                                .collect();
                            plot_ui.line(Line::new("Position", points).color(egui::Color32::from_rgb(52, 152, 219)));
                        });
                    
                    ui.add_space(5.0);
                    
                    // Graphique de température
                    Plot::new("temperature_plot")
                        .height(150.0)
                        .view_aspect(2.0)
                        .show(ui, |plot_ui| {
                            let points: PlotPoints = state.temperature_history.iter()
                                .map(|(x, y)| [*x, *y])
                                .collect();
                            plot_ui.line(Line::new("Temperature", points).color(egui::Color32::from_rgb(231, 76, 60)));
                        });
                });
            }2
        });

        ctx.request_repaint_after(Duration::from_millis(200));
    }
}

fn monitoring_thread(state: Arc<Mutex<AppState>>, ctx: egui::Context, rx: Receiver<ServoCommand>) {
    let mut servo_connection: Option<ST3215> = None;
    let mut cycle_count = 0u32;
    let mut cached_servo_ids: Vec<u8> = Vec::new();
    
    loop {
        // Essayer de se connecter si pas de connexion
        if servo_connection.is_none() {
            servo_connection = ST3215::new(PORT).ok();
            if servo_connection.is_some() {
                // Scanner les servos au démarrage
                if let Some(ref servo) = servo_connection {
                    cached_servo_ids = servo.list_servos();
                    let mut state = state.lock().unwrap();
                    state.connected = true;
                    state.servo_ids = cached_servo_ids.clone();
                }
            }
        }
        
        if let Some(ref servo) = servo_connection {
            // Traiter toutes les commandes en attente
            while let Ok(cmd) = rx.try_recv() {
                match cmd {
                    ServoCommand::Move { id, position, speed, acceleration } => {
                        // Activer le torque avant de bouger
                        let _ = servo.enable_torque(id);
                        thread::sleep(Duration::from_millis(10));
                        let _ = servo.move_to(id, position, speed, acceleration, false);
                    }
                    ServoCommand::EnableTorque { id } => {
                        let _ = servo.enable_torque(id);
                    }
                    ServoCommand::DisableTorque { id } => {
                        let _ = servo.disable_torque(id);
                    }
                    ServoCommand::ScanServos => {
                        cached_servo_ids = servo.list_servos();
                        let mut state = state.lock().unwrap();
                        state.servo_ids = cached_servo_ids.clone();
                    }
                }
            }
            
            // Lecture des données du servo sélectionné (lock court)
            let (selected_servo, start_time) = {
                let state = state.lock().unwrap();
                (state.selected_servo, state.start_time)
            };
            
            if let Some(servo_id) = selected_servo {
                if cached_servo_ids.contains(&servo_id) {
                    // Lire position et température à chaque cycle
                    let pos = servo.read_position(servo_id);
                    let temp = servo.read_temperature(servo_id);
                    
                    let voltage = if cycle_count % 5 == 0 {
                        servo.read_voltage(servo_id)
                    } else {
                        None
                    };
                    
                    let current = if cycle_count % 5 == 1 {
                        servo.read_current(servo_id)
                    } else {
                        None
                    };
                    
                    let speed = if cycle_count % 3 == 0 {
                        servo.read_speed(servo_id).map(|s| s as u16)
                    } else {
                        None
                    };
                    
                    // Mettre à jour l'état
                    let mut state = state.lock().unwrap();
                    let time = start_time.elapsed().as_secs_f64();
                    
                    if let Some(pos) = pos {
                        state.servo_data.position = Some(pos);
                        state.position_history.push((time, pos as f64));
                        if state.position_history.len() > 100 {
                            state.position_history.remove(0);
                        }
                    }
                    
                    if let Some(temp) = temp {
                        state.servo_data.temperature = Some(temp);
                        state.temperature_history.push((time, temp as f64));
                        if state.temperature_history.len() > 100 {
                            state.temperature_history.remove(0);
                        }
                    }
                    
                    if let Some(v) = voltage {
                        state.servo_data.voltage = Some(v);
                    }
                    
                    if let Some(c) = current {
                        state.servo_data.current = Some(c);
                    }
                    
                    if let Some(s) = speed {
                        state.servo_data.speed = Some(s);
                    }
                    
                    state.servo_data.last_update = Instant::now();
                }
            }
        } else {
            // Pas de connexion
            let mut state = state.lock().unwrap();
            state.connected = false;
        }
        
        cycle_count = cycle_count.wrapping_add(1);
        ctx.request_repaint();
        thread::sleep(Duration::from_millis(100));
    }
}

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1000.0, 800.0])
            .with_min_inner_size([800.0, 600.0])
            .with_icon(
                eframe::icon_data::from_png_bytes(&[]).unwrap_or_default(),
            ),
        ..Default::default()
    };
    
    eframe::run_native(
        "Cogni-Robot Servo Control",
        options,
        Box::new(|cc| Ok(Box::new(ServoGuiApp::new(cc)))),
    )
}
