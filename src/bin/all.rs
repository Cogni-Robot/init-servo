use eframe::egui;
use st3215::ST3215;
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;
use std::time::{Duration, Instant};

// --- CONSTANTES ---
const SERIAL_PORT: &str = "/dev/ttyACM0";
const MAX_SERVO_ID: u8 = 15;

// --- COMMANDES ---
enum AppCommand {
    Move { id: u8, position: u16, speed: u16 },
    ToggleTorque { id: u8, enable: bool },
}

// --- ÉTAT D'UN SERVO UNIQUE ---
#[derive(Clone, Debug)]
struct IndividualServo {
    id: u8,
    current_pos: u16,      // Position réelle lue
    target_pos: u16,       // Position du slider (consigne)
    temperature: u8,
    voltage: f32,
    load: f32,
    torque_on: bool,
}

// --- ÉTAT GLOBAL DE L'APPLICATION ---
struct SharedState {
    connected: bool,
    // On utilise BTreeMap pour qu'ils soient triés par ID (1, 2, 3...) automatiquement
    servos: BTreeMap<u8, IndividualServo>, 
}

impl Default for SharedState {
    fn default() -> Self {
        Self {
            connected: false,
            servos: BTreeMap::new(),
        }
    }
}

// --- APPLICATION GUI ---
struct MultiServoApp {
    state: Arc<Mutex<SharedState>>,
    tx: Sender<AppCommand>,
}

impl MultiServoApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let (tx, rx) = channel();
        let state = Arc::new(Mutex::new(SharedState::default()));

        // Configuration du style
        let mut style = (*cc.egui_ctx.style()).clone();
        style.visuals.window_corner_radius = egui::CornerRadius::same(8);
        style.spacing.item_spacing = egui::vec2(10.0, 10.0);
        cc.egui_ctx.set_style(style);

        // Lancement du thread de gestion des servos
        let state_clone = state.clone();
        let ctx_clone = cc.egui_ctx.clone();
        thread::spawn(move || {
            servo_worker(state_clone, rx, ctx_clone);
        });

        Self { state, tx }
    }
}

impl eframe::App for MultiServoApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let mut state = self.state.lock().unwrap();

        // --- EN-TÊTE ---
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                ui.heading("🤖 Multi-Servo Controller (1-15)");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if state.connected {
                        ui.colored_label(egui::Color32::GREEN, "● Connected");
                    } else {
                        ui.colored_label(egui::Color32::RED, "● Disconnected");
                    }
                });
            });
            ui.add_space(8.0);
        });

        // --- ZONE PRINCIPALE (SCROLLABLE) ---
        egui::CentralPanel::default().show(ctx, |ui| {
            if state.servos.is_empty() && state.connected {
                ui.centered_and_justified(|ui| {
                    ui.label("Scanning IDs 1 to 15... No servos found yet.");
                });
            } else if !state.connected {
                 ui.centered_and_justified(|ui| {
                    ui.heading("Connecting to Serial Port...");
                });
            } else {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    // On itère sur tous les servos trouvés pour afficher leur contrôles
                    for (id, servo) in state.servos.iter_mut() {
                        ui.push_id(*id, |ui| {
                            draw_servo_card(ui, servo, &self.tx);
                        });
                    }
                });
            }
        });
    }
}

// --- COMPOSANT GRAPHIQUE POUR UN SERVO ---
fn draw_servo_card(ui: &mut egui::Ui, servo: &mut IndividualServo, tx: &Sender<AppCommand>) {
    egui::Frame::group(ui.style())
        .inner_margin(10.0)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                // ID et Température
                ui.colored_label(egui::Color32::LIGHT_BLUE, format!("ID {}", servo.id));
                ui.separator();
                
                // Indicateur Température
                let temp_color = if servo.temperature > 60 { egui::Color32::RED } else { egui::Color32::GRAY };
                ui.colored_label(temp_color, format!("{}°C", servo.temperature));
                
                // Indicateur Voltage
                ui.label(format!("{:.1}V", servo.voltage));
                
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // Bouton Torque
                    let btn_text = if servo.torque_on { "Torque ON" } else { "Torque OFF" };
                    let btn = ui.button(btn_text);
                    if btn.clicked() {
                        servo.torque_on = !servo.torque_on;
                        let _ = tx.send(AppCommand::ToggleTorque { 
                            id: servo.id, 
                            enable: servo.torque_on 
                        });
                    }
                });
            });

            ui.add_space(5.0);

            // Slider de Position
            ui.horizontal(|ui| {
                ui.label("Pos:");
                // Slider qui contrôle 'target_pos'
                let slider = ui.add(egui::Slider::new(&mut servo.target_pos, 0..=4095)
                    .text("Target"));
                
                // Si l'utilisateur bouge le slider, on envoie la commande
                if slider.changed() {
                    let _ = tx.send(AppCommand::Move { 
                        id: servo.id, 
                        position: servo.target_pos, 
                        speed: 0 // 0 = vitesse max ou par défaut selon config
                    });
                }
                
                // Affichage de la position réelle (feedback)
                ui.label(format!("(Real: {})", servo.current_pos));
            });
            
            // Barre de charge (Load)
            let load_pct = (servo.load.abs() / 1000.0).clamp(0.0, 1.0);
            ui.add(egui::ProgressBar::new(load_pct).text("Load"));
        });
}

// --- BACKEND (THREAD) ---
fn servo_worker(state: Arc<Mutex<SharedState>>, rx: Receiver<AppCommand>, ctx: egui::Context) {
    let mut driver_opt: Option<ST3215> = None;

    loop {
        // 1. Tentative de connexion si pas connecté
        if driver_opt.is_none() {
            if let Ok(mut driver) = ST3215::new(SERIAL_PORT) {
                println!("Serial Open. Scanning 1-15...");
                let mut detected_servos = BTreeMap::new();

                // 2. SCAN INITIAL (1 à 15)
                for id in 1..=MAX_SERVO_ID {
                    // On essaie de lire la position pour voir si le servo existe
                    if let Some(pos) = driver.read_position(id) {
                        println!("Found Servo ID {}", id);
                        let temp = driver.read_temperature(id).unwrap_or(0);
                        let volt = driver.read_voltage(id).unwrap_or(0.0);
                        
                        // Création de l'état initial
                        detected_servos.insert(id, IndividualServo {
                            id,
                            current_pos: pos,
                            target_pos: pos, // IMPORTANT: Le slider commence à la position actuelle !
                            temperature: temp,
                            voltage: volt,
                            load: 0.0,
                            torque_on: false, // Par défaut souvent off au démarrage
                        });
                    }
                }

                // Mise à jour de l'état partagé
                let mut s = state.lock().unwrap();
                s.connected = true;
                s.servos = detected_servos;
                driver_opt = Some(driver);
            }
        }

        // 3. Boucle principale de communication
        if let Some(ref mut driver) = driver_opt {
            // A. Traitement des commandes UI (Move, Torque)
            while let Ok(cmd) = rx.try_recv() {
                match cmd {
                    AppCommand::Move { id, position, speed } => {
                        // On assume speed=0 pour vitesse max, time=0
                        let _ = driver.move_to(id, position, speed, 50, false); // Accel à 50 arbitraire
                    }
                    AppCommand::ToggleTorque { id, enable } => {
                        if enable {
                            let _ = driver.enable_torque(id);
                        } else {
                            let _ = driver.disable_torque(id);
                        }
                    }
                }
            }

            // B. Mise à jour des infos (Polling)
            {
                let mut s = state.lock().unwrap();
                // On récupère la liste des IDs à mettre à jour
                let ids: Vec<u8> = s.servos.keys().cloned().collect();
                
                for id in ids {
                    if let Some(mut servo_state) = s.servos.get_mut(&id) {
                        // Lecture position réelle
                        if let Some(pos) = driver.read_position(id) {
                            servo_state.current_pos = pos;
                        }
                        // Lecture température/voltage/load (cycle court)
                        if let Some(temp) = driver.read_temperature(id) {
                            servo_state.temperature = temp;
                        }
                        if let Some(volt) = driver.read_voltage(id) {
                            servo_state.voltage = volt;
                        }
                         if let Some(load) = driver.read_load(id) {
                            servo_state.load = load as f32;
                        }
                    }
                }
            } // Release lock
            
            ctx.request_repaint(); // Rafraichir l'UI
        } else {
            // Pas de driver, on indique déconnecté
            let mut s = state.lock().unwrap();
            s.connected = false;
            // On attend avant de réessayer
            thread::sleep(Duration::from_secs(1));
        }

        thread::sleep(Duration::from_millis(20));
    }
}

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([500.0, 800.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Servo Control Panel",
        options,
        Box::new(|cc| Ok(Box::new(MultiServoApp::new(cc)))),
    )
}